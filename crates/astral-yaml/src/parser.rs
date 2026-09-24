use std::{borrow::Cow, collections::HashMap};

use crate::{
    CollectionStyle, Comment, Directive, Document, Entry, Error, Node, NodeId, NodeKind, Property,
    Scalar, ScalarStyle, Span, Stream,
};

type Result<T> = std::result::Result<T, Error>;

/// Resource limits applied before allocation or recursive descent.
#[derive(Clone, Copy, Debug)]
pub struct ParseOptions {
    /// Maximum collection nesting, between 1 and 128.
    pub max_depth: usize,
    /// Maximum number of nodes, comments, and directives in a stream.
    pub max_nodes: usize,
    /// Maximum UTF-8 input length in bytes.
    pub max_input_bytes: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            max_depth: 64,
            max_nodes: 1_000_000,
            max_input_bytes: 64 * 1024 * 1024,
        }
    }
}

/// Parses a UTF-8 YAML stream with the default resource limits.
///
/// # Errors
/// Returns a located error for invalid syntax or an exceeded resource limit.
pub fn parse(source: &str) -> Result<Stream<'_>> {
    parse_with_options(source, ParseOptions::default())
}

/// Parses a UTF-8 YAML stream with explicit resource limits.
///
/// # Errors
/// Returns a located error for invalid syntax, invalid options, or an exceeded limit.
pub fn parse_with_options(source: &str, options: ParseOptions) -> Result<Stream<'_>> {
    let error = |message| Error {
        span: Span { start: 0, end: 0 },
        message,
    };
    if !(1..=128).contains(&options.max_depth) {
        return Err(error("max_depth must be between 1 and 128"));
    }
    if source.len() > options.max_input_bytes {
        return Err(error("input byte limit exceeded"));
    }
    let stream = Parser {
        source,
        pos: 0,
        options,
        nodes: Vec::new(),
        documents: Vec::new(),
        comments: Vec::new(),
        anchors: HashMap::new(),
        tags: HashMap::new(),
        byte_order_marks: Vec::new(),
        elements: 0,
    }
    .stream()?;
    validate_characters(&stream)?;
    Ok(stream)
}

struct Parser<'a> {
    source: &'a str,
    pos: usize,
    options: ParseOptions,
    nodes: Vec<Node<'a>>,
    documents: Vec<Document<'a>>,
    comments: Vec<Comment<'a>>,
    anchors: HashMap<&'a str, NodeId>,
    tags: HashMap<&'a str, String>,
    byte_order_marks: Vec<Span>,
    elements: usize,
}

