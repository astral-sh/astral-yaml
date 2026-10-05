use super::{
    Comment, CommentPosition, EditableDocument, NodeKind,
    view::{Emission, Syntax},
};
use crate::{CollectionStyle, Error, NodeId, ScalarStyle, Span, comments::CommentPlacement};
use std::fmt::Write;

#[cfg(test)]
mod tests;

/// A rewritten region includes descendant comments, but leaves the root's outer
/// comments in their original source positions unless explicitly requested.
pub(super) fn region(document: &EditableDocument, id: NodeId, outer: bool) -> Span {
    let root = &document.nodes[id.0];
    let mut start = root.span.start();
    let mut end = root.span.end();
    let mut pending = vec![id];
    while let Some(current) = pending.pop() {
        let node = &document.nodes[current.0];
        if current != id || outer {
            for comment in &node.comments {
                start = start.min(comment.span.start());
                end = end.max(comment.span.end());
            }
        }
        children(&node.kind, &mut pending);
    }
    Span::new(start, end).expect("ordered source ranges")
}

pub(super) fn children(kind: &NodeKind, pending: &mut Vec<NodeId>) {
    match kind {
        NodeKind::Sequence { items, .. } => pending.extend(items),
        NodeKind::Mapping { entries, .. } => {
            for &(key, value) in entries {
                pending.extend([key, value]);
            }
        }
        NodeKind::Scalar { .. } | NodeKind::Alias { .. } => {}
    }
}

pub(super) fn indent(source: &str, at: usize) -> usize {
    source[..at]
        .rsplit('\n')
        .next()
        .unwrap_or("")
        .bytes()
        .take_while(|&byte| byte == b' ')
        .count()
}

pub(super) fn render<'a, Location: Copy + Into<Option<Span>> + 'a>(
    document: impl Into<Emission<'a, Location>>,
    id: NodeId,
    region: Span,
    indent: usize,
    flow: bool,
) -> Result<Rendered, Error> {
    let document = document.into();
    render_with_comments(
        document,
        id,
        region,
        indent,
        flow,
        &document.nodes[id.0].comments,
    )
}

/// Renders a region after the transaction has planned any outer comment patches.
pub(super) fn render_with_comments<'a, Location: Copy + Into<Option<Span>> + 'a>(
    document: impl Into<Emission<'a, Location>>,
    id: NodeId,
    region: Span,
    indent: usize,
    flow: bool,
    root_comments: &'a [Comment<Location>],
) -> Result<Rendered, Error> {
    let document = document.into();
    let mut emitter = Emitter {
        document,
        root: id,
        root_comments,
        region,
        output: String::new(),
        work: document.options.max_nodes,
        pending_indent: None,
        trailing_block_scalar: false,
    };
    let prefix = document.source[..region.start()]
        .rsplit(['\r', '\n'])
        .next()
        .unwrap_or("");
    let own_line = id == document.root
        && (block_collection(&document.nodes[id.0].kind, flow)
            || emitter.visible_comments(id).any(|comment| comment.placement == CommentPlacement::Leading));
    if own_line && !prefix.bytes().all(|byte| byte == b' ') {
        // The unchanged prefix can end in a document marker. Block collections
        // and leading comments must start on their own line, unlike inline roots.
        emitter.line(indent)?;
    } else if region.is_empty()
        && matches!(
            document.source.as_bytes()[..region.start()].last(),
            Some(b':' | b'-' | b'?')
        )
    {
        emitter.push(" ")?;
    }
    let context = Context::value(indent, flow, if id == document.root { 0 } else { indent });
    emitter.node(id, context, "", 0)?;
    if region.is_empty() && document.source.as_bytes().get(region.end()) == Some(&b'#') {
        emitter.push(" ")?;
    }
    emitter.finish()
}

/// Complete replacement text and the source separators it owns.
#[derive(Debug)]
pub(super) struct Rendered {
    pub(super) text: String,
    pub(super) span: Span,
}

#[derive(Clone, Copy)]
struct Context {
    /// Indentation for continuation lines and comments.
    indent: usize,
    position: Position,
}

#[derive(Clone, Copy)]
enum Position {
    // A block scalar's indentation is relative to its parent, even when its
    // header is moved onto a continuation line by properties or comments.
    Block { parent_indent: usize },
    Flow,
    ImplicitKey { flow: bool },
}

impl Context {
    fn value(indent: usize, flow: bool, parent_indent: usize) -> Self {
        Self {
            indent,
            position: if flow {
                Position::Flow
            } else {
                Position::Block { parent_indent }
            },
        }
    }

    fn flow(indent: usize) -> Self {
        Self {
            indent,
            position: Position::Flow,
        }
    }

    /// Block scalar content and suffix comments use the parent's indentation,
    /// even when properties or leading comments move the header farther in.
    fn scalar_indent(self, style: ScalarStyle) -> usize {
        match self.position {
            Position::Block { parent_indent }
                if matches!(style, ScalarStyle::Literal | ScalarStyle::Folded) =>
            {
                parent_indent
            }
            _ => self.indent,
        }
    }

