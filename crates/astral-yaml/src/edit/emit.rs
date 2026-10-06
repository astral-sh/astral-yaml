use super::{Comment, CommentPosition, EditableDocument, NodeKind};
use crate::{CollectionStyle, Error, NodeId, ScalarStyle, Span, comments::CommentPlacement};
use std::fmt::Write;

#[cfg(test)]
#[path = "emit_tests.rs"]
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
        .rsplit(['\r', '\n'])
        .next()
        .unwrap_or("")
        .bytes()
        .take_while(|&byte| byte == b' ')
        .count()
}

pub(super) fn newline(source: &str) -> &'static str {
    match source
        .bytes()
        .position(|byte| matches!(byte, b'\r' | b'\n'))
    {
        Some(index) if source.as_bytes()[index] == b'\r' => {
            if source.as_bytes().get(index + 1) == Some(&b'\n') {
                "\r\n"
            } else {
                "\r"
            }
        }
        _ => "\n",
    }
}

pub(super) fn render(
    document: &EditableDocument,
    id: NodeId,
    region: Span,
    indent: usize,
    flow: bool,
) -> Result<String, Error> {
    let mut emitter = Emitter {
        document,
        root: id,
        region,
        newline: newline(&document.source),
        output: String::new(),
        work: document.options.max_nodes,
    };
    if region.is_empty()
        && matches!(
            document.source.as_bytes()[..region.start()].last(),
            Some(b':' | b'-' | b'?')
        )
    {
        emitter.push(" ")?;
    }
    let context = Context::value(indent, flow, if id == document.root { 0 } else { indent });
    emitter.node(id, context, "", 0)?;
    Ok(emitter.output)
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
struct Comments {
    remaining: std::vec::IntoIter<Comment>,
}

impl Comments {
    fn next_is(&self, placement: CommentPlacement) -> bool {
        self.remaining
            .as_slice()
            .first()
            .is_some_and(|comment| comment.placement == placement)
    }

    fn take(&mut self, placement: CommentPlacement) -> Option<Comment> {
        self.next_is(placement)
            .then(|| self.remaining.next().expect("next comment"))
    }

    fn has_prefix(&self) -> bool {
        self.remaining.as_slice().iter().any(|comment| {
            comment.position != CommentPosition::After
                || comment.placement == CommentPlacement::Leading
        })
    }

    fn take_header(&mut self) -> Option<Comment> {
        self.remaining
            .as_slice()
            .first()
            .is_some_and(|comment| {
                comment.position == CommentPosition::Header
                    && comment.placement != CommentPlacement::Inline
            })
            .then(|| self.remaining.next().expect("header comment"))
    }

    fn needs_value_line(&self) -> bool {
        self.remaining
            .as_slice()
            .first()
            .is_some_and(|comment| comment.position == CommentPosition::Before)
            || self
                .remaining
                .as_slice()
                .iter()
                .any(|comment| comment.placement == CommentPlacement::Leading)
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

struct Emitter<'a> {
    document: &'a EditableDocument,
    root: NodeId,
    region: Span,
    newline: &'static str,
    output: String,
    work: usize,
}

impl Emitter<'_> {
    fn error(&self, message: &'static str) -> Error {
        Error {
            span: self.region,
            message,
        }
    }

    fn push(&mut self, text: &str) -> Result<(), Error> {
        if text.len()
            > self
                .document
                .options
                .max_input_bytes
                .saturating_sub(self.output.len())
        {
            return Err(self.error("emitted YAML exceeds input byte limit"));
        }
        self.output.push_str(text);
        Ok(())
    }

    fn line(&mut self, indent: usize) -> Result<(), Error> {
        if let Some(index) = self
            .output
            .rfind(['\r', '\n'])
            .filter(|&index| self.output[index + 1..].bytes().all(|byte| byte == b' '))
        {
            self.output.truncate(index + 1);
        } else {
            self.push(self.newline)?;
        }
        for _ in 0..indent {
            self.push(" ")?;
        }
        Ok(())
    }

    fn inline(&mut self, comments: &mut Comments) -> Result<bool, Error> {
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
        comments: &mut Comments,
        indent: usize,
        block_collection: bool,
    ) -> Result<(), Error> {
        // Reserve the last inline comment for the content, unless a leading
        // comment still needs a boundary before that content begins.
        if (block_collection || comments.needs_header()) && self.inline(comments)? {
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

    fn visible_comments(&self, id: NodeId) -> impl Iterator<Item = &Comment> {
        self.document.nodes[id.0]
            .comments
            .iter()
            .filter(move |comment| {
                id != self.root
                    || comment.span.is_empty()
                    || (comment.span.start() >= self.region.start()
                        && comment.span.end() <= self.region.end())
            })
    }

    fn node_comments(&self, id: NodeId) -> Comments {
        Comments {
            remaining: self
                .visible_comments(id)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }

    fn before(&mut self, comments: &mut Comments, indent: usize) -> Result<(), Error> {
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

    fn after(&mut self, comments: Comments, indent: usize, suffix: &str) -> Result<(), Error> {
        self.push(suffix)?;
        for comment in comments.remaining {
            if comment.placement == CommentPlacement::Inline && !self.output.ends_with(['\n', '\r'])
            {
                self.push(" ")?;
            } else {
                self.line(indent)?;
            }
            if comment.placement == CommentPlacement::Detached {
                self.push(self.newline)?;
                self.line(indent)?;
            }
            self.push("#")?;
            self.push(&comment.text)?;
            // A comment must never swallow a following delimiter or property.
            self.line(indent)?;
        }
        Ok(())
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
        mut comments: Comments,
    ) -> Result<(), Error> {
        let indent = context.indent;
        let flow = context.in_flow();
        if depth >= self.document.options.max_depth || self.work == 0 {
            return Err(self.error("emission resource limit exceeded"));
        }
        self.work -= 1;
        self.before(&mut comments, indent)?;
        self.properties(id, indent, flow, &mut comments)?;
        let node = &self.document.nodes[id.0];
        match &node.kind {
            NodeKind::Scalar { value, style } => {
                self.scalar(value, *style, context, &mut comments)?;
            }
            NodeKind::Alias { name, .. } => {
                self.push("*")?;
                self.push(name)?;
            }
            NodeKind::Sequence { items, style } => {
                let is_flow = flow || *style == CollectionStyle::Flow || items.is_empty();
                if is_flow {
                    self.push("[")?;
                    if comments.needs_header() && self.inline(&mut comments)? {
                        self.line(indent + 2)?;
                        self.before(&mut comments, indent + 2)?;
                    }
                    for (index, &item) in items.iter().enumerate() {
                        if index != 0 {
                            self.push(" ")?;
                        }
                        self.node(
                            item,
                            Context::flow(indent + 2),
                            if index + 1 == items.len() { "" } else { "," },
                            depth + 1,
                        )?;
                    }
                    self.push("]")?;
                } else {
                    if node.anchor.is_some() || node.tag.is_some() {
                        self.line(indent)?;
                    }
                    for (index, &item) in items.iter().enumerate() {
                        if index != 0 {
                            self.line(indent)?;
                        }
                        let mut comments = self.node_comments(item);
                        self.before(&mut comments, indent)?;
                        self.push("-")?;
                        self.value(item, context, "", depth + 1, comments)?;
                    }
                }
            }
            NodeKind::Mapping { entries, style } => {
                let is_flow = flow || *style == CollectionStyle::Flow || entries.is_empty();
                if is_flow {
                    self.push("{")?;
                    if comments.needs_header() && self.inline(&mut comments)? {
                        self.line(indent + 2)?;
                        self.before(&mut comments, indent + 2)?;
                    }
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
                    let mut value_comments = self.node_comments(value);
                    self.before(
                        &mut value_comments,
                        if is_flow { indent + 2 } else { indent },
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
                    self.push("}")?;
                }
            }
        }
        self.after(comments, indent, suffix)
    }

    fn key(&mut self, key: NodeId, context: Context, depth: usize) -> Result<(), Error> {
        let indent = context.indent;
        let is_flow = context.in_flow();
        let simple = self.simple_key(key, is_flow);
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
        comments: &mut Comments,
    ) -> Result<(), Error> {
        let node = &self.document.nodes[id.0];
        let block_collection = block_collection(&node.kind, flow);
        let continuation_indent = if flow || block_collection {
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
            self.property_comment(comments, continuation_indent, block_collection)?;
        }
        if let Some(anchor) = &node.anchor {
            self.push("&")?;
            self.push(anchor)?;
            self.push(" ")?;
            self.property_comment(comments, continuation_indent, block_collection)?;
        }
        Ok(())
    }

    fn scalar(
        &mut self,
        value: &str,
        style: ScalarStyle,
        context: Context,
        comments: &mut Comments,
    ) -> Result<(), Error> {
        let indent = match context.position {
            Position::Block { parent_indent }
                if matches!(style, ScalarStyle::Literal | ScalarStyle::Folded) =>
            {
                parent_indent
            }
            _ => context.indent,
        };
        let position = match context.position {
            Position::Flow => ScalarPosition::Flow,
            Position::ImplicitKey { flow: true } => ScalarPosition::FlowKey,
            Position::ImplicitKey { flow: false } => ScalarPosition::BlockKey,
            Position::Block { .. } if self.at_document_boundary() => ScalarPosition::Document,
            Position::Block { .. } => ScalarPosition::Block,
        };
        let text = scalar(value, style, position, indent, self.newline);
        if text.starts_with(['|', '>']) {
            let (header, body) = text.split_once(self.newline).expect("block scalar header");
            self.push(header)?;
            self.inline(comments)?;
            self.push(self.newline)?;
            self.push(body)?;
            return Ok(());
        }
        self.push(&text)
    }

    fn at_document_boundary(&self) -> bool {
        // Document markers are recognized only at column zero. Include the
        // retained source prefix when emission has not started a new line.
        if let Some(index) = self.output.rfind(['\r', '\n']) {
            return self.output[index + 1..].is_empty();
        }
        let prefix = self.document.source[..self.region.start()]
            .rsplit(['\r', '\n'])
            .next()
            .unwrap_or("");
        prefix.trim_start_matches('\u{feff}').is_empty() && self.output.is_empty()
    }

    fn simple_key(&self, id: NodeId, flow: bool) -> bool {
        let node = &self.document.nodes[id.0];
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
        let text = scalar(value, *style, position, 0, self.newline);
        let tag = node.tag.as_ref().map_or(0, |tag| tag.chars().count() + 4);
        let anchor = node
            .anchor
            .as_ref()
            .map_or(0, |anchor| anchor.chars().count() + 2);
        !text.contains(['\r', '\n']) && text.chars().count() + tag + anchor <= 1024
    }

    fn value(
        &mut self,
        id: NodeId,
        context: Context,
        suffix: &str,
        depth: usize,
        mut comments: Comments,
    ) -> Result<(), Error> {
        let indent = context.indent;
        let flow = context.in_flow();
        let block = block_collection(&self.document.nodes[id.0].kind, flow);
        let leading = comments.needs_value_line();
        let value_indent = if block || flow || leading {
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
        let context = Context::value(value_indent, flow, indent);
        self.node_with_comments(id, context, suffix, depth, comments)
    }
}

fn block_collection(kind: &NodeKind, flow: bool) -> bool {
    !flow
        && (matches!(kind,
        NodeKind::Sequence { items, style: CollectionStyle::Block } if !items.is_empty())
            || matches!(kind,
        NodeKind::Mapping { entries, style: CollectionStyle::Block } if !entries.is_empty()))
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

fn flow_scalar(value: &str, indent: usize, newline: &str) -> String {
    let continuation = " ".repeat(indent + 2);
    let mut result = String::new();
    let mut after_break = false;
    for character in value.chars() {
        if character == '\n' {
            // Flow folding consumes one physical break in each run. An extra
            // empty line retains every decoded break, including at the edges
            // of a single-quoted scalar.
            if !after_break {
                result.push_str(newline);
            }
            result.push_str(newline);
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

fn block_scalar(value: &str, style: ScalarStyle, indent: usize, newline: &str) -> String {
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
    let mut result = format!("{indicator}2{chomp}{newline}");
    let indentation = " ".repeat(indent + 2);
    for line in body.split_terminator('\n') {
        result.push_str(&indentation);
        result.push_str(line);
        result.push_str(newline);
    }
    result
}

fn scalar(
    value: &str,
    style: ScalarStyle,
    position: ScalarPosition,
    indent: usize,
    newline: &str,
) -> String {
    // Check the generated scalar in its actual flow/block context. This also
    // handles plain-scalar punctuation, folding, and explicit indentation.
    let candidate = match style {
        // Empty scalar text is observable through Serde string visitors and
        // application tags, even when the core schema resolves it as null.
        ScalarStyle::Plain if value.is_empty() => return String::new(),
        ScalarStyle::Plain => flow_scalar(value, indent, newline),
        ScalarStyle::SingleQuoted => {
            format!(
                "'{}'",
                flow_scalar(value, indent, newline).replace('\'', "''")
            )
        }
        ScalarStyle::Literal | ScalarStyle::Folded
            if matches!(position, ScalarPosition::Block | ScalarPosition::Document) =>
        {
            block_scalar(value, style, indent, newline)
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
pub(super) fn correspondence(
    expected: &EditableDocument,
    actual: &EditableDocument,
) -> Option<Vec<Option<NodeId>>> {
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
            (NodeKind::Scalar { value: a, .. }, NodeKind::Scalar { value: b, .. }) => {
                let kind = scalar_type(left);
                if kind != scalar_type(right) || a != b {
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

pub(super) fn comments_preserved(
    expected: &EditableDocument,
    actual: &EditableDocument,
    correspondence: &[Option<NodeId>],
) -> bool {
    correspondence.iter().enumerate().all(|(id, target)| {
        let Some(target) = target else {
            return true;
        };
        let left = &expected.nodes[id].comments;
        let right = &actual.nodes[target.0].comments;
        left.len() == right.len()
            && left
                .iter()
                .zip(right)
                .all(|(left, right)| left.text == right.text && left.placement == right.placement)
    })
}

fn scalar_type(node: &super::Node) -> crate::schema::ScalarType {
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
        NodeKind::Scalar {
            value,
            style: ScalarStyle::Plain,
        } => classify(value),
        _ => ScalarType::String,
    }
}