impl<'a> Parser<'a> {
    fn error(&self, message: &'static str) -> Error {
        Error {
            span: Span {
                start: self.pos,
                end: self.pos + self.rest().chars().next().map_or(0, char::len_utf8),
            },
            message,
        }
    }
    fn rest(&self) -> &'a str {
        &self.source[self.pos..]
    }
    fn byte(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos).copied()
    }
    fn next_byte(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos + 1).copied()
    }
    fn column(&self) -> usize {
        let line_start = self.source[..self.pos]
            .rfind(['\r', '\n'])
            .map_or(0, |n| n + 1);
        let prefix = &self.source[line_start..self.pos];
        prefix.strip_prefix('\u{feff}').unwrap_or(prefix).len()
    }
    fn line_end(&self) -> usize {
        self.rest()
            .find(['\r', '\n'])
            .map_or(self.source.len(), |n| self.pos + n)
    }
    fn spaces(&mut self) {
        while matches!(self.byte(), Some(b' ' | b'\t')) {
            self.pos += 1;
        }
    }
    fn newline(&mut self) -> bool {
        match self.byte() {
            Some(b'\r') => {
                self.pos += 1;
                if self.byte() == Some(b'\n') {
                    self.pos += 1;
                }
                true
            }
            Some(b'\n') => {
                self.pos += 1;
                true
            }
            _ => false,
        }
    }
    fn reserve_element(&mut self) -> Result<()> {
        if self.elements == self.options.max_nodes {
            return Err(self.error("syntax element limit exceeded"));
        }
        self.elements += 1;
        Ok(())
    }
    fn comment(&mut self) -> Result<()> {
        self.reserve_element()?;
        let start = self.pos;
        self.pos = self.line_end();
        self.comments.push(Comment {
            span: Span {
                start,
                end: self.pos,
            },
            text: &self.source[start + 1..self.pos],
        });
        Ok(())
    }
    /// Advances to the next nonempty block line, preserving comments.
    fn block_space(&mut self) -> Result<()> {
        loop {
            self.spaces();
            if self.byte() == Some(b'#') {
                self.comment()?;
            }
            if self.newline() {
                continue;
            }
            return Ok(());
        }
    }
    fn flow_space(&mut self, parent: isize) -> Result<()> {
        let mut crossed_line = false;
        loop {
            self.spaces();
            if self.byte() == Some(b'#') {
                if self.pos > 0 && !is_space(self.source.as_bytes()[self.pos - 1]) {
                    return Err(self.error("comments must be separated by whitespace"));
                }
                self.comment()?;
            }
            if !self.newline() {
                break;
            }
            crossed_line = true;
        }
        if crossed_line && self.byte().is_some() && signed(self.leading_spaces()) <= parent {
            return Err(self.error("flow continuation must be indented"));
        }
        Ok(())
    }
    fn leading_spaces(&self) -> usize {
        let start = self.pos - self.column();
        self.source[start..]
            .bytes()
            .take_while(|&b| b == b' ')
            .count()
    }
    fn document_boundary(&self) -> bool {
        self.byte().is_none()
            || self.marker("---")
            || self.marker("...")
            || (self.column() == 0 && self.rest().starts_with('\u{feff}'))
    }
    fn finish_line(&mut self) -> Result<()> {
        let before = self.pos;
        self.spaces();
        if self.byte() == Some(b'#') {
            if before == self.pos {
                return Err(self.error("a comment must be separated from its node"));
            }
            self.comment()?;
        }
        if self.newline() {
            self.block_space()
        } else if self.byte().is_none() {
            Ok(())
        } else {
            Err(self.error("unexpected content after node"))
        }
    }
    fn indicator(&self, b: u8) -> bool {
        self.byte() == Some(b) && self.next_byte().is_none_or(is_space)
    }
    fn marker(&self, marker: &str) -> bool {
        self.column() == 0
            && self.rest().starts_with(marker)
            && self
                .source
                .as_bytes()
                .get(self.pos + marker.len())
                .is_none_or(|b| is_space(*b))
    }
    fn boundary(&self) -> bool {
        self.document_boundary() || (self.column() == 0 && self.byte() == Some(b'%'))
    }
    fn empty(&mut self, offset: usize) -> Result<NodeId> {
        self.reserve_element()?;
        let id = NodeId(self.nodes.len());
        let span = Span {
            start: offset,
            end: offset,
        };
        self.nodes.push(Node {
            span,
            anchor: None,
            tag: None,
            kind: NodeKind::Scalar(Scalar {
                value: Cow::Borrowed(""),
                style: ScalarStyle::Plain,
                span,
            }),
        });
        Ok(id)
    }
    fn check_depth(&self, depth: usize) -> Result<()> {
        if depth >= self.options.max_depth {
            Err(self.error("nesting limit exceeded"))
        } else {
            Ok(())
        }
    }
    fn consume_bom(&mut self) {
        if self.column() == 0 && self.rest().starts_with('\u{feff}') {
            self.byte_order_marks.push(Span {
                start: self.pos,
                end: self.pos + 3,
            });
            self.pos += 3;
        }
    }
    fn stream(mut self) -> Result<Stream<'a>> {
        self.consume_bom();
        self.block_space()?;
        let mut previous_ended = true;
        while self.byte().is_some() {
            self.consume_bom();
            self.block_space()?;
            if self.byte().is_none() {
                break;
            }
            if self.marker("...") {
                self.pos += 3;
                self.finish_line()?;
                previous_ended = true;
                continue;
            }
            self.anchors.clear();
            self.tags.clear();
            self.tags.insert("!", "!".to_owned());
            self.tags.insert("!!", "tag:yaml.org,2002:".to_owned());
            let start = self.pos;
            if !previous_ended && self.byte() == Some(b'%') {
                return Err(self.error("directives require a preceding document end marker"));
            }
            let directives = self.directives()?;
            let start_marker = if self.marker("---") {
                let span = Span {
                    start: self.pos,
                    end: self.pos + 3,
                };
                self.pos += 3;
                self.spaces();
                if self.byte() == Some(b'#') {
                    self.comment()?;
                }
                if self.newline() {
                    self.block_space()?;
                } else if self.indicator(b'-')
                    || self.indicator(b'?')
                    || self.simple_colon().is_some()
                {
                    return Err(
                        self.error("block collections cannot start on the document marker line")
                    );
                }
                Some(span)
            } else {
                if !directives.is_empty() {
                    return Err(self.error("directives require a document start marker"));
                }
                if !previous_ended {
                    return Err(self.error("expected a document start marker"));
                }
                None
            };
            let root = if self.boundary() {
                self.empty(self.pos)?
            } else {
                self.block(-1, 0, false)?
            };
            let content_end = self.nodes[root.0].span.end;
            let end_marker = if self.marker("...") {
                let span = Span {
                    start: self.pos,
                    end: self.pos + 3,
                };
                self.pos += 3;
                self.finish_line()?;
                Some(span)
            } else {
                None
            };
            previous_ended = end_marker.is_some();
            self.documents.push(Document {
                span: Span {
                    start,
                    end: end_marker.map_or(content_end, |s| s.end),
                },
                root,
                start_marker,
                end_marker,
                directives,
            });
        }
        Ok(Stream {
            source: self.source,
            nodes: self.nodes,
            documents: self.documents,
            comments: self.comments,
            byte_order_marks: self.byte_order_marks,
        })
    }
    fn directives(&mut self) -> Result<Vec<Directive<'a>>> {
        let mut directives = Vec::new();
        let mut version = false;
        let mut handles = Vec::new();
        while self.column() == 0 && self.byte() == Some(b'%') {
            self.reserve_element()?;
            let start = self.pos;
            let end = self.line_end();
            self.pos += 1;
            let name_start = self.pos;
            while self.byte().is_some_and(|b| !is_space(b)) {
                self.pos += 1;
            }
            let name = &self.source[name_start..self.pos];
            self.spaces();
            let argument_start = self.pos;
            while self.pos < end {
                if self.byte() == Some(b'#')
                    && self.pos > argument_start
                    && self.source.as_bytes()[self.pos - 1].is_ascii_whitespace()
                {
                    break;
                }
                self.pos += self.rest().chars().next().map_or(0, char::len_utf8);
            }
            let args = self.source[argument_start..self.pos].trim_end_matches([' ', '\t']);
            let span = Span {
                start,
                end: argument_start + args.len(),
            };
            let words: Vec<_> = args.split_ascii_whitespace().collect();
            match name {
                "YAML" => {
                    if version {
                        return Err(self.error("duplicate YAML directive"));
                    }
                    if words.len() != 1
                        || !words[0].starts_with("1.")
                        || words[0].len() < 3
                        || !words[0][2..].bytes().all(|b| b.is_ascii_digit())
                    {
                        return Err(self.error("invalid YAML version directive"));
                    }
                    // New minor versions use the 1.2 grammar; 1.1 schema behavior is not enabled.
                    version = true;
                }
                "TAG" => {
                    if words.len() != 2 || !valid_handle(words[0]) {
                        return Err(self.error("invalid TAG directive"));
                    }
                    if handles.contains(&words[0]) {
                        return Err(self.error("duplicate TAG directive"));
                    }
                    handles.push(words[0]);
                    let prefix = tag_uri(words[1]).ok_or_else(|| self.error("invalid tag URI"))?;
                    self.tags.insert(words[0], prefix);
                }
                "" => return Err(self.error("empty directive name")),
                _ => {}
            }
            directives.push(Directive {
                span,
                name,
                arguments: args,
            });
            if self.byte() == Some(b'#') {
                self.comment()?;
            }
            if self.newline() {
                self.block_space()?;
            }
        }
        Ok(directives)
    }
    fn block(&mut self, parent: isize, depth: usize, indentless: bool) -> Result<NodeId> {
        self.block_into(parent, depth, indentless, None)
    }
    fn block_into(
        &mut self,
        parent: isize,
        depth: usize,
        indentless: bool,
        existing: Option<NodeId>,
    ) -> Result<NodeId> {
        self.check_depth(depth)?;
        let start = self.pos;
        let indent = self.column();
        let prefix = &self.source[self.pos - indent..self.pos];
        let block_collection = self.indicator(b'-')
            || self.indicator(b'?')
            || self.indicator(b':')
            || self.simple_colon().is_some();
        if prefix.contains('\t')
            && (block_collection
                || (prefix.bytes().all(is_space) && signed(self.leading_spaces()) <= parent))
        {
            return Err(self.error("tab in block indentation"));
        }
        // A property preceding a mapping key belongs to that key, not the mapping.
        if self.indicator(b'-') {
            return self.block_sequence(indent, depth, existing);
        }
        if self.indicator(b'?') || self.indicator(b':') || self.simple_colon().is_some() {
            return self.block_mapping(indent, depth, existing);
        }
        let id = if let Some(id) = existing {
            id
        } else {
            self.empty(start)?
        };
        self.properties(id, false, parent)?;
        if self.byte() == Some(b'#') {
            self.comment()?;
        }
        let property_end = self.pos;
        if self.newline() {
            self.block_space()?;
            if !self.boundary()
                && (signed(self.column()) > parent
                    || (indentless && signed(self.column()) == parent && self.indicator(b'-')))
            {
                return self.block_into(parent, depth + 1, indentless, Some(id));
            }
            let span = Span {
                start: property_end,
                end: property_end,
            };
            self.nodes[id.0].span.end = property_end;
            self.nodes[id.0].kind = NodeKind::Scalar(Scalar {
                value: Cow::Borrowed(""),
                style: ScalarStyle::Plain,
                span,
            });
            return Ok(id);
        }
        let content_start = self.pos;
        let kind = match self.byte() {
            Some(b'|' | b'>') => self.block_scalar(parent)?,
            _ => self.inline_kind(false, parent, depth, false)?,
        };
        if matches!(kind, NodeKind::Alias { .. })
            && (self.nodes[id.0].anchor.is_some() || self.nodes[id.0].tag.is_some())
        {
            return Err(self.error("aliases cannot have node properties"));
        }
        let end = match &kind {
            NodeKind::Scalar(scalar) => scalar.span.end,
            _ => self.pos,
        };
        self.nodes[id.0].kind = kind;
        self.nodes[id.0].span.end = end.max(content_start);
        // Block scalar scanning already stops at the next content line.
        if !matches!(
            &self.nodes[id.0].kind,
            NodeKind::Scalar(Scalar {
                style: ScalarStyle::Literal | ScalarStyle::Folded,
                ..
            })
        ) {
            self.finish_line()?;
        }
        Ok(id)
    }
    fn block_sequence(
        &mut self,
        indent: usize,
        depth: usize,
        existing: Option<NodeId>,
    ) -> Result<NodeId> {
        let id = if let Some(id) = existing {
            id
        } else {
            self.empty(self.pos)?
        };
        let mut items = Vec::new();
        let mut end = self.pos;
        loop {
            self.pos += 1;
            self.spaces();
            let empty_at = self.pos;
            let item = if matches!(self.byte(), None | Some(b'\r' | b'\n' | b'#')) {
                if self.byte() == Some(b'#') {
                    self.comment()?;
                }
                self.newline();
                self.block_space()?;
                if !self.boundary() && self.column() > indent {
                    self.block(signed(indent), depth + 1, false)?
                } else {
                    self.empty(empty_at)?
                }
            } else {
                self.block(signed(indent), depth + 1, false)?
            };
            end = end.max(self.nodes[item.0].span.end).max(empty_at);
            items.push(item);
            if self.boundary() || self.column() != indent || !self.indicator(b'-') {
                break;
            }
        }
        self.nodes[id.0].span.end = end;
        self.nodes[id.0].kind = NodeKind::Sequence {
            items,
            style: CollectionStyle::Block,
        };
        Ok(id)
    }
    /// Finds a same-line implicit key separator outside quotes and flow brackets.
    fn simple_colon(&self) -> Option<usize> {
        let bytes = self.source.as_bytes();
        let mut i = self.pos;
        // Colons in property and alias names are not mapping separators.
        while bytes.get(i).is_some_and(|b| b"!&*".contains(b)) {
            while bytes.get(i).is_some_and(|b| !is_space(*b)) {
                i += 1;
            }
            while bytes.get(i).is_some_and(|b| matches!(b, b' ' | b'\t')) {
                i += 1;
            }
        }
        let plain = bytes.get(i).is_some_and(|b| !b"[{'\"".contains(b));
        let mut quote = None;
        let mut level = 0usize;
        while let Some(&b) = bytes.get(i) {
            if matches!(b, b'\r' | b'\n') {
                break;
            }
            if let Some(q) = quote {
                if b == b'\\' && q == b'"' {
                    i += 2;
                    continue;
                }
                if b == q {
                    quote = None;
                }
            } else {
                match b {
                    b'\'' | b'"' if !plain => quote = Some(b),
                    b'[' | b'{' if !plain => level += 1,
                    b']' | b'}' if !plain => level = level.saturating_sub(1),
                    b'#' if i == self.pos || is_space(bytes[i - 1]) => break,
                    b':' if level == 0 && bytes.get(i + 1).is_none_or(|b| is_space(*b)) => {
                        return Some(i);
                    }
                    _ => {}
                }
            }
            i += 1;
        }
        None
    }
    fn block_mapping(
        &mut self,
        indent: usize,
        depth: usize,
        existing: Option<NodeId>,
    ) -> Result<NodeId> {
        let id = if let Some(id) = existing {
            id
        } else {
            self.empty(self.pos)?
        };
        let mut entries = Vec::new();
        let mut end = self.pos;
        loop {
            let start = self.pos;
            let explicit = self.indicator(b'?');
            let key = if explicit {
                self.pos += 1;
                self.spaces();
                let at = self.pos;
                if matches!(self.byte(), None | Some(b'\n' | b'\r' | b'#')) {
                    if self.byte() == Some(b'#') {
                        self.comment()?;
                    }
                    self.newline();
                    self.block_space()?;
                    if !self.boundary()
                        && (self.column() > indent
                            || (self.column() == indent && self.indicator(b'-')))
                    {
                        self.block(signed(indent), depth + 1, true)?
                    } else {
                        self.empty(at)?
                    }
                } else {
                    self.block(signed(indent), depth + 1, true)?
                }
            } else if self.indicator(b':') {
                self.empty(self.pos)?
            } else {
                let before = self.pos;
                let key = self.inline(false, signed(indent), depth + 1, true)?;
                if self.source[before..self.pos].chars().count() > 1024
                    || self.source[before..self.pos].contains(['\r', '\n'])
                {
                    return Err(self
                        .error("an implicit key must fit on one line of at most 1024 characters"));
                }
                self.spaces();
                key
            };
            let has_value = self.indicator(b':') && (!explicit || self.column() == indent);
            let value = if has_value {
                self.pos += 1;
                self.spaces();
                let at = self.pos;
                if matches!(self.byte(), None | Some(b'\r' | b'\n' | b'#')) {
                    if self.byte() == Some(b'#') {
                        self.comment()?;
                    }
                    self.newline();
                    self.block_space()?;
                    if !self.boundary()
                        && (self.column() > indent
                            || (self.column() == indent && self.indicator(b'-')))
                    {
                        self.block(signed(indent), depth + 1, true)?
                    } else {
                        self.empty(at)?
                    }
                } else {
                    if !explicit && (self.simple_colon().is_some() || self.indicator(b'-')) {
                        return Err(
                            self.error("compact mapping is not allowed after an implicit key")
                        );
                    }
                    self.block(signed(indent), depth + 1, true)?
                }
            } else if explicit {
                self.empty(self.nodes[key.0].span.end)?
            } else {
                return Err(self.error("expected ':' after mapping key"));
            };
            end = end
                .max(self.nodes[key.0].span.end)
                .max(self.nodes[value.0].span.end);
            entries.push(Entry {
                key,
                value,
                span: Span { start, end },
            });
            if self.boundary()
                || self.column() != indent
                || !(self.indicator(b'?') || self.indicator(b':') || self.simple_colon().is_some())
            {
                break;
            }
        }
        self.nodes[id.0].span.end = end;
        self.nodes[id.0].kind = NodeKind::Mapping {
            entries,
            style: CollectionStyle::Block,
        };
        Ok(id)
    }
    fn properties(&mut self, id: NodeId, flow: bool, parent: isize) -> Result<()> {
        while matches!(self.byte(), Some(b'&' | b'!')) {
            let start = self.pos;
            let anchor = self.byte() == Some(b'&');
            self.pos += 1;
            if anchor {
                let name_start = self.pos;
                while self
                    .byte()
                    .is_some_and(|b| !is_space(b) && !b"[]{},".contains(&b))
                {
                    self.pos += self.rest().chars().next().map_or(0, char::len_utf8);
                }
                let name = &self.source[name_start..self.pos];
                if name.is_empty() {
                    return Err(self.error("empty anchor name"));
                }
                if self.nodes[id.0].anchor.is_some() {
                    return Err(self.error("duplicate anchor property"));
                }
                self.nodes[id.0].anchor = Some(Property {
                    span: Span {
                        start,
                        end: self.pos,
                    },
                    value: Cow::Borrowed(name),
                });
                self.anchors.insert(name, id);
            } else {
                if self.nodes[id.0].tag.is_some() {
                    return Err(self.error("duplicate tag property"));
                }
                let value = if self.byte() == Some(b'<') {
                    self.pos += 1;
                    let uri_start = self.pos;
                    while self.byte().is_some_and(|b| b != b'>' && !is_space(b)) {
                        self.pos += self.rest().chars().next().map_or(0, char::len_utf8);
                    }
                    if self.byte() != Some(b'>') || self.pos == uri_start {
                        return Err(self.error("invalid verbatim tag"));
                    }
                    let value = tag_uri(&self.source[uri_start..self.pos])
                        .ok_or_else(|| self.error("invalid tag URI"))?;
                    self.pos += 1;
                    if !complete_tag(&value) {
                        return Err(self.error("invalid verbatim tag URI"));
                    }
                    value
                } else {
                    while self
                        .byte()
                        .is_some_and(|b| !is_space(b) && !b"[]{},".contains(&b))
                    {
                        self.pos += self.rest().chars().next().map_or(0, char::len_utf8);
                    }
                    let raw = &self.source[start..self.pos];
                    if raw == "!" {
                        "!".to_owned()
                    } else {
                        let (handle, suffix) = raw[1..]
                            .find('!')
                            .map_or(("!", &raw[1..]), |n| (&raw[..n + 2], &raw[n + 2..]));
                        let prefix = self
                            .tags
                            .get(handle)
                            .ok_or_else(|| self.error("undefined tag handle"))?;
                        if suffix.is_empty() || suffix.contains('!') {
                            return Err(self.error("invalid tag suffix"));
                        }
                        let decoded =
                            tag_uri(suffix).ok_or_else(|| self.error("invalid tag URI"))?;
                        let expanded = format!("{prefix}{decoded}");
                        if !complete_tag(&expanded) {
                            return Err(self.error("invalid expanded tag URI"));
                        }
                        expanded
                    }
                };
                self.nodes[id.0].tag = Some(Property {
                    span: Span {
                        start,
                        end: self.pos,
                    },
                    value: Cow::Owned(value),
                });
            }
            if self
                .byte()
                .is_some_and(|b| !(is_space(b) || flow && b",]}".contains(&b)))
            {
                return Err(self.error("node properties must be separated by whitespace"));
            }
            if flow {
                self.flow_space(parent)?;
            } else {
                self.spaces();
            }
        }
        Ok(())
    }
    fn inline(&mut self, flow: bool, parent: isize, depth: usize, key: bool) -> Result<NodeId> {
        self.check_depth(depth)?;
        let id = self.empty(self.pos)?;
        self.properties(id, flow, parent)?;
        let kind = self.inline_kind(flow, parent, depth, key)?;
        if matches!(kind, NodeKind::Alias { .. })
            && (self.nodes[id.0].tag.is_some() || self.nodes[id.0].anchor.is_some())
        {
            return Err(self.error("aliases cannot have node properties"));
        }
        self.nodes[id.0].kind = kind;
        self.nodes[id.0].span.end = self.pos;
        Ok(id)
    }
    fn inline_kind(
        &mut self,
        flow: bool,
        parent: isize,
        depth: usize,
        key: bool,
    ) -> Result<NodeKind<'a>> {
        match self.byte() {
            None | Some(b'\r' | b'\n' | b'#' | b',' | b']' | b'}') => {
                Ok(NodeKind::Scalar(Scalar {
                    value: Cow::Borrowed(""),
                    style: ScalarStyle::Plain,
                    span: Span {
                        start: self.pos,
                        end: self.pos,
                    },
                }))
            }
            Some(b':')
                if self
                    .next_byte()
                    .is_none_or(|b| is_space(b) || (flow && b",]}".contains(&b))) =>
            {
                Ok(NodeKind::Scalar(Scalar {
                    value: Cow::Borrowed(""),
                    style: ScalarStyle::Plain,
                    span: Span {
                        start: self.pos,
                        end: self.pos,
                    },
                }))
            }
            Some(b'[') => self.flow_sequence(parent, depth),
            Some(b'{') => self.flow_mapping(parent, depth),
            Some(b'\'' | b'"') => self.quoted(parent, key),
            Some(b'*') => {
                self.pos += 1;
                let start = self.pos;
                while self
                    .byte()
                    .is_some_and(|b| !is_space(b) && !b"[]{},".contains(&b))
                {
                    self.pos += self.rest().chars().next().map_or(0, char::len_utf8);
                }
                let name = &self.source[start..self.pos];
                let target = self
                    .anchors
                    .get(name)
                    .copied()
                    .ok_or_else(|| self.error("unknown alias"))?;
                Ok(NodeKind::Alias { name, target })
            }
            _ => self.plain(flow, parent, key),
        }
    }
    fn flow_sequence(&mut self, parent: isize, depth: usize) -> Result<NodeKind<'a>> {
        self.pos += 1;
        self.flow_space(parent)?;
        let mut items = Vec::new();
        while self.byte() != Some(b']') {
            if self.boundary() {
                return Err(self.error("unterminated flow sequence"));
            }
            let start = self.pos;
            let explicit = self.indicator(b'?');
            if explicit {
                self.pos += 1;
                self.flow_space(parent)?;
            }
            if self.byte() == Some(b',') {
                return Err(self.error("empty flow sequence entry"));
            }
            let key = self.inline(true, parent, depth + 1, false)?;
            self.flow_space(parent)?;
            if self.byte() == Some(b':') || explicit {
                if !explicit
                    && (self.source[start..self.pos].contains(['\r', '\n'])
                        || self.source[start..self.pos].chars().count() > 1024)
                {
                    return Err(self.error("invalid implicit flow pair key"));
                }
                let value = if self.byte() == Some(b':') {
                    self.pos += 1;
                    self.flow_space(parent)?;
                    if matches!(self.byte(), Some(b',' | b']')) {
                        self.empty(self.pos)?
                    } else {
                        self.inline(true, parent, depth + 1, false)?
                    }
                } else {
                    self.empty(self.pos)?
                };
                let end = self.nodes[value.0].span.end;
                let pair = self.empty(start)?;
                self.nodes[pair.0].span.end = end;
                self.nodes[pair.0].kind = NodeKind::Mapping {
                    entries: vec![Entry {
                        key,
                        value,
                        span: Span { start, end },
                    }],
                    style: CollectionStyle::Flow,
                };
                items.push(pair);
                self.flow_space(parent)?;
            } else {
                items.push(key);
            }
            match self.byte() {
                Some(b',') => {
                    self.pos += 1;
                    self.flow_space(parent)?;
                }
                Some(b']') => break,
                _ => return Err(self.error("expected ',' or ']'")),
            }
        }
        self.pos += 1;
        Ok(NodeKind::Sequence {
            items,
            style: CollectionStyle::Flow,
        })
    }
    fn flow_mapping(&mut self, parent: isize, depth: usize) -> Result<NodeKind<'a>> {
        self.pos += 1;
        self.flow_space(parent)?;
        let mut entries = Vec::new();
        while self.byte() != Some(b'}') {
            if self.boundary() {
                return Err(self.error("unterminated flow mapping"));
            }
            let start = self.pos;
            if self.byte() == Some(b',') {
                return Err(self.error("empty flow mapping entry"));
            }
            if self.indicator(b'?') {
                self.pos += 1;
                self.flow_space(parent)?;
            }
            let key = if self.byte() == Some(b':') {
                self.empty(self.pos)?
            } else {
                self.inline(true, parent, depth + 1, false)?
            };
            self.flow_space(parent)?;
            let value = if self.byte() == Some(b':') {
                self.pos += 1;
                self.flow_space(parent)?;
                if matches!(self.byte(), Some(b',' | b'}')) {
                    self.empty(self.pos)?
                } else {
                    self.inline(true, parent, depth + 1, false)?
                }
            } else {
                self.empty(self.pos)?
            };
            entries.push(Entry {
                key,
                value,
                span: Span {
                    start,
                    end: self.nodes[value.0].span.end,
                },
            });
            self.flow_space(parent)?;
            match self.byte() {
                Some(b',') => {
                    self.pos += 1;
                    self.flow_space(parent)?;
                }
                Some(b'}') => break,
                _ => return Err(self.error("expected ',' or '}'")),
            }
        }
        self.pos += 1;
        Ok(NodeKind::Mapping {
            entries,
            style: CollectionStyle::Flow,
        })
    }
    fn plain(&mut self, flow: bool, parent: isize, key: bool) -> Result<NodeKind<'a>> {
        let start = self.pos;
        if let Some(b) = self.byte() {
            if b"@`|>!&*%{}[],\"'".contains(&b)
                || (b"-?:".contains(&b)
                    && self
                        .next_byte()
                        .is_none_or(|b| is_space(b) || (flow && b"[]{},".contains(&b))))
            {
                return Err(self.error("invalid plain scalar start"));
            }
        }
        let mut end = start;
        let mut value: Option<String> = None;
        let mut segment = start;
        loop {
            match self.byte() {
                None => break,
                Some(b'\r' | b'\n') => {
                    if key {
                        break;
                    }
                    let save = self.pos;
                    let mut breaks = 0;
                    while self.newline() {
                        breaks += 1;
                        self.spaces();
                    }
                    if self.document_boundary()
                        || self.byte() == Some(b'#')
                        || signed(self.column()) <= parent
                        || (self.byte() == Some(b':')
                            && self
                                .next_byte()
                                .is_none_or(|b| is_space(b) || (flow && b",[]{}".contains(&b))))
                        || (flow && self.byte().is_some_and(|b| b",[]{}".contains(&b)))
                    {
                        self.pos = save;
                        break;
                    }
                    let text = value.get_or_insert_with(String::new);
                    text.push_str(&self.source[segment..end]);
                    if breaks == 1 {
                        text.push(' ');
                    } else {
                        text.extend(std::iter::repeat_n('\n', breaks - 1));
                    }
                    segment = self.pos;
                }
                Some(b'#')
                    if self.pos == start
                        || self.source.as_bytes()[self.pos - 1].is_ascii_whitespace() =>
                {
                    break;
                }
                Some(b':')
                    if self
                        .next_byte()
                        .is_none_or(|b| is_space(b) || (flow && b",[]{}".contains(&b))) =>
                {
                    break;
                }
                Some(b',' | b'[' | b']' | b'{' | b'}') if flow => break,
                Some(b) => {
                    self.pos += self.rest().chars().next().map_or(0, char::len_utf8);
                    if !matches!(b, b' ' | b'\t') {
                        end = self.pos;
                    }
                }
            }
        }
        self.pos = end;
        let value = value.map_or_else(
            || Cow::Borrowed(&self.source[start..end]),
            |mut text| {
                text.push_str(&self.source[segment..end]);
                Cow::Owned(text)
            },
        );
        if start == end
            && self
                .byte()
                .is_some_and(|b| !is_space(b) && !b"#,:]}".contains(&b))
        {
            return Err(self.error("expected a node"));
        }
        Ok(NodeKind::Scalar(Scalar {
            value,
            style: ScalarStyle::Plain,
            span: Span { start, end },
        }))
    }
    fn quoted(&mut self, parent: isize, key: bool) -> Result<NodeKind<'a>> {
        let start = self.pos;
        let quote = self.byte().unwrap_or(b'"');
        self.pos += 1;
        let content_start = self.pos;
        let mut segment = self.pos;
        let mut output: Option<String> = None;
        loop {
            match self.byte() {
                None => return Err(self.error("unterminated quoted scalar")),
                Some(b) if b == quote => {
                    let end = self.pos;
                    self.pos += 1;
                    if quote == b'\'' && self.byte() == Some(b'\'') {
                        let text = output.get_or_insert_with(String::new);
                        text.push_str(&self.source[segment..end]);
                        text.push('\'');
                        self.pos += 1;
                        segment = self.pos;
                    } else {
                        let value = output.map_or_else(
                            || Cow::Borrowed(&self.source[content_start..end]),
                            |mut s| {
                                s.push_str(&self.source[segment..end]);
                                Cow::Owned(s)
                            },
                        );
                        let style = if quote == b'\'' {
                            ScalarStyle::SingleQuoted
                        } else {
                            ScalarStyle::DoubleQuoted
                        };
                        return Ok(NodeKind::Scalar(Scalar {
                            value,
                            style,
                            span: Span {
                                start,
                                end: self.pos,
                            },
                        }));
                    }
                }
                Some(b'\\') if quote == b'"' => {
                    let text = output.get_or_insert_with(String::new);
                    text.push_str(&self.source[segment..self.pos]);
                    self.pos += 1;
                    if self.newline() {
                        if key {
                            return Err(self.error("implicit keys cannot span lines"));
                        }
                        self.spaces();
                        while self.newline() {
                            text.push('\n');
                            self.spaces();
                        }
                    } else {
                        text.push(self.escape()?);
                    }
                    segment = self.pos;
                }
                Some(b'\r' | b'\n') => {
                    if key {
                        return Err(self.error("implicit keys cannot span lines"));
                    }
                    let text = output.get_or_insert_with(String::new);
                    text.push_str(self.source[segment..self.pos].trim_end_matches([' ', '\t']));
                    let mut breaks = 0;
                    while self.newline() {
                        breaks += 1;
                        self.spaces();
                    }
                    if self.document_boundary() || signed(self.leading_spaces()) <= parent {
                        return Err(self.error("invalid continuation of quoted scalar"));
                    }
                    if breaks == 1 {
                        text.push(' ');
                    } else {
                        text.extend(std::iter::repeat_n('\n', breaks - 1));
                    }
                    segment = self.pos;
                }
                _ => {
                    self.pos += self.rest().chars().next().map_or(0, char::len_utf8);
                }
            }
        }
    }
    fn escape(&mut self) -> Result<char> {
        let b = self.byte().ok_or_else(|| self.error("incomplete escape"))?;
        if !b.is_ascii() {
            return Err(self.error("unknown escape"));
        }
        self.pos += 1;
        let c = match b {
            b'0' => '\0',
            b'a' => '\u{7}',
            b'b' => '\u{8}',
            b't' | b'\t' => '\t',
            b'n' => '\n',
            b'v' => '\u{b}',
            b'f' => '\u{c}',
            b'r' => '\r',
            b'e' => '\u{1b}',
            b' ' => ' ',
            b'"' => '"',
            b'/' => '/',
            b'\\' => '\\',
            b'N' => '\u{85}',
            b'_' => '\u{a0}',
            b'L' => '\u{2028}',
            b'P' => '\u{2029}',
            b'x' | b'u' | b'U' => {
                let count = match b {
                    b'x' => 2,
                    b'u' => 4,
                    _ => 8,
                };
                let mut value = 0u32;
                for _ in 0..count {
                    let digit = self
                        .byte()
                        .and_then(|b| char::from(b).to_digit(16))
                        .ok_or_else(|| self.error("invalid hexadecimal escape"))?;
                    value = (value << 4) | digit;
                    self.pos += 1;
                }
                char::from_u32(value)
                    .ok_or_else(|| self.error("escape is not a Unicode scalar value"))?
            }
            _ => return Err(self.error("unknown escape")),
        };
        Ok(c)
    }
    fn block_scalar(&mut self, parent: isize) -> Result<NodeKind<'a>> {
        let start = self.pos;
        let folded = self.byte() == Some(b'>');
        self.pos += 1;
        let mut chomp = b' ';
        let mut increment = None;
        for _ in 0..2 {
            match self.byte() {
                Some(b @ (b'+' | b'-')) if chomp == b' ' => {
                    chomp = b;
                    self.pos += 1;
                }
                Some(b @ b'1'..=b'9') if increment.is_none() => {
                    increment = Some(usize::from(b - b'0'));
                    self.pos += 1;
                }
                _ => break,
            }
        }
        let before = self.pos;
        self.spaces();
        if self.byte() == Some(b'#') && before != self.pos {
            self.comment()?;
        }
        if self.byte().is_some() && !self.newline() {
            return Err(self.error("invalid block scalar header"));
        }
        let base = usize::try_from(parent).unwrap_or(0);
        let mut indent = increment.map(|n| base + n);
        let mut lines: Vec<(&str, bool, bool)> = Vec::new();
        let mut end = self.pos;
        let mut leading_indent = 0;
        while self.byte().is_some() {
            let line_start = self.pos;
            while self.byte() == Some(b' ') {
                self.pos += 1;
            }
            let spaces = self.pos - line_start;
            let line_end = self.line_end();
            let empty = self.pos == line_end;
            if !empty {
                let required =
                    *indent.get_or_insert(spaces.max(usize::try_from(parent + 1).unwrap_or(0)));
                if spaces < required || self.marker("---") || self.marker("...") {
                    if self.byte() == Some(b'\t') {
                        return Err(self.error("tab in block scalar indentation"));
                    }
                    self.pos = line_start;
                    break;
                }
                if lines.iter().all(|(s, _, _)| s.is_empty()) && leading_indent > required {
                    return Err(self.error("leading empty line is over-indented"));
                }
            } else if indent.is_none() {
                leading_indent = leading_indent.max(spaces);
            }
            let text_start = line_start + indent.unwrap_or(spaces).min(spaces);
            let text = &self.source[text_start..line_end];
            self.pos = line_end;
            let newline = self.newline() || self.byte().is_none();
            lines.push((text, text.starts_with([' ', '\t']), newline));
            end = self.pos;
        }
        let value = fold_block_lines(&lines, folded, chomp);
        self.block_space()?;
        let style = if folded {
            ScalarStyle::Folded
        } else {
            ScalarStyle::Literal
        };
        Ok(NodeKind::Scalar(Scalar {
            value: Cow::Owned(value),
            style,
            span: Span { start, end },
        }))
    }
}

fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | b'\n')
}
fn valid_handle(s: &str) -> bool {
    s == "!"
        || (s.starts_with('!')
            && s.ends_with('!')
            && s[1..s.len() - 1]
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-'))
}
fn tag_uri(s: &str) -> Option<String> {
    let mut bytes = s.bytes();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            if !bytes.next()?.is_ascii_hexdigit() || !bytes.next()?.is_ascii_hexdigit() {
                return None;
            }
        } else if !b.is_ascii_alphanumeric() && !b"-#;/?:@&=+$,_.!~*'()[]".contains(&b) {
            return None;
        }
    }
    // URI escapes are part of tag identity, not scalar-style escapes (YAML 1.2.2 §5.6).
    Some(s.to_owned())
}

fn complete_tag(s: &str) -> bool {
    if let Some(local) = s.strip_prefix('!') {
        return !local.is_empty();
    }
    let Some((scheme, _)) = s.split_once(':') else {
        return false;
    };
    scheme
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphabetic)
        && scheme
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"+.-".contains(&b))
}

fn signed(value: usize) -> isize {
    isize::try_from(value).unwrap_or(isize::MAX)
}

fn fold_block_lines(lines: &[(&str, bool, bool)], folded: bool, chomp: u8) -> String {
    let mut value = String::new();
    for (i, &(text, more, newline)) in lines.iter().enumerate() {
        value.push_str(text);
        if !newline {
            continue;
        }
        if folded && !text.is_empty() && !more {
            if let Some(&(next, next_more, _)) = lines.get(i + 1) {
                if !next.is_empty() && !next_more {
                    value.push(' ');
                    continue;
                }
                if next.is_empty()
                    && lines[i + 1..]
                        .iter()
                        .find(|(s, _, _)| !s.is_empty())
                        .is_some_and(|(_, more, _)| !more)
                {
                    continue;
                }
            }
        }
        value.push('\n');
    }
    if chomp != b'+' {
        let had_break = value.ends_with('\n');
        value.truncate(value.trim_end_matches('\n').len());
        if chomp != b'-' && had_break && !value.is_empty() {
            value.push('\n');
        }
    }
    value
}