    fn in_flow(self) -> bool {
        matches!(
            self.position,
            Position::Flow | Position::ImplicitKey { flow: true }
        )
    }
}

/// Comments are consumed in source order as the node's syntax is emitted.
/// Passing the cursor through field headers, properties, and content makes
/// interleaved inline/leading comments independent of the number of properties.
struct Comments<Location> {
    remaining: std::vec::IntoIter<Comment<Location>>,
}

impl<Location> Comments<Location> {
    fn at_opening(&self) -> bool {
        self.remaining
            .as_slice()
            .first()
            .is_some_and(|comment| comment.position == CommentPosition::Opening)
    }

    fn next_is(&self, placement: CommentPlacement) -> bool {
        self.remaining
            .as_slice()
            .first()
            .is_some_and(|comment| comment.placement == placement)
    }

    fn take(&mut self, placement: CommentPlacement) -> Option<Comment<Location>> {
        self.next_is(placement)
            .then(|| self.remaining.next().expect("next comment"))
    }

    fn has_prefix(&self) -> bool {
        self.remaining.as_slice().iter().any(|comment| {
            matches!(
                comment.position,
                CommentPosition::Before | CommentPosition::Header
            ) || comment.placement == CommentPlacement::Leading
        })
    }

    fn take_header(&mut self) -> Option<Comment<Location>> {
        self.remaining
            .as_slice()
            .first()
            .is_some_and(|comment| {
                comment.position == CommentPosition::Header
                    && comment.placement != CommentPlacement::Inline
            })
            .then(|| self.remaining.next().expect("header comment"))
    }

    fn needs_value_line(&self, properties: usize) -> bool {
        self.remaining
            .as_slice()
            .first()
            .is_some_and(|comment| comment.position == CommentPosition::Before)
            || self
                .remaining
                .as_slice()
                .iter()
                .any(|comment| comment.placement == CommentPlacement::Leading)
            || self
                .remaining
                .as_slice()
                .iter()
                .filter(|comment| {
                    comment.placement == CommentPlacement::Inline
                        && matches!(
                            comment.position,
                            CommentPosition::Before | CommentPosition::Header
                        )
                })
                .count()
                > properties
    }

    fn needs_header(&self) -> bool {
        self.has_prefix()
            || self
                .remaining
                .as_slice()
                .iter()
                .filter(|comment| comment.placement == CommentPlacement::Inline)
                .count()
                > 1
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScalarPosition {
    Document,
    Block,
    Flow,
    BlockKey,
    FlowKey,
}

/// Erasing a root can remove its entire line or leave a document marker behind.
enum ErasedRootLine {
    Standalone { start: usize },
    Shared,
}

struct Emitter<'a, Location> {
    document: Emission<'a, Location>,
    root: NodeId,
    root_comments: &'a [Comment<Location>],
    region: Span,
    output: String,
    work: usize,
    // A comment terminator is materialized only when following syntax needs it.
    pending_indent: Option<usize>,
    trailing_block_scalar: bool,
}

impl<Location: Copy + Into<Option<Span>>> Emitter<'_, Location> {
    fn error(&self, message: &'static str) -> Error {
        Error {
            span: self.region,
            message,
        }
    }

    fn finish(mut self) -> Result<Rendered, Error> {
        let source = self.document.source;
        let mut span = self.region;
        if self.trailing_block_scalar {
            // Blank lines immediately following a block scalar belong to its
            // content. The emitted value supplies all of them itself.
            span.end = following_line_breaks(source, span.end());
        } else if let Some(line) = self.erased_root_line() {
            // An omitted root has no token to occupy its former line. Leaving
            // that line behind would turn adjacent leading comments into
            // detached comments, or indent a following document marker.
            span.end = following_line_breaks(source, span.end());
            match line {
                ErasedRootLine::Standalone { start } => span.start = start,
                ErasedRootLine::Shared => {
                    // A retained marker still needs its line terminator. The
                    // root's original span may already include that break,
                    // as it does for a block scalar.
                    if source[self.region.start()..span.end()].contains('\n') {
                        self.line(0)?;
                    }
                }
            }
        }
        let suffix = &source[span.end()..];
        let trimmed = suffix.trim_start_matches([' ', '\t']);
        let suffix_has_break = trimmed.starts_with('\n');
        if self.pending_indent.is_some() && suffix_has_break {
            // Share the retained terminator, but not the old line's trailing
            // spaces: they would become part of the emitted comment's text.
            span.end += suffix.len() - trimmed.len();
        }
        let needs_break = self.pending_indent.take().is_some()
            || self
                .document
                .end_marker
                .is_some_and(|marker| marker.start() == span.end())
            || (!span.is_empty() && source[..span.end()].ends_with('\n'));
        if needs_break
            && !self.output.is_empty()
            && !self.output.ends_with('\n')
            && !suffix_has_break
        {
            self.push("\n")?;
        }
        Ok(Rendered {
            text: self.output,
            span,
        })
    }

