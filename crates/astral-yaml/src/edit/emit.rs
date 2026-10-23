use super::{Comment, CommentPosition, EditableDocument, NodeKind, layout::Layout, view::Syntax};
use crate::{
    CollectionStyle, Error, NodeId, ParseOptions, ScalarStyle, Span, comments::CommentPlacement,
};
use std::fmt::Write;

#[cfg(test)]
mod tests;

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

/// Render `id` using the supplied placement.
/// Source ranges are selected by the splice planner before this function runs.
pub(super) fn render<'a, Location: Copy + Into<Option<Span>> + 'a>(
    document: Syntax<'a, Location>,
    options: ParseOptions,
    id: NodeId,
    placement: &super::splice::Placement<'a, Location>,
    layout: &Layout,
) -> Result<Fragment, Error> {
    let mut emitter = Emitter {
        document,
        options,
        layout,
        root: id,
        root_comments: placement.comments,
        region: placement.diagnostic_span,
        at_column_zero: placement.at_column_zero,
        output: String::new(),
        work: options.max_nodes,
        ending: EmitterEnding::Token,
    };
    match placement.before {
        super::splice::Separator::None => {}
        super::splice::Separator::Space => emitter.push(" ")?,
        super::splice::Separator::Line(indent) => emitter.line(indent)?,
    }
    emitter.node(id, placement.context, "", 0)?;
    if placement.comment_follows {
        emitter.push(" ")?;
    }
    Ok(Fragment {
        text: emitter.output,
        ending: match emitter.ending {
            EmitterEnding::Token => Ending::Token,
            EmitterEnding::Comment { .. } => Ending::Comment,
            EmitterEnding::BlockScalar => Ending::BlockScalar,
        },
    })
}

/// How a fragment ends, used when joining it to retained source.
pub(super) enum Ending {
    Token,
    Comment,
    BlockScalar,
}

pub(super) struct Fragment {
    pub(super) text: String,
    pub(super) ending: Ending,
}

#[derive(Clone, Copy)]
pub(super) struct Context {
    /// Indentation for continuation lines and comments.
    pub(super) indent: usize,
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
    pub(super) fn value(indent: usize, flow: bool, parent_indent: usize) -> Self {
        Self {
            indent,
            position: if flow {
                Position::Flow
            } else {
                Position::Block { parent_indent }
            },
        }
    }

    pub(super) fn key(indent: usize, flow: bool) -> Self {
        Self {
            indent,
            position: Position::ImplicitKey { flow },
        }
    }

    fn flow(indent: usize) -> Self {
        Self {
            indent,
            position: Position::Flow,
        }
    }

    /// Return the scalar's content indentation.
    /// Block content stays relative to the parent even when properties or comments
    /// push its header farther in.
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

    pub(super) fn in_flow(self) -> bool {
        matches!(
            self.position,
            Position::Flow | Position::ImplicitKey { flow: true }
        )
    }
}

/// A cursor over the node's comments.
/// Pass it through the node's indicators, properties, and content to keep
/// interleaved leading and inline comments in order.
struct Comments<'a, Location> {
    remaining: &'a [Comment<Location>],
}

impl<'a, Location> Comments<'a, Location> {
    fn at_opening(&self) -> bool {
        self.remaining
            .first()
            .is_some_and(|comment| comment.position == CommentPosition::Opening)
    }

    fn next_is(&self, placement: CommentPlacement) -> bool {
        self.remaining
            .first()
            .is_some_and(|comment| comment.placement == placement)
    }

    fn take(&mut self, placement: CommentPlacement) -> Option<&'a Comment<Location>> {
        self.take_if(|comment| comment.placement == placement)
    }

    fn take_if(
        &mut self,
        matches: impl FnOnce(&Comment<Location>) -> bool,
    ) -> Option<&'a Comment<Location>> {
        let (next, rest) = self.remaining.split_first()?;
        if matches(next) {
            self.remaining = rest;
            Some(next)
        } else {
            None
        }
    }

    fn has_prefix(&self) -> bool {
        self.remaining.iter().any(|comment| {
            matches!(
                comment.position,
                CommentPosition::Before | CommentPosition::Header
            ) || comment.placement == CommentPlacement::Leading
        })
    }

    fn take_header(&mut self) -> Option<&'a Comment<Location>> {
        self.take_if(|comment| {
            comment.position == CommentPosition::Header
                && comment.placement != CommentPlacement::Inline
        })
    }

    fn take_detached_prefix(&mut self) -> Option<&'a Comment<Location>> {
        self.take_if(|comment| {
            comment.position == CommentPosition::Before
                && comment.placement == CommentPlacement::Detached
        })
    }

    fn needs_value_line(&self, properties: usize) -> bool {
        self.remaining
            .first()
            .is_some_and(|comment| comment.position == CommentPosition::Before)
            || self
                .remaining
                .iter()
                .any(|comment| comment.placement == CommentPlacement::Leading)
            || self
                .remaining
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
                .iter()
                .filter(|comment| comment.placement == CommentPlacement::Inline)
                .count()
                > 1
    }
}

