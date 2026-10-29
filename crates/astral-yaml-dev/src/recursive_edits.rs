//! An independent tree and alias graph for composed structural edits.

use std::collections::{HashMap, HashSet};

use astral_yaml::{CollectionStyle, NodeId, ScalarStyle, comments::CommentPlacement};

use crate::Value;
use astral_yaml_serde::{
    Editable,
    edit::{Cursor, Node, NodeKind},
};

/// Some source graphs contain cycles. This projection leaves them available
/// for syntax inspection without expanding aliases during deserialization.
struct Graph;

impl<'de> serde::Deserialize<'de> for Graph {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Ok(Self)
    }
}

const TEXT: &[&str] = &[
    "old",
    "null",
    "true",
    "42",
    "",
    "two\nlines\n",
    "with: space",
    "\u{feff}",
    "\r\n",
    "1.5",
    "---",
    "...",
    "# comment",
    "? key",
    ": value",
    "- item",
    "tab\tinside",
    "line\n\nbreak",
    "nul\0inside",
    "é\u{2028}end",
    "\\literal",
    "quotes'\"",
];

#[derive(Clone, Debug)]
enum Scalar {
    String(usize),
    Key(usize),
    Null,
    Boolean(bool),
    Integer(i64),
}

impl Scalar {
    fn value(&self) -> String {
        match self {
            Self::String(index) => TEXT[*index].to_owned(),
            Self::Key(id) => format!("added-{id}"),
            Self::Null => "null".to_owned(),
            Self::Boolean(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
        }
    }

    fn yaml(&self) -> String {
        match self {
            Self::String(index) => serde_json::to_string(TEXT[*index]).unwrap(),
            Self::Key(_) => serde_json::to_string(&self.value()).unwrap(),
            _ => self.value(),
        }
    }
}

#[derive(Clone, Debug)]
enum Kind {
    Scalar(Scalar),
    Sequence(Vec<Tree>),
    Mapping(Vec<(Tree, Tree)>),
    Alias(usize),
}

#[derive(Clone, Debug)]
struct Tree {
    id: usize,
    anchor: Option<String>,
    tag: Option<String>,
    comments: Vec<(String, CommentPlacement)>,
    kind: Kind,
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Item(usize),
    Key(usize),
    Value(usize),
}

type Path = Vec<Step>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Operation {
    Replace,
    InsertItem,
    RemoveItem,
    InsertEntry,
    RemoveEntry,
    Flow,
    Block,
    CancelInsertion,
    InsertComment,
    ReplaceComment,
    RemoveComment,
    CancelComment,
}

impl Operation {
    fn from_byte(byte: u8) -> Self {
        const OPERATIONS: [Operation; 12] = [
            Operation::Replace,
            Operation::InsertItem,
            Operation::RemoveItem,
            Operation::InsertEntry,
            Operation::RemoveEntry,
            Operation::Flow,
            Operation::Block,
            Operation::CancelInsertion,
            Operation::InsertComment,
            Operation::ReplaceComment,
            Operation::RemoveComment,
            Operation::CancelComment,
        ];
        OPERATIONS[usize::from(byte) % OPERATIONS.len()]
    }
}

impl Tree {
    fn children(&self) -> Vec<&Self> {
        match &self.kind {
            Kind::Sequence(items) => items.iter().collect(),
            Kind::Mapping(entries) => entries
                .iter()
                .flat_map(|(key, value)| [key, value])
                .collect(),
            _ => Vec::new(),
        }
    }

    fn nodes(&self) -> Vec<&Self> {
        let mut nodes = Vec::new();
        let mut pending = vec![self];
        while let Some(node) = pending.pop() {
            nodes.push(node);
            pending.extend(node.children());
        }
        nodes
    }

    fn paths(&self) -> Vec<Path> {
        let mut paths = Vec::new();
        let mut pending = vec![(self, Vec::new())];
        while let Some((node, path)) = pending.pop() {
            let children: Vec<_> = match &node.kind {
                Kind::Sequence(items) => items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| (Step::Item(i), item))
                    .collect(),
                Kind::Mapping(entries) => entries
                    .iter()
                    .enumerate()
                    .flat_map(|(i, (key, value))| [(Step::Key(i), key), (Step::Value(i), value)])
                    .collect(),
                _ => Vec::new(),
            };
            for (step, child) in children {
                let mut path = path.clone();
                path.push(step);
                pending.push((child, path));
            }
            paths.push(path);
        }
        paths
    }