    fn erased_root_line(&self) -> Option<ErasedRootLine> {
        if self.root != self.document.root
            || self.region.is_empty()
            || !omitted(&self.document.nodes[self.root.0])
        {
            return None;
        }
        let prefix = self.document.source[..self.region.start()]
            .rsplit('\n')
            .next()
            .unwrap_or("")
            .trim_start_matches('\u{feff}');
        if prefix.bytes().all(|byte| matches!(byte, b' ' | b'\t')) {
            Some(ErasedRootLine::Standalone {
                start: self.region.start() - prefix.len(),
            })
        } else {
            Some(ErasedRootLine::Shared)
        }
    }

    fn push(&mut self, text: &str) -> Result<(), Error> {
        if !text.is_empty() {
            if let Some(indent) = self.pending_indent.take() {
                self.line(indent)?;
            }
        }
        if text.len()
            > self
                .document
                .options
                .max_input_bytes
                .saturating_sub(self.output.len())
        {
            return Err(self.error("emitted YAML exceeds input byte limit"));
        }
        if text.bytes().any(|byte| !byte.is_ascii_whitespace()) {
            self.trailing_block_scalar = false;
        }
        self.output.push_str(text);
        Ok(())
    }

    fn line(&mut self, indent: usize) -> Result<(), Error> {
        self.pending_indent = None;
        if let Some(index) = self
            .output
            .rfind('\n')
            .filter(|&index| self.output[index + 1..].bytes().all(|byte| byte == b' '))
        {
            self.output.truncate(index + 1);
        } else {
            self.push("\n")?;
        }
        for _ in 0..indent {
            self.push(" ")?;
        }
        Ok(())
    }

    fn inline(&mut self, comments: &mut Comments<Location>) -> Result<bool, Error> {
        let Some(comment) = comments.take(CommentPlacement::Inline) else {
            return Ok(false);
        };
        self.push(if self.output.ends_with(' ') {
            "#"
        } else {
            " #"
        })?;
        self.push(&comment.text)?;
        Ok(true)
    }

    fn property_comment(
        &mut self,
        comments: &mut Comments<Location>,
        indent: usize,
        collection: Option<CollectionStyle>,
    ) -> Result<(), Error> {
        // Reserve the last inline comment for the content, unless a leading
        // comment still needs a boundary before that content begins.
        // Flow-opening comments belong after the delimiter even when a tag or
        // anchor precedes it. Block presentation can use a property header.
        let opening = collection == Some(CollectionStyle::Flow) && comments.at_opening();
        if !opening
            && (collection == Some(CollectionStyle::Block) || comments.needs_header())
            && self.inline(comments)?
        {
            self.line(indent)?;
            self.before(comments, indent)?;
        }
        while let Some(comment) = comments.take_header() {
            self.line(indent)?;
            self.push("#")?;
            self.push(&comment.text)?;
            self.line(indent)?;
        }
        Ok(())
    }

    fn flow_start(
        &mut self,
        delimiter: &str,
        id: NodeId,
        comments: &mut Comments<Location>,
        indent: usize,
    ) -> Result<(), Error> {
        self.push(delimiter)?;
        // A closing comment may remain outside the rendered region. Its
        // visibility must not determine where the opening comment is emitted.
        // Only the final nested comment can use the closing delimiter's inline
        // slot: later full-line suffixes must stay inside the collection.
        // Document suffixes can remain outside the root's closing delimiter.
        let closing_inline = comments
            .remaining
            .as_slice()
            .last()
            .is_some_and(|comment| comment.placement == CommentPlacement::Inline);
        if (comments.at_opening()
            || comments.needs_header()
            || (id != self.document.root && !closing_inline))
            && self.inline(comments)?
        {
            self.line(indent)?;
            self.before(comments, indent)?;
        }
        Ok(())
    }

    fn visible_comments(&self, id: NodeId) -> impl Iterator<Item = &Comment<Location>> {
        let comments = if id == self.root {
            self.root_comments
        } else {
            &self.document.nodes[id.0].comments
        };
        comments
            .iter()
            .filter(move |comment| {
                id != self.root
                    || comment.span.into().is_none_or(|span| {
                        span.is_empty()
                            || (span.start() >= self.region.start()
                                && span.end() <= self.region.end())
                    })
            })
    }

