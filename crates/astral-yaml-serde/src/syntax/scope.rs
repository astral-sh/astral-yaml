//! Find an enclosing source region that can represent an edit.
//! A changed node may need its parent rewritten to supply YAML indicators or
//! preserve comments. Edits within a mapping key can also change the key's notation.

use super::emit::{self, Context, KeyForm};
use super::layout::Layout;
use super::lines::LineIndex;
use super::source::Source;
use super::view::Syntax;
use super::{Comment, CommentPosition, Node, NodeKind};
use astral_yaml::{CollectionStyle, NodeId, ScalarStyle, Span};

#[derive(Clone, Copy, Debug)]
enum KeySeparator {
    Missing,
    Separated,
    Adjacent,
}

#[derive(Clone, Copy, Debug)]
enum ValueHeader {
    Missing,
    Present,
    IndentlessSequence,
}

#[derive(Clone, Copy, Debug)]
enum Role {
    Document,
    Item {
        sequence: NodeId,
    },
    Key {
        mapping: NodeId,
        separator: KeySeparator,
        length_limited: bool,
    },
    Value {
        mapping: NodeId,
        header: ValueHeader,
    },
}

impl Role {
    fn owner(self) -> Option<NodeId> {
        match self {
            Self::Document => None,
            Self::Item { sequence } => Some(sequence),
            Self::Key { mapping, .. } | Self::Value { mapping, .. } => Some(mapping),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Slot {
    role: Role,
    /// Descendant edits can change an enclosing key's notation or length.
    enclosing_key: Option<NodeId>,
    flow: bool,
    own_line: bool,
    body: Span,
    outer_comments: bool,
}

pub(super) struct Scopes<'plan> {
    original: &'plan Source,
    candidate: Syntax<'plan, Option<Span>>,
    layout: &'plan Layout,
    slots: Vec<Slot>,
}

impl<'plan> Scopes<'plan> {
    pub(super) fn new(
        original: &'plan Source,
        candidate: Syntax<'plan, Option<Span>>,
        layout: &'plan Layout,
        lines: &mut LineIndex<'_>,
    ) -> Self {
        let mut slots = vec![None; original.nodes.len()];
        let mut order = Vec::new();
        let mut pending = vec![(original.root, Role::Document, None, false)];
        while let Some((id, role, enclosing_key, flow)) = pending.pop() {
            let node = &original.nodes[id.index()];
            let body = original.body_span(id);
            slots[id.index()] = Some(Slot {
                role,
                enclosing_key,
                flow,
                own_line: lines.at(body.start()).own_line(body.start()),
                body,
                outer_comments: node.comments.iter().any(|comment| {
                    comment.span.start() < body.start() || comment.span.end() > body.end()
                }),
            });
            order.push(id);
            let enclosing_key = enclosing_key.or(match role {
                Role::Key { mapping, .. } => Some(mapping),
                _ => None,
            });
            let flow = flow
                || layout.flow(id)
                || matches!(
                    candidate.nodes[id.index()].kind,
                    NodeKind::Sequence {
                        style: CollectionStyle::Flow,
                        ..
                    } | NodeKind::Mapping {
                        style: CollectionStyle::Flow,
                        ..
                    }
                );
            match &node.kind {
                NodeKind::Sequence { items, .. } => {
                    pending.extend(
                        items
                            .iter()
                            .map(|&item| (item, Role::Item { sequence: id }, enclosing_key, flow)),
                    );
                }
                NodeKind::Mapping { entries, .. } => {
                    let mut entry_start = original.contents[id.index()].start();
                    for &(key, value) in entries {
                        // Only an implicit block or compact-pair key starts at the
                        // next token. Other keys follow a brace, comma, or '?'.
                        let length_limited =
                            astral_yaml::comments::skip_trivia(&original.text, entry_start)
                                == original.nodes[key.index()].span.start();
                        entry_start = original.nodes[value.index()].span.end();
                        let separator = key_separator(original, key);
                        let header = value_header(original, id, value, separator, lines);
                        pending.push((
                            key,
                            Role::Key {
                                mapping: id,
                                separator,
                                length_limited,
                            },
                            enclosing_key,
                            flow,
                        ));
                        pending.push((
                            value,
                            Role::Value {
                                mapping: id,
                                header,
                            },
                            enclosing_key,
                            flow,
                        ));
                    }
                }
                _ => {}
            }
        }
        let mut slots: Vec<_> = slots
            .into_iter()
            .map(|slot| slot.expect("parsed occurrence belongs to tree"))
            .collect();
        // Each body owns its descendants' complete comment ranges. Build these
        // extents once, rather than walking subtrees on every scope expansion.
        for id in order.into_iter().rev() {
            let Some(parent) = slots[id.index()].role.owner() else {
                continue;
            };
            let mut owned = slots[id.index()].body;
            for comment in &original.nodes[id.index()].comments {
                owned = super::span(
                    owned.start().min(comment.span.start()),
                    owned.end().max(comment.span.end()),
                );
            }
            let body = &mut slots[parent.index()].body;
            *body = super::span(body.start().min(owned.start()), body.end().max(owned.end()));
        }
        Self {
            original,
            candidate,
            layout,
            slots,
        }
    }

    pub(super) fn body(&self, id: NodeId) -> Span {
        self.slots[id.index()].body
    }

    /// Choose a node whose source region can represent the edit at `id`.
    /// Each step moves to a parent. The root is the final fallback.
    pub(super) fn select(&self, mut id: NodeId) -> NodeId {
        loop {
            let slot = self.slots[id.index()];
            if let Some(mapping) = slot.enclosing_key {
                id = mapping;
                continue;
            }
            let Some(owner) = slot.role.owner() else {
                return id;
            };
            if self.accepts(id, slot) {
                return id;
            }
            id = owner;
        }
    }

    fn accepts(&self, id: NodeId, slot: Slot) -> bool {
        let old = &self.original.nodes[id.index()];
        let new = &self.candidate.nodes[id.index()];
        let same_collection = matches!(
            (&old.kind, &new.kind),
            (NodeKind::Sequence { .. }, NodeKind::Sequence { .. })
                | (NodeKind::Mapping { .. }, NodeKind::Mapping { .. })
        );
        if slot.body.start() < old.span.start()
            || (same_collection && slot.outer_comments)
            || (!slot.own_line && self.layout.multiline(id, &new.kind))
        {
            return false;
        }
        if !(same_collection || (inline_node(old) && inline_node(new)))
            || old.comments.len() != new.comments.len()
            || !old
                .comments
                .iter()
                .zip(&new.comments)
                .all(|(old, new)| same_attachment(old, new))
        {
            // Shape changes and changed headers need the collection's
            // indicators and comment attachment context.
            return false;
        }
        match slot.role {
            Role::Key {
                separator,
                length_limited,
                ..
            } => {
                // Block keys and compact flow-pair keys count the separation
                // before ':' toward their 1024-character limit.
                let padding = length_limited.then(|| {
                    let suffix = &self.original.text[old.span.end()..];
                    suffix.len() - suffix.trim_start_matches([' ', '\t']).len()
                });
                let form = emit::key_form(new, slot.flow, padding);
                form != KeyForm::Explicit
                    && match separator {
                        KeySeparator::Missing => !emit::omitted(new),
                        KeySeparator::Separated => true,
                        KeySeparator::Adjacent => form == KeyForm::Quoted,
                    }
            }
            Role::Value {
                header: ValueHeader::Missing,
                ..
            } => false,
            Role::Value {
                header: ValueHeader::IndentlessSequence,
                ..
            } => {
                // Only a block sequence may occupy this mapping-level slot.
                // Empty or flow output needs the mapping to place it after ':'.
                emit::block_collection(&new.kind, slot.flow || self.layout.flow(id))
            }
            _ => true,
        }
    }

    pub(super) fn context(&self, id: NodeId, indent: usize) -> Context {
        let slot = self.slots[id.index()];
        match slot.role {
            Role::Key { length_limited, .. } if length_limited || slot.flow => {
                Context::key(indent, slot.flow)
            }
            Role::Document => Context::value(indent, slot.flow, 0),
            _ => Context::value(indent, slot.flow, indent),
        }
    }
}

fn key_separator(original: &Source, key: NodeId) -> KeySeparator {
    let at =
        astral_yaml::comments::skip_trivia(&original.text, original.nodes[key.index()].span.end());
    let Some(suffix) = original.text[at..].strip_prefix(':') else {
        return KeySeparator::Missing;
    };
    if suffix.starts_with(|c: char| !c.is_ascii_whitespace() && !matches!(c, ',' | ']' | '}')) {
        KeySeparator::Adjacent
    } else {
        KeySeparator::Separated
    }
}

fn value_header(
    original: &Source,
    mapping: NodeId,
    value: NodeId,
    separator: KeySeparator,
    lines: &mut LineIndex<'_>,
) -> ValueHeader {
    let node = &original.nodes[value.index()];
    if node.span.is_empty() && matches!(separator, KeySeparator::Missing) {
        return ValueHeader::Missing;
    }
    let value_start = original.contents[value.index()].start();
    let mapping_start = original.contents[mapping.index()].start();
    if matches!(
        node.kind,
        NodeKind::Sequence {
            style: CollectionStyle::Block,
            ..
        }
    ) && lines.at(value_start).column(value_start)
        == lines.at(mapping_start).column(mapping_start)
    {
        ValueHeader::IndentlessSequence
    } else {
        ValueHeader::Present
    }
}

pub(super) fn same_attachment(old: &Comment, new: &Comment<Option<Span>>) -> bool {
    new.span == Some(old.span) && old.text == new.text && old.placement == new.placement
}

fn inline_node<Location>(node: &Node<Location>) -> bool {
    node.comments
        .iter()
        .all(|comment| comment.position != CommentPosition::Header)
        && matches!(
            node.kind,
            NodeKind::Scalar {
                style: ScalarStyle::Plain | ScalarStyle::SingleQuoted | ScalarStyle::DoubleQuoted,
                ..
            } | NodeKind::Alias { .. }
        )
}