    fn at(&self, path: &[Step]) -> &Self {
        let mut node = self;
        for step in path {
            node = match (&node.kind, step) {
                (Kind::Sequence(items), Step::Item(i)) => &items[*i],
                (Kind::Mapping(entries), Step::Key(i)) => &entries[*i].0,
                (Kind::Mapping(entries), Step::Value(i)) => &entries[*i].1,
                _ => unreachable!("model path"),
            };
        }
        node
    }

    fn at_mut(&mut self, path: &[Step]) -> &mut Self {
        let mut node = self;
        for step in path {
            node = match (&mut node.kind, step) {
                (Kind::Sequence(items), Step::Item(i)) => &mut items[*i],
                (Kind::Mapping(entries), Step::Key(i)) => &mut entries[*i].0,
                (Kind::Mapping(entries), Step::Value(i)) => &mut entries[*i].1,
                _ => unreachable!("model path"),
            };
        }
        node
    }

    fn yaml(&self) -> String {
        let anchors: HashMap<_, _> = self
            .nodes()
            .into_iter()
            .filter_map(|node| node.anchor.as_ref().map(|name| (node.id, name.as_str())))
            .collect();
        self.render(&anchors)
    }

    fn render(&self, anchors: &HashMap<usize, &str>) -> String {
        let body = match &self.kind {
            Kind::Scalar(value) => value.yaml(),
            Kind::Sequence(items) => format!(
                "[{}]",
                items
                    .iter()
                    .map(|item| item.render(anchors))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Kind::Mapping(entries) => format!(
                "{{{}}}",
                entries
                    .iter()
                    .map(|(key, value)| format!(
                        "? {} : {}",
                        key.render(anchors),
                        value.render(anchors)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Kind::Alias(target) => format!("*{}", anchors[target]),
        };
        let body = self
            .anchor
            .as_ref()
            .map_or(body.clone(), |name| format!("&{name} {body}"));
        let body = self
            .tag
            .as_ref()
            .map_or(body.clone(), |tag| format!("!<{tag}> {body}"));
        if self.comments.is_empty() {
            body
        } else {
            let mut output = String::from("\n");
            for (text, placement) in &self.comments {
                assert_eq!(*placement, CommentPlacement::Leading);
                output.push('#');
                output.push_str(text);
                output.push('\n');
            }
            output.push_str(&body);
            output
        }
    }

    fn value(&self) -> Value {
        match &self.kind {
            Kind::Scalar(Scalar::Null) => Value::Null,
            Kind::Scalar(Scalar::Boolean(value)) => Value::Bool(*value),
            Kind::Scalar(Scalar::Integer(value)) => Value::Integer(*value),
            Kind::Scalar(text) => Value::String(text.value()),
            Kind::Sequence(items) => Value::Sequence(items.iter().map(Self::value).collect()),
            Kind::Mapping(entries) => Value::Mapping(
                entries
                    .iter()
                    .map(|(k, v)| (k.value(), v.value()))
                    .collect(),
            ),
            Kind::Alias(_) => unreachable!("generated data has no aliases"),
        }
    }

    fn can_remove(&self, removed: &HashSet<usize>) -> bool {
        self.nodes().into_iter().all(|node| match node.kind {
            Kind::Alias(target) => removed.contains(&node.id) || !removed.contains(&target),
            _ => true,
        })
    }
}

struct Builder {
    next_id: usize,
    state: u64,
}

impl Builder {
    fn identity(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn document(&mut self) -> Tree {
        if self.state % 8 != 0 {
            return self.tree(3, &[]);
        }
        // Two definitions deliberately reuse a name. Each self reference and
        // following alias targets its own definition, not a global name lookup.
        let root = self.identity();
        let mut items = Vec::new();
        for _ in 0..2 {
            let definition = self.identity();
            let value = self.tree(1, &[definition]);
            let recursive = Tree {
                id: self.identity(),
                anchor: None,
                tag: None,
                comments: Vec::new(),
                kind: Kind::Alias(definition),
            };
            items.push(Tree {
                id: definition,
                anchor: Some("shared".to_owned()),
                tag: None,
                comments: Vec::new(),
                kind: Kind::Sequence(vec![value, recursive]),
            });
            items.push(Tree {
                id: self.identity(),
                anchor: None,
                tag: None,
                comments: Vec::new(),
                kind: Kind::Alias(definition),
            });
        }
        Tree {
            id: root,
            anchor: None,
            tag: None,
            comments: Vec::new(),
            kind: Kind::Sequence(items),
        }
    }

    fn next(&mut self) -> usize {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(self.state >> 32).unwrap()
    }

    fn value(&mut self, depth: usize) -> Tree {
        let id = self.identity();
        let choice = self.next();
        let kind = if depth == 0 || choice % 3 == 0 {
            Kind::Scalar(match choice % 4 {
                0 => Scalar::Null,
                1 => Scalar::Boolean(choice & 4 == 0),
                2 => Scalar::Integer([0, -1, 42][choice % 3]),
                _ => Scalar::String(choice % TEXT.len()),
            })
        } else if choice % 2 == 0 {
            Kind::Sequence(
                (0..(choice >> 8) % 4)
                    .map(|_| self.value(depth - 1))
                    .collect(),
            )
        } else {
            Kind::Mapping(
                (0..(choice >> 8) % 4)
                    .map(|_| (self.key(), self.value(depth - 1)))
                    .collect(),
            )
        };
        Tree {
            id,
            anchor: None,
            tag: None,
            comments: Vec::new(),
            kind,
        }
    }

    fn key(&mut self) -> Tree {
        let id = self.identity();
        Tree {
            id,
            anchor: None,
            tag: None,
            comments: Vec::new(),
            kind: Kind::Scalar(Scalar::Key(id)),
        }
    }

    fn tree(&mut self, depth: usize, ancestors: &[usize]) -> Tree {
        let id = self.identity();
        let choice = self.next();
        let comments = (choice % 7 == 0)
            .then(|| (format!(" generated {id}"), CommentPlacement::Leading))
            .into_iter()
            .collect();
        if choice % 5 == 0 && !ancestors.is_empty() {
            return Tree {
                id,
                anchor: None,
                tag: None,
                comments,
                kind: Kind::Alias(ancestors[choice % ancestors.len()]),
            };
        }
        let anchor = (choice % 3 == 0).then(|| format!("n{id}"));
        let mut ancestors = ancestors.to_vec();
        if anchor.is_some() {
            ancestors.push(id);
        }
        let kind = if depth == 0 || choice % 4 == 0 {
            Kind::Scalar(match self.next() % 4 {
                0 => Scalar::Null,
                1 => Scalar::Boolean(choice & 4 == 0),
                2 => Scalar::Integer([0, -1, 42][choice % 3]),
                _ => Scalar::String(choice % TEXT.len()),
            })
        } else if choice % 2 == 0 {
            Kind::Sequence(
                (0..(choice >> 8) % 4)
                    .map(|_| self.tree(depth - 1, &ancestors))
                    .collect(),
            )
        } else {
            Kind::Mapping(
                (0..(choice >> 8) % 4)
                    .map(|_| {
                        (
                            self.tree(depth - 1, &ancestors),
                            self.tree(depth - 1, &ancestors),
                        )
                    })
                    .collect(),
            )
        };
        Tree {
            id,
            anchor,
            tag: (choice % 5 == 0).then(|| "tag:example.com,2026:node".to_owned()),
            comments,
            kind,
        }
    }
}

fn with_node<R>(cursor: Cursor<'_, '_>, path: &[Step], f: impl FnOnce(Cursor<'_, '_>) -> R) -> R {
    let Some((step, rest)) = path.split_first() else {
        return f(cursor);
    };
    match *step {
        Step::Item(index) => with_node(cursor.item(index).unwrap(), rest, f),
        Step::Key(index) => with_node(
            cursor.into_mapping().unwrap().key_at(index).unwrap(),
            rest,
            f,
        ),
        Step::Value(index) => with_node(
            cursor.into_mapping().unwrap().value_at(index).unwrap(),
            rest,
            f,
        ),
    }
}

fn check<'a, Location: 'a>(
    model: &Tree,
    root: NodeId,
    node: impl Fn(NodeId) -> &'a Node<Location>,
) {
    let mut ids = HashMap::new();
    let mut aliases = Vec::new();
    let mut pending = vec![(model, root)];
    while let Some((expected, id)) = pending.pop() {
        assert!(ids.insert(expected.id, id).is_none());
        let actual = node(id);
        assert_eq!(actual.anchor(), expected.anchor.as_deref());
        assert_eq!(actual.tag(), expected.tag.as_deref());
        let comments: Vec<_> = actual
            .comments()
            .iter()
            .map(|comment| (comment.text(), comment.placement()))
            .collect();
        let expected_comments: Vec<_> = expected
            .comments
            .iter()
            .map(|(text, placement)| (text.as_str(), *placement))
            .collect();
        assert_eq!(
            comments, expected_comments,
            "comments on occurrence {}",
            expected.id
        );
        match (&expected.kind, actual.kind()) {
            (Kind::Scalar(expected), NodeKind::Scalar { value, style }) => {
                assert_eq!(*value, expected.value());
                match expected {
                    Scalar::String(1 | 2 | 3 | 4 | 9) => assert_ne!(
                        *style,
                        ScalarStyle::Plain,
                        "string must not acquire an implicit null/bool/number type"
                    ),
                    Scalar::String(_) | Scalar::Key(_) => {}
                    _ => assert_eq!(*style, ScalarStyle::Plain),
                }
            }
            (Kind::Sequence(expected), NodeKind::Sequence { items, .. }) => {
                assert_eq!(expected.len(), items.len());
                pending.extend(expected.iter().zip(items.iter().copied()));
            }
            (Kind::Mapping(expected), NodeKind::Mapping { entries, .. }) => {
                assert_eq!(expected.len(), entries.len());
                for ((key, value), &(a, b)) in expected.iter().zip(entries) {
                    pending.extend([(key, a), (value, b)]);
                }
            }
            (Kind::Alias(expected), NodeKind::Alias { target }) => {
                aliases.push((*expected, *target));
            }
            _ => panic!(
                "model shape differs: {:?} != {:?}",
                expected.kind,
                actual.kind()
            ),
        }
    }
    for (expected, actual) in aliases {
        assert_eq!(ids[&expected], actual, "alias identity");
    }
}

fn check_draft(model: &Tree, root: NodeId, editor: &mut Cursor<'_, '_>) {
    let mut nodes = HashMap::new();
    let mut pending = vec![(root, Vec::new())];
    while let Some((id, path)) = pending.pop() {
        let node = with_node(editor.reborrow(), &path, |cursor| {
            cursor.node().unwrap().clone()
        });
        let mut add = |child, step| {
            let mut path = path.clone();
            path.push(step);
            pending.push((child, path));
        };
        match node.kind() {
            NodeKind::Sequence { items, .. } => {
                for (index, &item) in items.iter().enumerate() {
                    add(item, Step::Item(index));
                }
            }
            NodeKind::Mapping { entries, .. } => {
                for (index, &(key, value)) in entries.iter().enumerate() {
                    add(key, Step::Key(index));
                    add(value, Step::Value(index));
                }
            }
            _ => {}
        }
        assert!(nodes.insert(id, node).is_none(), "one structural owner");
    }
    check(model, root, |id| &nodes[&id]);
}

fn set_layout(model: &Tree, editor: &mut Cursor<'_, '_>, style: CollectionStyle) {
    for path in model.paths() {
        let kind = &model.at(&path).kind;
        let empty = match kind {
            Kind::Sequence(items) => items.is_empty(),
            Kind::Mapping(entries) => entries.is_empty(),
            Kind::Scalar(Scalar::String(_)) if style == CollectionStyle::Flow => {
                with_node(editor.reborrow(), &path, |mut node| {
                    node.set_scalar_style(ScalarStyle::DoubleQuoted).unwrap();
                });
                continue;
            }
            _ => continue,
        };
        // A flow collection also constrains every descendant. Keep delimiters
        // around detached suffixes that may precede an ancestor's trailing
        // comments, suffixes followed by inline comments, and leading/inline
        // pairs without a property header.
        let delimiter = (0..=path.len()).any(|depth| {
            let node = model.at(&path[..depth]);
            let has = |placement| {
                node.comments
                    .iter()
                    .any(|(_, existing)| *existing == placement)
            };
            let closing_inline = node
                .comments
                .iter()
                .position(|(_, placement)| *placement == CommentPlacement::Inline)
                .is_some_and(|index| {
                    node.comments[..index].iter().any(|(_, placement)| {
                        matches!(
                            placement,
                            CommentPlacement::Trailing | CommentPlacement::Detached
                        )
                    })
                });
            depth != 0
                && (closing_inline
                    || has(CommentPlacement::Detached)
                    || (node.anchor.is_none()
                        && node.tag.is_none()
                        && has(CommentPlacement::Leading)
                        && has(CommentPlacement::Inline)))
        });
        let requested = if empty || (style == CollectionStyle::Block && delimiter) {
            CollectionStyle::Flow
        } else {
            style
        };
        with_node(editor.reborrow(), &path, |mut node| {
            node.set_collection_style(requested).unwrap();
        });
    }
}

fn replace(model: &mut Tree, editor: &mut Cursor<'_, '_>, builder: &mut Builder, selector: usize) {
    let paths: Vec<_> = model
        .paths()
        .into_iter()
        .filter(|path| {
            let root = model.at(path);
            let removed = root
                .nodes()
                .into_iter()
                .filter(|node| node.id != root.id)
                .map(|node| node.id)
                .collect();
            model.can_remove(&removed)
        })
        .collect();
    let path = &paths[selector % paths.len()];
    let old = model.at(path);
    let mut replacement = builder.value(selector % 3);
    // Nested scalar suffixes have no attachment boundary. Keep an enclosing
    // collection when the replaced occurrence owns full-line suffixes.
    if !path.is_empty()
        && old.comments.iter().any(|(_, placement)| {
            matches!(
                placement,
                CommentPlacement::Trailing | CommentPlacement::Detached
            )
        })
    {
        let child = replacement;
        replacement = Tree {
            id: builder.identity(),
            anchor: None,
            tag: None,
            comments: Vec::new(),
            kind: Kind::Sequence(vec![child]),
        };
    }
    with_node(editor.reborrow(), path, |mut cursor| {
        cursor.set(replacement.value()).unwrap();
    });
    replacement.id = old.id;
    replacement.anchor.clone_from(&old.anchor);
    replacement.comments.clone_from(&old.comments);
    *model.at_mut(path) = replacement;
}

fn comment_edit(
    model: &mut Tree,
    editor: &mut Cursor<'_, '_>,
    selector: usize,
    operation: Operation,
) {
    let paths: Vec<_> = model
        .paths()
        .into_iter()
        .filter(|path| {
            matches!(
                operation,
                Operation::InsertComment | Operation::CancelComment
            ) || !model.at(path).comments.is_empty()
        })
        .collect();
    if paths.is_empty() {
        return;
    }
    let path = &paths[selector % paths.len()];
    let children = match &model.at(path).kind {
        Kind::Sequence(items) => Some(items.len()),
        Kind::Mapping(entries) => Some(entries.len()),
        _ => None,
    };
    let comments = &mut model.at_mut(path).comments;
    let text = format!(" comment {selector}");
    match operation {
        Operation::InsertComment | Operation::CancelComment => {
            let mut placement = [
                CommentPlacement::Leading,
                CommentPlacement::Inline,
                CommentPlacement::Trailing,
                CommentPlacement::Detached,
            ][selector % 4];
            if !path.is_empty()
                && ((children.is_none()
                    && matches!(
                        placement,
                        CommentPlacement::Trailing | CommentPlacement::Detached
                    ))
                    || (children == Some(0) && placement == CommentPlacement::Trailing))
            {
                placement = CommentPlacement::Leading;
            }
            if placement == CommentPlacement::Inline
                && comments.iter().any(|(_, existing)| *existing == placement)
            {
                placement = CommentPlacement::Leading;
            }
            let closing_inline = (placement == CommentPlacement::Inline && selector & 4 != 0)
                || (comments
                    .last()
                    .is_some_and(|(_, placement)| *placement == CommentPlacement::Inline)
                    && comments.iter().any(|(_, placement)| {
                        matches!(
                            placement,
                            CommentPlacement::Trailing | CommentPlacement::Detached
                        )
                    }));
            let rank = |placement| match placement {
                CommentPlacement::Leading => 0,
                CommentPlacement::Inline if closing_inline => 4,
                CommentPlacement::Inline => 1,
                CommentPlacement::Trailing => 2,
                CommentPlacement::Detached => 3,
            };
            if comments
                .last()
                .is_some_and(|(_, existing)| rank(*existing) > rank(placement))
            {
                return;
            }
            with_node(editor.reborrow(), path, |mut node| {
                node.append_comment(&text, placement).unwrap();
                if operation == Operation::CancelComment {
                    node.remove_comment(comments.len()).unwrap();
                } else {
                    comments.push((text, placement));
                }
            });
        }
        Operation::ReplaceComment => {
            let index = selector % comments.len();
            with_node(editor.reborrow(), path, |mut node| {
                node.replace_comment(index, &text).unwrap();
            });
            comments[index].0 = text;
        }
        Operation::RemoveComment => {
            let index = selector % comments.len();
            with_node(editor.reborrow(), path, |mut node| {
                node.remove_comment(index).unwrap();
            });
            comments.remove(index);
        }
        _ => unreachable!("comment operation"),
    }
}

fn sequence_edit(
    model: &mut Tree,
    editor: &mut Cursor<'_, '_>,
    builder: &mut Builder,
    selector: usize,
    operation: Operation,
) {
    let paths: Vec<_> = model
        .paths()
        .into_iter()
        .filter(|path| matches!(model.at(path).kind, Kind::Sequence(_)))
        .collect();
    if paths.is_empty() {
        return;
    }
    let path = &paths[selector % paths.len()];
    let Kind::Sequence(items) = &model.at(path).kind else {
        unreachable!()
    };
    if operation == Operation::RemoveItem {
        let indices: Vec<_> = items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                let removed = item.nodes().into_iter().map(|node| node.id).collect();
                model.can_remove(&removed).then_some(i)
            })
            .collect();
        if indices.is_empty() {
            return;
        }
        if items.len() == 1
            && !path.is_empty()
            && model
                .at(path)
                .comments
                .iter()
                .any(|(_, placement)| *placement == CommentPlacement::Trailing)
        {
            return;
        }
        let index = indices[selector % indices.len()];
        with_node(editor.reborrow(), path, |node| {
            node.into_sequence().unwrap().remove(index).unwrap();
        });
        let Kind::Sequence(items) = &mut model.at_mut(path).kind else {
            unreachable!()
        };
        items.remove(index);
        if items.is_empty() {
            with_node(editor.reborrow(), path, |mut node| {
                node.set_collection_style(CollectionStyle::Flow).unwrap();
            });
        }
    } else {
        let index = selector % (items.len() + 1);
        let item = builder.value(1);
        with_node(editor.reborrow(), path, |node| {
            let mut sequence = node.into_sequence().unwrap();
            sequence.insert(index, item.value()).unwrap();
            if operation == Operation::CancelInsertion {
                sequence.remove(index).unwrap();
            }
        });
        if operation != Operation::CancelInsertion {
            let Kind::Sequence(items) = &mut model.at_mut(path).kind else {
                unreachable!()
            };
            items.insert(index, item);
        }
    }
}

fn mapping_edit(
    model: &mut Tree,
    editor: &mut Cursor<'_, '_>,
    builder: &mut Builder,
    selector: usize,
    operation: Operation,
) {
    let paths: Vec<_> = model
        .paths()
        .into_iter()
        .filter(|path| matches!(model.at(path).kind, Kind::Mapping(_)))
        .collect();
    if paths.is_empty() {
        return;
    }
    let path = &paths[selector % paths.len()];
    let Kind::Mapping(entries) = &model.at(path).kind else {
        unreachable!()
    };
    if operation == Operation::RemoveEntry {
        let indices: Vec<_> = entries
            .iter()
            .enumerate()
            .filter_map(|(i, (key, value))| {
                let removed = key
                    .nodes()
                    .into_iter()
                    .chain(value.nodes())
                    .map(|node| node.id)
                    .collect();
                (matches!(key.kind, Kind::Scalar(Scalar::Key(_))) && model.can_remove(&removed))
                    .then_some(i)
            })
            .collect();
        if indices.is_empty() {
            return;
        }
        if entries.len() == 1
            && !path.is_empty()
            && model
                .at(path)
                .comments
                .iter()
                .any(|(_, placement)| *placement == CommentPlacement::Trailing)
        {
            return;
        }
        let index = indices[selector % indices.len()];
        let Kind::Scalar(key) = &entries[index].0.kind else {
            unreachable!()
        };
        with_node(editor.reborrow(), path, |node| {
            assert!(node.into_mapping().unwrap().remove(&key.value()).unwrap());
        });
        let Kind::Mapping(entries) = &mut model.at_mut(path).kind else {
            unreachable!()
        };
        entries.remove(index);
        if entries.is_empty() {
            with_node(editor.reborrow(), path, |mut node| {
                node.set_collection_style(CollectionStyle::Flow).unwrap();
            });
        }
    } else {
        let key = builder.key();
        let Kind::Scalar(name) = &key.kind else {
            unreachable!()
        };
        let value = builder.value(1);
        with_node(editor.reborrow(), path, |node| {
            node.into_mapping()
                .unwrap()
                .set(&name.value(), value.value())
                .unwrap();
        });
        let Kind::Mapping(entries) = &mut model.at_mut(path).kind else {
            unreachable!()
        };
        entries.push((key, value));
    }
}

/// Generate a recursive syntax graph, then compare each modeled operation and commit
/// with independently updated child lists, scalar values and alias target IDs.
///
/// # Panics
/// Panics if a generated valid edit fails or differs from the model.
pub fn check_recursive_edits(data: &[u8]) {
    if data.len() < 4 || data.len() > 512 {
        return;
    }
    // The operation bytes also seed the initial graph, so fuzzing can explore
    // more shapes than the batch-control byte alone can select.
    let mut seed = [0; 8];
    let length = data.len().min(seed.len());
    seed[..length].copy_from_slice(&data[..length]);
    let mut builder = Builder {
        next_id: 0,
        state: u64::from_le_bytes(seed),
    };
    let mut model = builder.document();
    let yaml = model.yaml();
    let source = match (data[0] >> 4) % 4 {
        0 => yaml,
        1 => format!("--- {yaml}\n...\n"),
        2 => format!("%YAML 1.2\n---\n{yaml}\n...\n"),
        _ => format!("\u{feff}---\n{yaml}\n"),
    };
    let mut document = Editable::<Graph>::parse(&source).unwrap();
    check(&model, document.syntax().root(), |id| {
        document.syntax().node(id).unwrap()
    });
    let batch = [1, 4, 16][usize::from(data[0] >> 6) % 3];
    for operations in data[1..]
        .chunks_exact(3)
        .take(32)
        .collect::<Vec<_>>()
        .chunks(batch)
        // Commit explicit styles before subsequent edits can require a different
        // presentation. The model generates successful edits, not style errors.
        .flat_map(|batch| {
            batch.split_inclusive(|op| {
                matches!(
                    Operation::from_byte(op[0]),
                    Operation::Flow | Operation::Block
                )
            })
        })
    {
        let root = document.syntax().root();
        let unchanged = operations
            .iter()
            .all(|op| {
                matches!(
                    Operation::from_byte(op[0]),
                    Operation::CancelInsertion | Operation::CancelComment
                )
            })
            .then(|| document.source().to_owned());
        document
            .edit(|mut cursor| {
                let editor = &mut cursor;
                for operation in operations {
                    let selector = usize::from(operation[1]) * 256 + usize::from(operation[2]);
                    match Operation::from_byte(operation[0]) {
                        Operation::Replace => replace(&mut model, editor, &mut builder, selector),
                        op @ (Operation::InsertItem
                        | Operation::RemoveItem
                        | Operation::CancelInsertion) => {
                            sequence_edit(&mut model, editor, &mut builder, selector, op);
                        }
                        op @ (Operation::InsertEntry | Operation::RemoveEntry) => {
                            mapping_edit(&mut model, editor, &mut builder, selector, op);
                        }
                        Operation::Flow => set_layout(&model, editor, CollectionStyle::Flow),
                        Operation::Block => set_layout(&model, editor, CollectionStyle::Block),
                        op @ (Operation::InsertComment
                        | Operation::ReplaceComment
                        | Operation::RemoveComment
                        | Operation::CancelComment) => {
                            comment_edit(&mut model, editor, selector, op);
                        }
                    }
                    check_draft(&model, root, editor);
                }
                Ok(())
            })
            .unwrap_or_else(|error| {
                panic!(
                    "{error}\ninput={data:?}\nsource={}\nmodel={model:?}",
                    document.source()
                )
            });
        check(&model, document.syntax().root(), |id| {
            document.syntax().node(id).unwrap()
        });
        if let Some(source) = unchanged {
            assert_eq!(document.source(), source, "canceled edits retain source");
        }
    }
}