/// Return whether a block collection has enough headers for its inline comments.
/// Comments can use the enclosing indicator, tag, and anchor. Any remaining
/// inline comment requires a flow delimiter.
pub(super) fn block_headers_cover_comments<Location>(
    node: &super::Node<Location>,
    prefix_before_slot: bool,
) -> bool {
    let mut comments = Comments {
        remaining: &node.comments,
    };
    if prefix_before_slot {
        while comments.take(CommentPlacement::Leading).is_some() {}
    }
    comments.take(CommentPlacement::Inline);
    while comments
        .take(CommentPlacement::Leading)
        .or_else(|| comments.take_detached_prefix())
        .is_some()
    {}
    for _ in node.tag.iter().chain(node.anchor.iter()) {
        if comments.take(CommentPlacement::Inline).is_some() {
            while comments.take(CommentPlacement::Leading).is_some() {}
        }
        while comments.take_header().is_some() {}
    }
    !comments
        .remaining
        .iter()
        .any(|comment| comment.placement == CommentPlacement::Inline)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScalarPosition {
    Document,
    Block,
    Flow,
    BlockKey,
    FlowKey,
}

enum EmitterEnding {
    Token,
    // A comment terminator is materialized only when following syntax needs it.
    Comment { indent: usize },
    BlockScalar,
}

struct Emitter<'a, 'layout, Location> {
    document: Syntax<'a, Location>,
    options: ParseOptions,
    layout: &'layout Layout,
    root: NodeId,
    root_comments: &'a [Comment<Location>],
    region: Span,
    at_column_zero: bool,
    output: String,
    work: usize,
    ending: EmitterEnding,
}

impl<'a, Location: Copy + Into<Option<Span>>> Emitter<'a, '_, Location> {
    fn error(&self, message: &'static str) -> Error {
        Error {
            span: self.region,
            message,
        }
    }

    fn push(&mut self, text: &str) -> Result<(), Error> {
        if !text.is_empty() {
            if let EmitterEnding::Comment { indent } = self.ending {
                self.line(indent)?;
            }
        }
        if text.len()
            > self
                .options
                .max_input_bytes
                .saturating_sub(self.output.len())
        {
            return Err(self.error("emitted YAML exceeds input byte limit"));
        }
        if text.bytes().any(|byte| !byte.is_ascii_whitespace()) {
            self.ending = EmitterEnding::Token;
        }
        self.output.push_str(text);
        Ok(())
    }

    fn line(&mut self, indent: usize) -> Result<(), Error> {
        if matches!(self.ending, EmitterEnding::Comment { .. }) {
            self.ending = EmitterEnding::Token;
        }
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

    fn inline(&mut self, comments: &mut Comments<'_, Location>) -> Result<bool, Error> {
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

    fn comment_text(&mut self, text: &str) -> Result<(), Error> {
        self.push("#")?;
        self.push(text)
    }

    fn property_comment(
        &mut self,
        comments: &mut Comments<'_, Location>,
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
            self.comment_text(&comment.text)?;
            self.line(indent)?;
        }
        Ok(())
    }

    fn flow_start(
        &mut self,
        delimiter: &str,
        id: NodeId,
        comments: &mut Comments<'_, Location>,
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

    fn node_comments(&self, id: NodeId) -> Comments<'a, Location> {
        let comments = if id == self.root {
            self.root_comments
        } else {
            &self.document.nodes[id.0].comments
        };
        Comments {
            remaining: comments,
        }
    }

    fn before(
        &mut self,
        comments: &mut Comments<'_, Location>,
        indent: usize,
    ) -> Result<(), Error> {
        // Leading comments need their own line, including after flow delimiters.
        if comments.next_is(CommentPlacement::Leading) && !self.output.is_empty() {
            self.line(indent)?;
        }
        while let Some(comment) = comments.take(CommentPlacement::Leading) {
            self.comment_text(&comment.text)?;
            self.line(indent)?;
        }
        Ok(())
    }

    fn prefix(
        &mut self,
        comments: &mut Comments<'_, Location>,
        indent: usize,
    ) -> Result<(), Error> {
        self.before(comments, indent)?;
        while let Some(comment) = comments.take_detached_prefix() {
            if !self.output.is_empty() {
                self.line(indent)?;
            }
            self.comment_text(&comment.text)?;
            self.push("\n\n")?;
            self.line(indent)?;
            self.before(comments, indent)?;
        }
        Ok(())
    }

    /// Keep the first child's leading comments after its field or item indicator.
    /// Otherwise they can attach to the enclosing collection's property header.
    fn slot_prefix(
        &mut self,
        comments: &mut Comments<'_, Location>,
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
        comments: &mut Comments<'_, Location>,
        child: NodeId,
        indent: usize,
    ) -> Result<(), Error> {
        while let Some(comment) =
            comments.take_if(|comment| comment.position == CommentPosition::BeforeChild(child))
        {
            self.line(indent)?;
            if !matches!(self.ending, EmitterEnding::BlockScalar) {
                self.push("\n")?;
            }
            self.line(indent)?;
            self.comment_text(&comment.text)?;
            // Blank separation on both sides keeps a container comment from
            // becoming a leading comment on the following child.
            self.push("\n\n")?;
            self.line(indent)?;
        }
        Ok(())
    }

    fn after(
        &mut self,
        comments: &[Comment<Location>],
        indent: usize,
        suffix: &str,
    ) -> Result<(), Error> {
        self.push(suffix)?;
        for comment in comments {
            self.comment(comment, indent)?;
        }
        Ok(())
    }

    fn comment(&mut self, comment: &Comment<Location>, indent: usize) -> Result<(), Error> {
        if comment.placement == CommentPlacement::Inline
            && !matches!(self.ending, EmitterEnding::Comment { .. })
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
        self.comment_text(&comment.text)?;
        self.ending = EmitterEnding::Comment { indent };
        Ok(())
    }

    /// Close a flow collection, keeping full-line suffix comments inside its delimiters.
    /// An inline comment on the closing delimiter stays outside.
    fn flow_end(
        &mut self,
        delimiter: &str,
        id: NodeId,
        comments: &mut Comments<'_, Location>,
        indent: usize,
        mut needs_separator: bool,
    ) -> Result<(), Error> {
        let node_span = self.document.nodes[id.0].span.into();
        while let Some(comment) = comments.take_if(|comment| {
            let full_line = matches!(
                comment.placement,
                CommentPlacement::Trailing | CommentPlacement::Detached
            );
            let inside = comment
                .span
                .into()
                .zip(node_span)
                .is_some_and(|(comment, node)| comment.end() <= node.end());
            full_line && (id != self.document.root || inside)
        }) {
            // Close the last entry before emitting a collection-owned suffix.
            // In particular, an omitted value otherwise starts at the closing
            // delimiter and captures the intervening comments as its prefix.
            if needs_separator {
                self.push(",")?;
                needs_separator = false;
            }
            self.comment(comment, indent)?;
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
        mut comments: Comments<'_, Location>,
    ) -> Result<(), Error> {
        let context = if self.layout.flow(id) {
            Context::flow(context.indent)
        } else {
            context
        };
        let indent = context.indent;
        let flow = context.in_flow();
        if depth >= self.options.max_depth || self.work == 0 {
            return Err(self.error("emission resource limit exceeded"));
        }
        self.work -= 1;
        self.prefix(&mut comments, indent)?;
        self.properties(id, indent, flow, &mut comments)?;
        let node = &self.document.nodes[id.0];
        match &node.kind {
            NodeKind::Scalar { value, style } => {
                self.scalar(
                    value,
                    self.layout.scalar_style(id, *style),
                    id == self.document.root,
                    context,
                    &mut comments,
                )?;
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
        self.after(comments.remaining, suffix_indent, suffix)
    }

    fn sequence(
        &mut self,
        items: &[NodeId],
        style: CollectionStyle,
        id: NodeId,
        context: Context,
        depth: usize,
        comments: &mut Comments<'_, Location>,
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
        let simple = key_form(
            &self.document.nodes[key.0],
            is_flow,
            (!is_flow).then_some(0),
        ) != KeyForm::Explicit;
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
                Context::key(indent + 2, is_flow)
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
        comments: &mut Comments<'_, Location>,
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
        document_root: bool,
        context: Context,
        comments: &mut Comments<'_, Location>,
    ) -> Result<(), Error> {
        let indent = context.scalar_indent(style);
        let position = match context.position {
            Position::Flow => ScalarPosition::Flow,
            Position::ImplicitKey { flow: true } => ScalarPosition::FlowKey,
            Position::ImplicitKey { flow: false } => ScalarPosition::BlockKey,
            Position::Block { .. }
                if matches!(style, ScalarStyle::Literal | ScalarStyle::Folded) =>
            {
                if document_root {
                    ScalarPosition::Document
                } else {
                    ScalarPosition::Block
                }
            }
            Position::Block { .. } if self.at_document_boundary() => ScalarPosition::Document,
            Position::Block { .. } => ScalarPosition::Block,
        };
        let remaining = self
            .options
            .max_input_bytes
            .saturating_sub(self.output.len());
        let text = scalar(value, style, position, indent, remaining)
            .ok_or_else(|| self.error("emitted YAML exceeds input byte limit"))?;
        if text.starts_with(['|', '>']) {
            let (header, body) = text.split_once('\n').expect("block scalar header");
            self.push(header)?;
            self.inline(comments)?;
            self.push("\n")?;
            self.push(body)?;
            self.ending = EmitterEnding::BlockScalar;
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
        self.at_column_zero && self.output.is_empty()
    }

    fn value(
        &mut self,
        id: NodeId,
        context: Context,
        suffix: &str,
        depth: usize,
        mut comments: Comments<'_, Location>,
    ) -> Result<(), Error> {
        let indent = context.indent;
        let flow = context.in_flow();
        let node = &self.document.nodes[id.0];
        let block = block_collection(&node.kind, flow || self.layout.flow(id));
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

/// Return whether the node is an empty plain scalar with no tag or anchor.
pub(super) fn omitted<Location>(node: &super::Node<Location>) -> bool {
    node.anchor.is_none()
        && node.tag.is_none()
        && matches!(&node.kind, NodeKind::Scalar { value, style: ScalarStyle::Plain } if value.is_empty())
}

/// The notation used to render a mapping key.
/// A quoted key allows the value to follow `:` without separation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum KeyForm {
    Explicit,
    Implicit,
    Quoted,
}

pub(super) fn key_form<Location>(
    node: &super::Node<Location>,
    flow: bool,
    implicit_key_padding: Option<usize>,
) -> KeyForm {
    let NodeKind::Scalar { value, style } = &node.kind else {
        return KeyForm::Explicit;
    };
    if !node.comments.is_empty() || matches!(style, ScalarStyle::Literal | ScalarStyle::Folded) {
        return KeyForm::Explicit;
    }
    // Determine whether the requested spelling needs explicit notation before
    // imposing implicit-key restrictions. Multiline styles can survive in an
    // explicit key. The implicit limit includes properties and escapes.
    let position = if flow {
        ScalarPosition::Flow
    } else {
        ScalarPosition::Block
    };
    let text = scalar(value, *style, position, 0, usize::MAX).expect("unlimited scalar budget");
    if text.contains('\n') {
        return KeyForm::Explicit;
    }
    if let Some(padding) = implicit_key_padding {
        let tag = match node.tag.as_deref() {
            None => 0,
            Some("!") => 2,
            Some(tag) => tag.chars().count() + 4,
        };
        let anchor = node
            .anchor
            .as_ref()
            .map_or(0, |anchor| anchor.chars().count() + 2);
        if text.chars().count() + tag + anchor + padding > 1024 {
            return KeyForm::Explicit;
        }
    }
    if text.starts_with(['\'', '"']) {
        KeyForm::Quoted
    } else {
        KeyForm::Implicit
    }
}

/// Return whether a block collection or scalar provides an inline-comment header.
/// Collections need a tag or anchor. Block scalars use their indicator.
pub(super) fn has_inline_header<Location>(node: &super::Node<Location>, flow: bool) -> bool {
    !flow
        && (matches!(
            node.kind,
            NodeKind::Scalar {
                style: ScalarStyle::Literal | ScalarStyle::Folded,
                ..
            }
        ) || (block_collection(&node.kind, flow)
            && (node.tag.is_some() || node.anchor.is_some())))
}

pub(super) fn block_collection(kind: &NodeKind, flow: bool) -> bool {
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

pub(super) fn quoted(value: &str) -> String {
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

/// Indentation can multiply the size of a small multiline value. Check the
/// remaining output budget before appending it to the candidate spelling.
fn indent_scalar(output: &mut String, spaces: usize, limit: usize) -> Option<()> {
    if spaces > limit.saturating_sub(output.len()) {
        return None;
    }
    output.extend(std::iter::repeat_n(' ', spaces));
    Some(())
}

fn flow_scalar(value: &str, indent: usize, limit: usize) -> Option<String> {
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
                indent_scalar(&mut result, indent + 2, limit)?;
            }
            result.push(character);
            after_break = false;
        }
    }
    if after_break {
        indent_scalar(&mut result, indent + 2, limit)?;
    }
    Some(result)
}

fn block_scalar(
    value: &str,
    style: ScalarStyle,
    position: ScalarPosition,
    indent: usize,
    limit: usize,
) -> Option<String> {
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
    // Content is indented two spaces. At the document root its parent is at -1.
    let increment = if position == ScalarPosition::Document {
        3
    } else {
        2
    };
    let mut result = format!("{indicator}{increment}{chomp}\n");
    let mut previous_text = false;
    for line in value.split_terminator('\n') {
        if style == ScalarStyle::Folded && !line.is_empty() {
            // Only breaks between ordinary text lines fold. An extra empty
            // line preserves the decoded break between them.
            let text = !line.starts_with([' ', '\t']);
            if previous_text && text {
                indent_scalar(&mut result, indent + 2, limit)?;
                result.push('\n');
            }
            previous_text = text;
        }
        indent_scalar(&mut result, indent + 2, limit)?;
        result.push_str(line);
        result.push('\n');
    }
    Some(result)
}

fn scalar(
    value: &str,
    style: ScalarStyle,
    position: ScalarPosition,
    indent: usize,
    limit: usize,
) -> Option<String> {
    // Check the generated scalar in its actual flow/block context. This also
    // handles plain-scalar punctuation, folding, and explicit indentation.
    let candidate = match style {
        // Empty scalar text is observable through Serde string visitors and
        // application tags, even when the core schema resolves it as null.
        ScalarStyle::Plain if value.is_empty() => return Some(String::new()),
        ScalarStyle::Plain => flow_scalar(value, indent, limit)?,
        ScalarStyle::SingleQuoted => {
            format!(
                "'{}'",
                flow_scalar(value, indent, limit)?.replace('\'', "''")
            )
        }
        ScalarStyle::Literal | ScalarStyle::Folded
            if matches!(position, ScalarPosition::Block | ScalarPosition::Document) =>
        {
            block_scalar(value, style, position, indent, limit)?
        }
        ScalarStyle::DoubleQuoted | ScalarStyle::Literal | ScalarStyle::Folded => {
            return Some(quoted(value));
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
    Some(if matches { candidate } else { quoted(value) })
}

/// Reparse rendered YAML and require the intended content, alias targets, styles, and comments.
pub(super) fn parse_rendered<Location: Copy>(
    expected: Syntax<'_, Location>,
    output: &str,
    options: ParseOptions,
    span: Span,
    requested_styles: &[NodeId],
) -> Result<EditableDocument, Error> {
    let error = |message| Error { span, message };
    let parsed = EditableDocument::parse_with_options(output, options)?;
    let matched = correspondence(expected, &parsed)
        .ok_or_else(|| error("emission changed YAML content or alias identity"))?;

    for &id in requested_styles {
        let kind = &expected.nodes[id.0].kind;
        let preserved = matched[id.0].is_some_and(|target| {
            super::draft::same_style(kind, &parsed.source.nodes[target.0].kind)
        });
        if !preserved {
            return Err(error(
                "requested style is not representable in this context",
            ));
        }
    }
    if !comments_preserved(expected, &parsed, &matched) {
        return Err(error("emission changed comment ownership or placement"));
    }
    Ok(parsed)
}

/// Match nodes by traversal order and check that aliases reach matching targets.
/// Node IDs can differ between trees. Alias targets are compared without expansion.
pub(super) fn correspondence<'a, Location: 'a>(
    expected: impl Into<Syntax<'a, Location>>,
    actual: &EditableDocument,
) -> Option<Vec<Option<NodeId>>> {
    let expected = expected.into();
    let actual = &actual.source;
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
                if a != b || !same_scalar_type(a, left.tag.as_deref(), *left_style, *right_style) {
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
    let actual = &actual.source;
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

/// Treat a detached prefix and a gap before the first child as the same position,
/// even if properties or delimiters have moved around the comment.
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

/// Only a change to or from plain style can affect implicit scalar resolution.
pub(super) fn same_scalar_type(
    value: &str,
    tag: Option<&str>,
    before: ScalarStyle,
    after: ScalarStyle,
) -> bool {
    use crate::schema::{ScalarType, classify};
    use ScalarStyle::Plain;
    if before == after || (before != Plain && after != Plain) {
        return true;
    }
    matches!(
        tag,
        Some(
            "!" | "tag:yaml.org,2002:str"
                | "tag:yaml.org,2002:null"
                | "tag:yaml.org,2002:bool"
                | "tag:yaml.org,2002:int"
                | "tag:yaml.org,2002:float"
        )
    ) || classify(value) == ScalarType::String
}
