use super::{Comment, CommentPosition, EditableDocument, NodeKind, layout::Layout, view::Syntax};
use crate::{CollectionStyle, Error, NodeId, ScalarStyle, Span, comments::CommentPlacement};
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
    options: crate::ParseOptions,
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
        pending_indent: None,
        trailing_block_scalar: false,
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
        ending: if emitter.trailing_block_scalar {
            Ending::BlockScalar
        } else if emitter.pending_indent.is_some() {
            Ending::Comment
        } else {
            Ending::Token
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
    remaining: std::slice::Iter<'a, Comment<Location>>,
}

impl<'a, Location> Comments<'a, Location> {
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

    fn take(&mut self, placement: CommentPlacement) -> Option<&'a Comment<Location>> {
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

    fn take_header(&mut self) -> Option<&'a Comment<Location>> {
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

/// Return whether a block collection has enough headers for its inline comments.
/// Comments can use the enclosing indicator, tag, and anchor. Any remaining
/// inline comment requires a flow delimiter.
pub(super) fn block_headers_cover_comments<Location>(
    node: &super::Node<Location>,
    prefix_before_slot: bool,
) -> bool {
    let mut comments = Comments {
        remaining: node.comments.iter(),
    };
    if prefix_before_slot {
        while comments.take(CommentPlacement::Leading).is_some() {}
    }
    comments.take(CommentPlacement::Inline);
    while comments
        .remaining
        .as_slice()
        .first()
        .is_some_and(|comment| {
            comment.placement == CommentPlacement::Leading
                || (comment.position == CommentPosition::Before
                    && comment.placement == CommentPlacement::Detached)
        })
    {
        comments.remaining.next();
    }
    for _ in node.tag.iter().chain(node.anchor.iter()) {
        if comments.take(CommentPlacement::Inline).is_some() {
            while comments.take(CommentPlacement::Leading).is_some() {}
        }
        while comments.take_header().is_some() {}
    }
    !comments
        .remaining
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

struct Emitter<'a, 'layout, Location> {
    document: Syntax<'a, Location>,
    options: crate::ParseOptions,
    layout: &'layout Layout,
    root: NodeId,
    root_comments: &'a [Comment<Location>],
    region: Span,
    at_column_zero: bool,
    output: String,
    work: usize,
    // A comment terminator is materialized only when following syntax needs it.
    pending_indent: Option<usize>,
    trailing_block_scalar: bool,
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
            if let Some(indent) = self.pending_indent.take() {
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

    fn node_comments(&self, id: NodeId) -> Comments<'a, Location> {
        let comments = if id == self.root {
            self.root_comments
        } else {
            &self.document.nodes[id.0].comments
        };
        Comments {
            remaining: comments.iter(),
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
            self.push("#")?;
            self.push(&comment.text)?;
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
        comments: Comments<'_, Location>,
        indent: usize,
        suffix: &str,
    ) -> Result<(), Error> {
        self.push(suffix)?;
        for comment in comments.remaining {
            self.comment(comment, indent)?;
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
        while comments
            .remaining
            .as_slice()
            .first()
            .is_some_and(|comment| {
                id != self.document.root
                    || comment.span.into().is_some_and(|span| {
                        self.document.nodes[id.0]
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
        self.after(comments, suffix_indent, suffix)
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
        let simple = key_form(&self.document.nodes[key.0], is_flow) != KeyForm::Explicit;
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
        context: Context,
        comments: &mut Comments<'_, Location>,
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

pub(super) fn key_form<Location>(node: &super::Node<Location>, flow: bool) -> KeyForm {
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
    let text = scalar(value, *style, position, 0);
    let tag = node.tag.as_ref().map_or(0, |tag| tag.chars().count() + 4);
    let anchor = node
        .anchor
        .as_ref()
        .map_or(0, |anchor| anchor.chars().count() + 2);
    if text.contains('\n') || text.chars().count() + tag + anchor > 1024 {
        KeyForm::Explicit
    } else if text.starts_with(['\'', '"']) {
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