    fn node_comments(&self, id: NodeId) -> Comments<Location> {
        Comments {
            remaining: self
                .visible_comments(id)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }

    fn before(&mut self, comments: &mut Comments<Location>, indent: usize) -> Result<(), Error> {
        // Leading comments need their own line, including after flow delimiters.
        if comments.next_is(CommentPlacement::Leading) && !self.output.is_empty() {
            self.line(indent)?;
        }
        while let Some(comment) = comments.take(CommentPlacement::Leading) {
            self.push("#")?;
            self.push(&comment.text)?;
            self.line(indent)?;
        }
        Ok(())
    }

    fn prefix(&mut self, comments: &mut Comments<Location>, indent: usize) -> Result<(), Error> {
        self.before(comments, indent)?;
        while comments
            .remaining
            .as_slice()
            .first()
            .is_some_and(|comment| {
                comment.position == CommentPosition::Before
                    && comment.placement == CommentPlacement::Detached
            })
        {
            let comment = comments.remaining.next().expect("detached prefix comment");
            if !self.output.is_empty() {
                self.line(indent)?;
            }
            self.push("#")?;
            self.push(&comment.text)?;
            self.push("\n")?;
            self.push("\n")?;
            self.line(indent)?;
            self.before(comments, indent)?;
        }
        Ok(())
    }

    /// A block collection's property header owns the space before its first
    /// field or dash. Keep the first child's leading comments after that slot's
    /// indicator, where they cannot attach to the enclosing header.
    fn slot_prefix(
        &mut self,
        comments: &mut Comments<Location>,
        collection: NodeId,
        first: bool,
        context: Context,
    ) -> Result<(), Error> {
        let parent = &self.document.nodes[collection.0];
        if first && !context.in_flow() && (parent.anchor.is_some() || parent.tag.is_some()) {
            return Ok(());
        }
        self.before(
            comments,
            context.indent + if context.in_flow() { 2 } else { 0 },
        )
    }

    fn between(
        &mut self,
        comments: &mut Comments<Location>,
        child: NodeId,
        indent: usize,
    ) -> Result<(), Error> {
        while comments
            .remaining
            .as_slice()
            .first()
            .is_some_and(|comment| comment.position == CommentPosition::BeforeChild(child))
        {
            let comment = comments.remaining.next().expect("comment before child");
            self.line(indent)?;
            if !self.trailing_block_scalar {
                self.push("\n")?;
            }
            self.line(indent)?;
            self.push("#")?;
            self.push(&comment.text)?;
            // Blank separation on both sides keeps a container comment from
            // becoming a leading comment on the following child.
            self.push("\n")?;
            self.push("\n")?;
            self.line(indent)?;
        }
        Ok(())
    }

    fn after(
        &mut self,
        comments: Comments<Location>,
        indent: usize,
        suffix: &str,
    ) -> Result<(), Error> {
        self.push(suffix)?;
        for comment in comments.remaining {
            self.comment(&comment, indent)?;
        }
        Ok(())
    }

    fn comment(&mut self, comment: &Comment<Location>, indent: usize) -> Result<(), Error> {
        if comment.placement == CommentPlacement::Inline
            && self.pending_indent.is_none()
            && !self.output.ends_with('\n')
        {
            self.push(" ")?;
        } else {
            self.line(indent)?;
        }
        if comment.placement == CommentPlacement::Detached {
            self.push("\n")?;
            self.line(indent)?;
        }
        self.push("#")?;
        self.push(&comment.text)?;
        self.pending_indent = Some(indent);
        Ok(())
    }

    /// Full-line suffixes need the collection's enclosing delimiters to retain
    /// their owner. An inline closing comment still follows the delimiter.
    fn flow_end(
        &mut self,
        delimiter: &str,
        id: NodeId,
        comments: &mut Comments<Location>,
        indent: usize,
        mut needs_separator: bool,
    ) -> Result<(), Error> {
        while comments
            .remaining
            .as_slice()
            .first()
            .is_some_and(|comment| {
                id != self.document.root
                    || comment.span.into().is_some_and(|span| {
                        !span.is_empty()
                            && self.document.nodes[id.0]
                                .span
                                .into()
                                .is_some_and(|node| span.end() <= node.end())
                    })
            })
        {
            let Some(comment) = comments
                .take(CommentPlacement::Trailing)
                .or_else(|| comments.take(CommentPlacement::Detached))
            else {
                break;
            };
            // Close the last entry before emitting a collection-owned suffix.
            // In particular, an omitted value otherwise starts at the closing
            // delimiter and captures the intervening comments as its prefix.
            if needs_separator {
                self.push(",")?;
                needs_separator = false;
            }
            self.comment(&comment, indent)?;
        }
        self.push(delimiter)
    }

    fn node(
        &mut self,
        id: NodeId,
        context: Context,
        suffix: &str,
        depth: usize,
    ) -> Result<(), Error> {
        let comments = self.node_comments(id);
        self.node_with_comments(id, context, suffix, depth, comments)
    }

    fn node_with_comments(
        &mut self,
        id: NodeId,
        context: Context,
        suffix: &str,
        depth: usize,
        mut comments: Comments<Location>,
    ) -> Result<(), Error> {
        let indent = context.indent;
        let flow = context.in_flow();
        if depth >= self.document.options.max_depth || self.work == 0 {
            return Err(self.error("emission resource limit exceeded"));
        }
        self.work -= 1;
        self.prefix(&mut comments, indent)?;
        self.properties(id, indent, flow, &mut comments)?;
        let node = &self.document.nodes[id.0];
        match &node.kind {
            NodeKind::Scalar { value, style } => {
                self.scalar(value, *style, context, &mut comments)?;
            }
            NodeKind::Alias { target } => {
                let name = self.document.nodes[target.0]
                    .anchor
                    .as_deref()
                    .ok_or_else(|| self.error("alias target has no anchor"))?;
                self.push("*")?;
                self.push(name)?;
            }
            NodeKind::Sequence { items, style } => {
                self.sequence(items, *style, id, context, depth, &mut comments)?;
            }
            NodeKind::Mapping { entries, style } => {
                let is_flow = flow || *style == CollectionStyle::Flow || entries.is_empty();
                if is_flow {
                    self.flow_start("{", id, &mut comments, indent + 2)?;
                } else if node.anchor.is_some() || node.tag.is_some() {
                    self.line(indent)?;
                }
                for (index, &(key, value)) in entries.iter().enumerate() {
                    if index != 0 {
                        if is_flow {
                            self.push(" ")?;
                        } else {
                            self.line(indent)?;
                        }
                    }
                    self.between(
                        &mut comments,
                        key,
                        if is_flow { indent + 2 } else { indent },
                    )?;
                    let mut value_comments = self.node_comments(value);
                    self.slot_prefix(
                        &mut value_comments,
                        id,
                        index == 0,
                        Context::value(indent, is_flow, indent),
                    )?;
                    self.key(key, Context::value(indent, is_flow, indent), depth + 1)?;
                    self.push(":")?;
                    let suffix = if is_flow && index + 1 != entries.len() {
                        ","
                    } else {
                        ""
                    };
                    self.value(
                        value,
                        Context::value(indent, is_flow, indent),
                        suffix,
                        depth + 1,
                        value_comments,
                    )?;
                }
                if is_flow {
                    self.flow_end("}", id, &mut comments, indent + 2, !entries.is_empty())?;
                }
            }
        }
        let suffix_indent = match &node.kind {
            NodeKind::Scalar { style, .. } => context.scalar_indent(*style),
            _ => indent,
        };
        self.after(comments, suffix_indent, suffix)
    }

    fn sequence(
        &mut self,
        items: &[NodeId],
        style: CollectionStyle,
        id: NodeId,
        context: Context,
        depth: usize,
        comments: &mut Comments<Location>,
    ) -> Result<(), Error> {
        let indent = context.indent;
        let is_flow = context.in_flow() || style == CollectionStyle::Flow || items.is_empty();
        if is_flow {
            self.flow_start("[", id, comments, indent + 2)?;
            for (index, &item) in items.iter().enumerate() {
                if index != 0 {
                    self.push(" ")?;
                }
                self.between(comments, item, indent + 2)?;
                self.node(
                    item,
                    Context::flow(indent + 2),
                    if index + 1 == items.len() { "" } else { "," },
                    depth + 1,
                )?;
            }
            self.flow_end("]", id, comments, indent + 2, !items.is_empty())?;
        } else {
            let node = &self.document.nodes[id.0];
            if node.anchor.is_some() || node.tag.is_some() {
                self.line(indent)?;
            }
            for (index, &item) in items.iter().enumerate() {
                if index != 0 {
                    self.line(indent)?;
                }
                self.between(comments, item, indent)?;
                let mut comments = self.node_comments(item);
                self.slot_prefix(&mut comments, id, index == 0, context)?;
                self.push("-")?;
                self.value(item, context, "", depth + 1, comments)?;
            }
        }
        Ok(())
    }

    fn key(&mut self, key: NodeId, context: Context, depth: usize) -> Result<(), Error> {
        let indent = context.indent;
        let is_flow = context.in_flow();
        let simple = simple_key(&self.document.nodes[key.0], is_flow);
        if !simple && !is_flow {
            self.push("?")?;
            let key_comments = self.node_comments(key);
            self.value(key, context, "", depth, key_comments)?;
            self.line(indent)?;
        } else {
            if !simple {
                self.push("? ")?;
            }
            let key_context = if simple {
                Context {
                    indent: indent + 2,
                    position: Position::ImplicitKey { flow: is_flow },
                }
            } else {
                Context::flow(indent + 2)
            };
            self.node(key, key_context, "", depth)?;
            if !simple {
                self.line(indent + 2)?;
            }
        }
        Ok(())
    }

    fn properties(
        &mut self,
        id: NodeId,
        indent: usize,
        flow: bool,
        comments: &mut Comments<Location>,
    ) -> Result<(), Error> {
        let node = &self.document.nodes[id.0];
        let collection = collection_style(&node.kind, flow);
        let continuation_indent = if flow || collection == Some(CollectionStyle::Block) {
            indent
        } else {
            indent + 2
        };
        if let Some(tag) = &node.tag {
            if tag == "!" {
                self.push("! ")?;
            } else {
                self.push("!<")?;
                self.push(tag)?;
                self.push("> ")?;
            }
            self.property_comment(comments, continuation_indent, collection)?;
        }
        if let Some(anchor) = &node.anchor {
            self.push("&")?;
            self.push(anchor)?;
            self.push(" ")?;
            self.property_comment(comments, continuation_indent, collection)?;
        }
        Ok(())
    }

    fn scalar(
        &mut self,
        value: &str,
        style: ScalarStyle,
        context: Context,
        comments: &mut Comments<Location>,
    ) -> Result<(), Error> {
        let indent = context.scalar_indent(style);
        let position = match context.position {
            Position::Flow => ScalarPosition::Flow,
            Position::ImplicitKey { flow: true } => ScalarPosition::FlowKey,
            Position::ImplicitKey { flow: false } => ScalarPosition::BlockKey,
            Position::Block { .. } if self.at_document_boundary() => ScalarPosition::Document,
            Position::Block { .. } => ScalarPosition::Block,
        };
        let text = scalar(value, style, position, indent);
        if text.starts_with(['|', '>']) {
            let (header, body) = text.split_once('\n').expect("block scalar header");
            self.push(header)?;
            self.inline(comments)?;
            self.push("\n")?;
            self.push(body)?;
            self.trailing_block_scalar = true;
            return Ok(());
        }
        self.push(&text)
    }

    fn at_document_boundary(&self) -> bool {
        // Document markers are recognized only at column zero. Include the
        // retained source prefix when emission has not started a new line.
        if let Some(index) = self.output.rfind('\n') {
            return self.output[index + 1..].is_empty();
        }
        let prefix = self.document.source[..self.region.start()]
            .rsplit('\n')
            .next()
            .unwrap_or("");
        prefix.trim_start_matches('\u{feff}').is_empty() && self.output.is_empty()
    }

    fn value(
        &mut self,
        id: NodeId,
        context: Context,
        suffix: &str,
        depth: usize,
        mut comments: Comments<Location>,
    ) -> Result<(), Error> {
        let indent = context.indent;
        let flow = context.in_flow();
        let node = &self.document.nodes[id.0];
        let block = block_collection(&node.kind, flow);
        let properties = usize::from(node.tag.is_some()) + usize::from(node.anchor.is_some());
        let leading = comments.needs_value_line(properties);
        let mut value_indent = if block || flow || leading {
            indent + 2
        } else {
            indent
        };
        if block || leading {
            self.inline(&mut comments)?;
            self.line(value_indent)?;
        } else {
            self.push(" ")?;
        }
        if block && properties == 0 && comments.next_is(CommentPlacement::Leading) {
            // With no delimiter or property header, indentation is the opening
            // boundary. Put the collection's prefix outside its child slots.
            self.before(&mut comments, value_indent)?;
            value_indent += 2;
            self.line(value_indent)?;
        }
        let context = Context::value(value_indent, flow, indent);
        self.node_with_comments(id, context, suffix, depth, comments)
    }
}

/// An empty plain scalar without properties has no syntax token of its own.
pub(super) fn omitted<Location>(node: &super::Node<Location>) -> bool {
    node.anchor.is_none()
        && node.tag.is_none()
        && matches!(&node.kind, NodeKind::Scalar { value, style: ScalarStyle::Plain } if value.is_empty())
}

/// Consume complete whitespace-only lines, retaining the indentation of any
/// following syntax. Both a block scalar and an erased root own these breaks.
fn following_line_breaks(source: &str, start: usize) -> usize {
    let suffix = &source[start..];
    match suffix.bytes().position(|byte| !byte.is_ascii_whitespace()) {
        Some(index) => suffix[..index]
            .rfind('\n')
            .map_or(start, |offset| start + offset + 1),
        None => source.len(),
    }
}

pub(super) fn simple_key(node: &super::Node, flow: bool) -> bool {
    let NodeKind::Scalar { value, style } = &node.kind else {
        return false;
    };
    if !node.comments.is_empty() || matches!(style, ScalarStyle::Literal | ScalarStyle::Folded)
    {
        return false;
    }
    // YAML limits the entire implicit key, including escapes and properties,
    // to 1024 source characters on one line.
    let position = if flow {
        ScalarPosition::Flow
    } else {
        ScalarPosition::Block
    };
    let text = scalar(value, *style, position, 0);
    let tag = node.tag.as_ref().map_or(0, |tag| tag.chars().count() + 4);
    let anchor = node
        .anchor
        .as_ref()
        .map_or(0, |anchor| anchor.chars().count() + 2);
    !text.contains(['\r', '\n']) && text.chars().count() + tag + anchor <= 1024
}

fn block_collection(kind: &NodeKind, flow: bool) -> bool {
    collection_style(kind, flow) == Some(CollectionStyle::Block)
}

fn collection_style(kind: &NodeKind, flow: bool) -> Option<CollectionStyle> {
    let (empty, style) = match kind {
        NodeKind::Sequence { items, style } => (items.is_empty(), *style),
        NodeKind::Mapping { entries, style } => (entries.is_empty(), *style),
        _ => return None,
    };
    Some(if flow || empty {
        CollectionStyle::Flow
    } else {
        style
    })
}

fn quoted(value: &str) -> String {
    let mut result = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() || matches!(c, '\u{85}' | '\u{2028}' | '\u{2029}') => {
                write!(result, "\\U{:08X}", u32::from(c)).expect("writing a String");
            }
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

fn flow_scalar(value: &str, indent: usize) -> String {
    let continuation = " ".repeat(indent + 2);
    let mut result = String::new();
    let mut after_break = false;
    for character in value.chars() {
        if character == '\n' {
            // Flow folding consumes one physical break in each run. An extra
            // empty line retains every decoded break, including at the edges
            // of a single-quoted scalar.
            if !after_break {
                result.push('\n');
            }
            result.push('\n');
            after_break = true;
        } else {
            if after_break {
                result.push_str(&continuation);
            }
            result.push(character);
            after_break = false;
        }
    }
    if after_break {
        result.push_str(&continuation);
    }
    result
}

fn block_scalar(value: &str, style: ScalarStyle, indent: usize) -> String {
    let indicator = if style == ScalarStyle::Literal {
        '|'
    } else {
        '>'
    };
    let trailing = value
        .bytes()
        .rev()
        .take_while(|&byte| byte == b'\n')
        .count();
    let chomp = match trailing {
        0 => "-",
        // Clip retains the final break of a nonempty content line. A scalar
        // consisting only of empty lines needs keep even for a single break.
        1 if value.len() > trailing => "",
        _ => "+",
    };
    let mut body = value.to_owned();
    if style == ScalarStyle::Folded {
        // Only breaks between ordinary text lines fold in block scalars.
        // Leading, trailing, and more-indented lines already retain breaks.
        body.clear();
        let mut previous_text = false;
        for (index, line) in value.split('\n').enumerate() {
            if index != 0 {
                body.push('\n');
            }
            if !line.is_empty() {
                if previous_text && !line.starts_with([' ', '\t']) {
                    body.push('\n');
                }
                previous_text = !line.starts_with([' ', '\t']);
            }
            body.push_str(line);
        }
    }
    let mut result = format!("{indicator}2{chomp}\n");
    let indentation = " ".repeat(indent + 2);
    for line in body.split_terminator('\n') {
        result.push_str(&indentation);
        result.push_str(line);
        result.push('\n');
    }
    result
}

fn scalar(value: &str, style: ScalarStyle, position: ScalarPosition, indent: usize) -> String {
    // Check the generated scalar in its actual flow/block context. This also
    // handles plain-scalar punctuation, folding, and explicit indentation.
    let candidate = match style {
        // Empty scalar text is observable through Serde string visitors and
        // application tags, even when the core schema resolves it as null.
        ScalarStyle::Plain if value.is_empty() => return String::new(),
        ScalarStyle::Plain => flow_scalar(value, indent),
        ScalarStyle::SingleQuoted => {
            format!("'{}'", flow_scalar(value, indent).replace('\'', "''"))
        }
        ScalarStyle::Literal | ScalarStyle::Folded
            if matches!(position, ScalarPosition::Block | ScalarPosition::Document) =>
        {
            block_scalar(value, style, indent)
        }
        ScalarStyle::DoubleQuoted | ScalarStyle::Literal | ScalarStyle::Folded => {
            return quoted(value);
        }
    };
    let source = match position {
        ScalarPosition::Document => candidate.clone(),
        ScalarPosition::Flow => format!("[{candidate}]"),
        ScalarPosition::BlockKey => format!("{candidate}: value"),
        ScalarPosition::FlowKey => format!("{{{candidate}: value}}"),
        // The sequence supplies the parent indentation used by block scalars.
        ScalarPosition::Block => format!("{}- {candidate}", " ".repeat(indent)),
    };
    let matches = crate::parse(&source).is_ok_and(|stream| {
        let [document] = stream.documents() else {
            return false;
        };
        let mut node = stream.node(document.root).expect("parsed root");
        match position {
            ScalarPosition::Document => {}
            ScalarPosition::Flow | ScalarPosition::Block => {
                let crate::NodeKind::Sequence { items, .. } = &node.kind else {
                    return false;
                };
                let [id] = items.as_slice() else {
                    return false;
                };
                node = stream.node(*id).expect("parsed item");
            }
            ScalarPosition::BlockKey | ScalarPosition::FlowKey => {
                let crate::NodeKind::Mapping { entries, .. } = &node.kind else {
                    return false;
                };
                let [entry] = entries.as_slice() else {
                    return false;
                };
                node = stream.node(entry.key).expect("parsed key");
            }
        }
        matches!(&node.kind, crate::NodeKind::Scalar(scalar)
            if scalar.value == value)
            && node.tag.is_none()
            && node.anchor.is_none()
            && stream.comments().is_empty()
    });
    if matches { candidate } else { quoted(value) }
}

/// Match syntax by traversal, not arena offsets, and verify alias identity after
/// matching the definitions. Aliases are never expanded by this comparison.
pub(super) fn correspondence<'a, Location: 'a>(
    expected: impl Into<Syntax<'a, Location>>,
    actual: &EditableDocument,
) -> Option<Vec<Option<NodeId>>> {
    let expected = expected.into();
    let mut pending = vec![(expected.root, actual.root)];
    let mut correspondence = vec![None; expected.nodes.len()];
    let mut aliases = Vec::new();
    while let Some((left_id, right_id)) = pending.pop() {
        correspondence[left_id.0] = Some(right_id);
        let left = &expected.nodes[left_id.0];
        let right = &actual.nodes[right_id.0];
        if left.anchor != right.anchor || left.tag != right.tag {
            return None;
        }
        match (&left.kind, &right.kind) {
            (
                NodeKind::Scalar {
                    value: a,
                    style: left_style,
                },
                NodeKind::Scalar {
                    value: b,
                    style: right_style,
                },
            ) => {
                let kind = meaning(left, *left_style);
                if kind != meaning(right, *right_style) || a != b {
                    return None;
                }
            }
            (NodeKind::Sequence { items: a, .. }, NodeKind::Sequence { items: b, .. }) => {
                if a.len() != b.len() {
                    return None;
                }
                pending.extend(a.iter().copied().zip(b.iter().copied()));
            }
            (NodeKind::Mapping { entries: a, .. }, NodeKind::Mapping { entries: b, .. }) => {
                if a.len() != b.len() {
                    return None;
                }
                for (&(ak, av), &(bk, bv)) in a.iter().zip(b) {
                    pending.extend([(ak, bk), (av, bv)]);
                }
            }
            (NodeKind::Alias { target: a, .. }, NodeKind::Alias { target: b, .. }) => {
                aliases.push((*a, *b));
            }
            _ => return None,
        }
    }
    aliases
        .into_iter()
        .all(|(a, b)| correspondence[a.0] == Some(b))
        .then_some(correspondence)
}

pub(super) fn comments_preserved<'a, Location: 'a>(
    expected: impl Into<Syntax<'a, Location>>,
    actual: &EditableDocument,
    correspondence: &[Option<NodeId>],
) -> bool {
    let expected = expected.into();
    correspondence.iter().enumerate().all(|(id, target)| {
        let Some(target) = target else {
            return true;
        };
        let left_node = &expected.nodes[id];
        let right_node = &actual.nodes[target.0];
        let left = &left_node.comments;
        let right = &right_node.comments;
        left.len() == right.len()
            && left.iter().zip(right).all(|(left, right)| {
                if left.text != right.text || left.placement != right.placement {
                    return false;
                }
                if left.placement != CommentPlacement::Detached {
                    return true;
                }
                let left = detached_position(&left_node.kind, left.position);
                let right = detached_position(&right_node.kind, right.position);
                match (left, right) {
                    (CommentPosition::BeforeChild(left), CommentPosition::BeforeChild(right)) => {
                        correspondence[left.0] == Some(right)
                    }
                    _ => left == right,
                }
            })
    })
}

/// A detached prefix and the gap before the first child describe the same
/// boundary, even when collection delimiters or properties move around it.
fn detached_position(kind: &NodeKind, position: CommentPosition) -> CommentPosition {
    if !matches!(position, CommentPosition::Before | CommentPosition::Header) {
        return position;
    }
    let first = match kind {
        NodeKind::Sequence { items, .. } => items.first().copied(),
        NodeKind::Mapping { entries, .. } => entries.first().map(|&(key, _)| key),
        _ => None,
    };
    first.map_or(CommentPosition::Before, CommentPosition::BeforeChild)
}

pub(super) fn meaning<Location>(
    node: &super::Node<Location>,
    style: ScalarStyle,
) -> crate::schema::ScalarType {
    use crate::schema::{ScalarType, classify};
    match node.tag.as_deref() {
        Some("!" | "tag:yaml.org,2002:str") => return ScalarType::String,
        Some("tag:yaml.org,2002:null") => return ScalarType::Null,
        Some("tag:yaml.org,2002:bool") => return ScalarType::Bool,
        Some("tag:yaml.org,2002:int") => return ScalarType::Int,
        Some("tag:yaml.org,2002:float") => return ScalarType::Float,
        _ => {}
    }
    match &node.kind {
        NodeKind::Scalar { value, .. } if style == ScalarStyle::Plain => classify(value),
        _ => ScalarType::String,
    }
}
