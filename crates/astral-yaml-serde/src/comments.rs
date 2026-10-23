use crate::Error;
use astral_yaml::{NodeId, Stream, comments::CommentIndex};
use std::cell::{Cell, OnceCell};

pub use astral_yaml::comments::{AttachedComment, CommentPlacement};

pub(crate) struct CommentCache<'de> {
    document: usize,
    index: OnceCell<Result<CommentIndex<'de>, Error>>,
}

impl<'de> CommentCache<'de> {
    pub(crate) fn new(document: usize) -> Self {
        Self {
            document,
            index: OnceCell::new(),
        }
    }

    pub(crate) fn get(
        &self,
        stream: &Stream<'de>,
        id: NodeId,
        budget: &Cell<usize>,
    ) -> Result<&[AttachedComment<'de>], Error> {
        let index = self
            .index
            .get_or_init(|| {
                CommentIndex::with_budget(stream, self.document, budget).map_err(Error::from)
            })
            .as_ref()
            .map_err(Clone::clone)?;

        Ok(index.subtree(id))
    }
}
#[cfg(test)]
mod tests {
    use super::{Cell, CommentCache};

    #[test]
    fn indexing_is_bounded_and_cached_including_failures() {
        let stream = astral_yaml::parse("42 # note\n").unwrap();
        let root = stream.documents()[0].root;
        let mut spans = Vec::new();

        // Two lines, one node, and one comment ownership record.
        for limit in 0..=4 {
            let budget = Cell::new(limit);
            let cache = CommentCache::new(0);
            let result = cache.get(&stream, root, &budget);
            if limit == 4 {
                assert_eq!(result.unwrap().len(), 1);
                assert_eq!(budget.get(), 0);
                assert_eq!(cache.get(&stream, root, &budget).unwrap().len(), 1);
            } else {
                let error = result.unwrap_err();
                budget.set(100);
                assert_eq!(cache.get(&stream, root, &budget).unwrap_err(), error);
                assert_eq!(budget.get(), 100);
                spans.push((
                    format!("budget {limit}: {}", error.message),
                    error.span.unwrap(),
                ));
            }
        }
        insta::assert_snapshot!(astral_yaml_dev::span_snapshot(stream.source(), spans), @"
          |
        1 | 42 # note
          | ^^ ^^^^^^^ budget 1: deserialization resource limit exceeded
          | |  |
          | |  budget 3: deserialization resource limit exceeded
          | budget 0: deserialization resource limit exceeded
          | budget 2: deserialization resource limit exceeded
        ");
    }

    #[test]
    fn documents_without_comments_do_not_build_a_tree() {
        let stream = astral_yaml::parse("---\n1 # first\n---\n2\n").unwrap();
        let budget = Cell::new(0);
        let cache = CommentCache::new(1);
        assert_eq!(
            cache
                .get(&stream, stream.documents()[1].root, &budget)
                .unwrap(),
            []
        );
    }
}
