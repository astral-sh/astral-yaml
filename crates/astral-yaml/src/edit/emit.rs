use super::{Comment, EditableDocument, NodeKind};
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
        skip_leading: false,
        skip_inline: 0,
    };
    if region.is_empty()
        && matches!(
            document.source.as_bytes()[..region.start()].last(),
            Some(b':' | b'-' | b'?')
        )
    {
        emitter.push(" ")?;
    }
    let context = Context {
        indent,
        flow,
        block_indent: if id == document.root { 0 } else { indent },
    };
    emitter.node(id, context, "", 0)?;
    Ok(emitter.output)
}

#[derive(Clone, Copy)]
struct Context {
    /// Indentation for this node's continuation lines and comments.
    indent: usize,
    flow: bool,
    /// A block scalar's indentation indicator is relative to its parent, even
    /// when comments or properties put its header on a more-indented line.
    block_indent: usize,
}

impl Context {
    fn flow(indent: usize) -> Self {
        Self {
            indent,
            flow: true,
            block_indent: indent,
        }
    }
}

struct Emitter<'a> {
    document: &'a EditableDocument,
    root: NodeId,
    region: Span,
    newline: &'static str,
    output: String,
    work: usize,
    skip_leading: bool,
    skip_inline: usize,
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

    fn inline(&mut self, id: NodeId, index: usize) -> Result<usize, Error> {
        let text = self
            .comments(id, CommentPlacement::Inline)
            .into_iter()
            .nth(index)
            .map(|comment| comment.text.clone());
        if let Some(text) = text {
            self.push(if self.output.ends_with(' ') {
                "#"
            } else {
                " #"
            })?;
            self.push(&text)?;
            Ok(1)
        } else {
            Ok(0)
        }
    }

    fn property_comment(
        &mut self,
        id: NodeId,
        inline_skip: &mut usize,
        indent: usize,
        block_collection: bool,
    ) -> Result<(), Error> {
        // Keep one comment for the scalar or flow collection itself. Block
        // collections must place their inline comments on the property lines.
        if (block_collection
            || self.comments(id, CommentPlacement::Inline).len() > *inline_skip + 1)
            && self.inline(id, *inline_skip)? != 0
        {
            *inline_skip += 1;
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

    fn comments(&self, id: NodeId, placement: CommentPlacement) -> Vec<&Comment> {
        self.visible_comments(id)
            .filter(|comment| comment.placement == placement)
            .collect()
    }

    fn leading_comments(&self, id: NodeId, inline_skip: usize) -> Vec<&Comment> {
        let mut inlines = 0;
        self.visible_comments(id)
            .filter(|comment| {
                if comment.placement == CommentPlacement::Inline {
                    inlines += 1;
                }
                comment.placement == CommentPlacement::Leading && inlines == inline_skip
            })
            .collect()
    }

    fn before(&mut self, id: NodeId, indent: usize, inline_skip: usize) -> Result<(), Error> {
        // Clone only comment text so writing can borrow the emitter mutably.
        let comments: Vec<_> = self
            .leading_comments(id, inline_skip)
            .into_iter()
            .map(|c| c.text.clone())
            .collect();
        // Leading comments need their own line, including after flow delimiters.
        if !comments.is_empty() && !self.output.is_empty() {
            self.line(indent)?;
        }
        for text in comments {
            self.push("#")?;
            self.push(&text)?;
            self.line(indent)?;
        }
        Ok(())
    }

    fn after(
        &mut self,
        id: NodeId,
        indent: usize,
        suffix: &str,
        inline_skip: usize,
    ) -> Result<(), Error> {
        self.push(suffix)?;
        for placement in [
            CommentPlacement::Inline,
            CommentPlacement::Trailing,
            CommentPlacement::Detached,
        ] {
            let comments: Vec<_> = self
                .comments(id, placement)
                .into_iter()
                .skip(if placement == CommentPlacement::Inline {
                    inline_skip
                } else {
                    0
                })
                .map(|c| c.text.clone())
                .collect();
            for text in comments {
                if placement == CommentPlacement::Inline && !self.output.ends_with(['\n', '\r']) {
                    self.push(" ")?;
                } else {
                    self.line(indent)?;
                }
                if placement == CommentPlacement::Detached {
                    self.push(self.newline)?;
                    self.line(indent)?;
                }
                self.push("#")?;
                self.push(&text)?;
                // A comment must never swallow a following delimiter or property.
                self.line(indent)?;
            }
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
        let Context { indent, flow, .. } = context;
        if depth >= self.document.options.max_depth || self.work == 0 {
            return Err(self.error("emission resource limit exceeded"));
        }
        self.work -= 1;
        let mut inline_skip = std::mem::take(&mut self.skip_inline);
        if !std::mem::take(&mut self.skip_leading) {
            self.before(id, indent, inline_skip)?;
        }
        self.properties(id, indent, flow, &mut inline_skip)?;
        let node = &self.document.nodes[id.0];
        match &node.kind {
            NodeKind::Scalar { value, style } => {
                inline_skip += self.scalar(id, value, *style, context, inline_skip)?;
            }
            NodeKind::Alias { name, .. } => {
                self.push("*")?;
                self.push(name)?;
            }
            NodeKind::Sequence { items, style } => {
                let is_flow = flow || *style == CollectionStyle::Flow || items.is_empty();
                if is_flow {
                    self.push("[")?;
                    if self.comments(id, CommentPlacement::Inline).len() > inline_skip + 1 {
                        inline_skip += self.inline(id, inline_skip)?;
                        self.line(indent + 2)?;
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
                        self.before(item, indent, 0)?;
                        self.push("-")?;
                        self.value(item, indent, false, "", depth + 1)?;
                    }
                }
            }
            NodeKind::Mapping { entries, style } => {
                let is_flow = flow || *style == CollectionStyle::Flow || entries.is_empty();
                if is_flow {
                    self.push("{")?;
                    if self.comments(id, CommentPlacement::Inline).len() > inline_skip + 1 {
                        inline_skip += self.inline(id, inline_skip)?;
                        self.line(indent + 2)?;
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
                    self.before(value, if is_flow { indent + 2 } else { indent }, 0)?;
                    let simple = self.simple_key(key);
                    if !simple {
                        self.push("? ")?;
                    }
                    self.node(key, Context::flow(indent + 2), "", depth + 1)?;
                    if !simple {
                        self.line(if is_flow { indent + 2 } else { indent })?;
                    }
                    self.push(":")?;
                    let suffix = if is_flow && index + 1 != entries.len() {
                        ","
                    } else {
                        ""
                    };
                    self.value(value, indent, is_flow, suffix, depth + 1)?;
                }
                if is_flow {
                    self.push("}")?;
                }
            }
        }
        self.after(id, indent, suffix, inline_skip)
    }

    fn properties(
        &mut self,
        id: NodeId,
        indent: usize,
        flow: bool,
        inline_skip: &mut usize,
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
            self.property_comment(id, inline_skip, continuation_indent, block_collection)?;
        }
        if let Some(anchor) = &node.anchor {
            self.push("&")?;
            self.push(anchor)?;
            self.push(" ")?;
            self.property_comment(id, inline_skip, continuation_indent, block_collection)?;
        }
        Ok(())
    }

    fn scalar(
        &mut self,
        id: NodeId,
        value: &str,
        style: ScalarStyle,
        context: Context,
        inline_skip: usize,
    ) -> Result<usize, Error> {
        let indent = if matches!(style, ScalarStyle::Literal | ScalarStyle::Folded) {
            context.block_indent
        } else {
            context.indent
        };
        let text = scalar(value, style, context.flow, indent, self.newline);
        if text.starts_with(['|', '>']) {
            let (header, body) = text.split_once(self.newline).expect("block scalar header");
            self.push(header)?;
            let emitted = self.inline(id, inline_skip)?;
            self.push(self.newline)?;
            self.push(body)?;
            return Ok(emitted);
        }
        self.push(&text)?;
        Ok(0)
    }

    fn simple_key(&self, id: NodeId) -> bool {
        let node = &self.document.nodes[id.0];
        let NodeKind::Scalar { value, style } = &node.kind else {
            return false;
        };
        if !node.comments.is_empty() {
            return false;
        }
        // YAML limits the entire implicit key, including escapes and properties,
        // to 1024 source characters on one line.
        let text = scalar(value, *style, true, 0, self.newline);
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
        indent: usize,
        flow: bool,
        suffix: &str,
        depth: usize,
    ) -> Result<(), Error> {
        let node = &self.document.nodes[id.0];
        let block = block_collection(&node.kind, flow);
        // A field/item header can own an inline comment before the value's
        // leading comments. Emit that ordered prefix here, then let the node
        // emit the leading comments that follow it on the continuation line.
        let leading_after_header = !self.leading_comments(id, 1).is_empty();
        let value_indent = if block || flow || leading_after_header {
            indent + 2
        } else {
            indent
        };
        if block || leading_after_header {
            self.skip_inline = self.inline(id, 0)?;
            self.line(value_indent)?;
        } else {
            self.push(" ")?;
        }
        self.skip_leading = !leading_after_header;
        let context = Context {
            indent: value_indent,
            flow,
            block_indent: indent,
        };
        self.node(id, context, suffix, depth)
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

fn scalar(value: &str, style: ScalarStyle, flow: bool, indent: usize, newline: &str) -> String {
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
        ScalarStyle::Literal | ScalarStyle::Folded if !flow => {
            block_scalar(value, style, indent, newline)
        }
        ScalarStyle::DoubleQuoted | ScalarStyle::Literal | ScalarStyle::Folded => {
            return quoted(value);
        }
    };
    let source = if flow {
        format!("[{candidate}]")
    } else {
        // A sequence supplies the parent indentation used by block scalars.
        // Indenting a standalone scalar's header does not establish it.
        format!("{}- {candidate}", " ".repeat(indent))
    };
    let matches = crate::parse(&source).is_ok_and(|stream| {
        let [document] = stream.documents() else {
            return false;
        };
        let root = stream.node(document.root).expect("parsed root");
        let crate::NodeKind::Sequence { items, .. } = &root.kind else {
            return false;
        };
        let [id] = items.as_slice() else {
            return false;
        };
        let node = stream.node(*id).expect("parsed item");
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
