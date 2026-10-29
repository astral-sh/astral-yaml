//! Track source ranges as document markers and line breaks are inserted.
//! The original and edited syntax use the same offset mapping.

use super::{
    Node,
    draft::map_locations,
    view::{Emission, Syntax},
};
use astral_yaml::{NodeId, ParseOptions, Span};

/// Return each node's content range, excluding tags, anchors, and intervening trivia.
/// These ranges are separate because properties can start on a different line
/// or at a different indentation.
pub(super) fn contents(source: &str, nodes: &[astral_yaml::Node<'_>]) -> Vec<Span> {
    nodes
        .iter()
        .map(|node| {
            let properties_end = node
                .anchor
                .iter()
                .chain(node.tag.iter())
                .map(|property| property.span.end())
                .max()
                .unwrap_or(node.span.start());
            super::span(
                astral_yaml::comments::skip_trivia(source, properties_end).min(node.span.end()),
                node.span.end(),
            )
        })
        .collect()
}

#[derive(Clone, Debug)]
pub(super) struct Source {
    pub(super) text: String,
    pub(super) nodes: Vec<Node>,
    /// Parsed content ranges exclude tags, anchors, and their intervening trivia.
    pub(super) contents: Vec<Span>,
    pub(super) root: NodeId,
    pub(super) options: ParseOptions,
    pub(super) byte_order_marks: Vec<Span>,
    pub(super) start_marker: Option<Span>,
    pub(super) end_marker: Option<Span>,
    pub(super) extra_end_markers: Vec<Span>,
}

impl<'a> From<&'a Source> for Syntax<'a, Span> {
    fn from(source: &'a Source) -> Self {
        Self {
            nodes: &source.nodes,
            root: source.root,
        }
    }
}

impl Source {
    pub(super) fn emission<'a, Location>(
        &'a self,
        syntax: Syntax<'a, Location>,
    ) -> Emission<'a, Location> {
        Emission {
            syntax,
            source: self,
        }
    }
}

/// An insertion and the offset change it causes.
/// `move_empty_root` determines which side of the insertion an empty root belongs to.
struct Insertion {
    at: usize,
    length: usize,
    move_empty_root: bool,
}

#[derive(Default)]
struct Locations {
    insertions: Vec<Insertion>,
}

impl Locations {
    fn span(&self, span: Span, root: bool) -> Span {
        self.after(span, 0, root)
    }

    fn after(&self, mut span: Span, generation: usize, root: bool) -> Span {
        for insertion in &self.insertions[generation..] {
            if span.end() > insertion.at
                || (root
                    && insertion.move_empty_root
                    && span.is_empty()
                    && span.start() == insertion.at)
            {
                let start = if span.start() >= insertion.at {
                    span.start() + insertion.length
                } else {
                    span.start()
                };
                span = super::span(start, span.end() + insertion.length);
            }
        }
        span
    }
}

/// A document marker and the point at which its span was recorded.
/// Only later insertions can shift its span.
struct Marker {
    span: Span,
    generation: usize,
}

/// Source text with inserted markers and line breaks, and a map of the offset changes.
pub(super) struct Frame<'a> {
    original: &'a Source,
    text: String,
    locations: Locations,
    start: Option<Marker>,
    ends: Vec<Marker>,
}

/// Source and syntax with spans adjusted for inserted markers and line breaks.
pub(super) struct Prepared {
    pub(super) source: Source,
    nodes: Vec<Node<Option<Span>>>,
    root: NodeId,
}

impl Prepared {
    pub(super) fn syntax(&self) -> Syntax<'_, Option<Span>> {
        Syntax {
            nodes: &self.nodes,
            root: self.root,
        }
    }
}

impl<'a> Frame<'a> {
    pub(super) fn new(original: &'a Source) -> Self {
        Self {
            original,
            text: original.text.clone(),
            locations: Locations::default(),
            start: original.start_marker.map(|span| Marker {
                span,
                generation: 0,
            }),
            ends: original
                .end_marker
                .iter()
                .chain(&original.extra_end_markers)
                .map(|&span| Marker {
                    span,
                    generation: 0,
                })
                .collect(),
        }
    }

    fn insert(&mut self, at: usize, text: &str, move_empty_root: bool) {
        self.text.insert_str(at, text);
        self.locations.insertions.push(Insertion {
            at,
            length: text.len(),
            move_empty_root,
        });
    }

    pub(super) fn end_markers(&mut self, mut insertions: Vec<usize>) {
        // Descending order retains the coordinates used by the boundary plan.
        insertions.sort_unstable();
        while let Some(at) = insertions.pop() {
            let mut count = 1;
            while insertions.last() == Some(&at) {
                insertions.pop();
                count += 1;
            }
            let prefix = self.text[..at].trim_end_matches('\u{feff}');
            let separator = if !prefix.is_empty() && !prefix.ends_with('\n') {
                "\n"
            } else {
                ""
            };
            self.insert(at, &format!("{separator}{}", "...\n".repeat(count)), false);
            self.ends.extend((0..count).map(|index| {
                let start = at + separator.len() + index * "...\n".len();
                Marker {
                    span: super::span(start, start + 3),
                    generation: self.locations.insertions.len(),
                }
            }));
        }
    }

    pub(super) fn separate_root(&mut self, at: usize) {
        self.insert(at, "\n", true);
    }

    pub(super) fn start_marker(&mut self, at: usize) {
        self.insert(at, "---\n", false);
        self.start = Some(Marker {
            span: super::span(at, at + 3),
            generation: self.locations.insertions.len(),
        });
    }

    pub(super) fn finish(self, syntax: Syntax<'_, Option<Span>>) -> Prepared {
        let mut source_nodes: Vec<_> = self
            .original
            .nodes
            .iter()
            .cloned()
            .map(|node| map_locations(node, |span| self.locations.span(span, false)))
            .collect();
        let root = self.original.root;
        source_nodes[root.index()].span = self
            .locations
            .span(self.original.nodes[root.index()].span, true);

        let mut nodes: Vec<_> = syntax
            .nodes
            .iter()
            .cloned()
            .map(|node| {
                map_locations(node, |span| {
                    span.map(|span| self.locations.span(span, false))
                })
            })
            .collect();
        nodes[syntax.root.index()].span = syntax.nodes[syntax.root.index()]
            .span
            .map(|span| self.locations.span(span, true));

        let mut ends: Vec<_> = self
            .ends
            .into_iter()
            .map(|marker| self.locations.after(marker.span, marker.generation, false))
            .collect();
        ends.sort_unstable_by_key(|span| span.start());
        let source = Source {
            text: self.text,
            nodes: source_nodes,
            contents: self
                .original
                .contents
                .iter()
                .enumerate()
                .map(|(index, &span)| self.locations.span(span, index == root.index()))
                .collect(),
            root,
            options: self.original.options,
            byte_order_marks: self
                .original
                .byte_order_marks
                .iter()
                .map(|&mark| self.locations.span(mark, false))
                .collect(),
            start_marker: self
                .start
                .map(|marker| self.locations.after(marker.span, marker.generation, false)),
            end_marker: ends.first().copied(),
            extra_end_markers: ends.into_iter().skip(1).collect(),
        };
        Prepared {
            source,
            nodes,
            root: syntax.root,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Insertion, Locations};
    use crate::syntax::span;

    #[test]
    fn an_insertion_inside_a_span_moves_only_its_end() {
        let locations = Locations {
            insertions: vec![Insertion {
                at: 3,
                length: 2,
                move_empty_root: false,
            }],
        };
        assert_eq!(locations.span(span(1, 5), false), span(1, 7));
    }
}