// JSON-compatible quoted scalars admit non-C0 characters excluded elsewhere.
fn validate_characters(stream: &Stream<'_>) -> Result<()> {
    let mut restricted = stream.source().char_indices().filter(|&(_, c)| {
        c == '\u{feff}' || !matches!(c, '\t' | '\r' | '\n' | '\u{20}'..='\u{7e}' | '\u{85}' | '\u{a0}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
    });
    let Some(first) = restricted.next() else {
        return Ok(());
    };
    let mut allowed: Vec<_> = stream
        .nodes()
        .iter()
        .filter_map(|node| match &node.kind {
            NodeKind::Scalar(s)
                if matches!(
                    s.style,
                    ScalarStyle::SingleQuoted | ScalarStyle::DoubleQuoted
                ) =>
            {
                Some(s.span)
            }
            _ => None,
        })
        .chain(stream.byte_order_marks().iter().copied())
        .collect();
    allowed.sort_unstable_by_key(|span| span.start);
    let mut index = 0;
    for (offset, c) in std::iter::once(first).chain(restricted) {
        while allowed.get(index).is_some_and(|span| span.end <= offset) {
            index += 1;
        }
        if c < ' '
            || !allowed
                .get(index)
                .is_some_and(|span| span.start <= offset && offset < span.end)
        {
            return Err(Error {
                span: Span {
                    start: offset,
                    end: offset + c.len_utf8(),
                },
                message: "character is not allowed in this context",
            });
        }
    }
    Ok(())
}
