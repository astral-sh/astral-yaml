use std::{borrow::Cow, cell::Cell, collections::HashMap};

use crate::{
    CollectionStyle, Comment, Directive, Document, Entry, Error, Node, NodeId, NodeKind, Property,
    Scalar, ScalarStyle, Span, Stream,
};

type Result<T> = std::result::Result<T, Error>;

/// Limits on input size, syntax depth, and syntax elements.
#[derive(Clone, Copy, Debug)]
pub struct ParseOptions {
    /// Maximum number of nodes on a path from a document root, between 1 and 128.
    /// A scalar root uses one level. A sequence containing a scalar uses two.
    /// An alias is a leaf.
    pub max_depth: usize,

    /// Maximum number of nodes, comments, directives, BOMs, and end markers in a stream.
    pub max_nodes: usize,

    /// Maximum UTF-8 input length in bytes. The same limit applies separately to the
    /// total length of expanded shorthand tags in the stream.
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

impl ParseOptions {
    pub(crate) fn check_input(self, bytes: usize) -> Result<()> {
        let error = |message| Error {
            span: Span { start: 0, end: 0 },
            message,
        };

        if !(1..=128).contains(&self.max_depth) {
            return Err(error("max_depth must be between 1 and 128"));
        }
        if bytes > self.max_input_bytes {
            return Err(error("input byte limit exceeded"));
        }
        Ok(())
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
    let mut remaining_tag_bytes = options.max_input_bytes;
    parse_with_tag_budget(source, options, &mut remaining_tag_bytes)
}

pub(crate) fn parse_with_tag_budget<'a>(
    source: &'a str,
    options: ParseOptions,
    remaining_tag_bytes: &mut usize,
) -> Result<Stream<'a>> {
    options.check_input(source.len())?;

    let stream = Parser {
        source,
        pos: 0,
        options,
        // Reserve for short mapping keys and values, while bounding unused
        // capacity for inputs dominated by a few large scalars.
        nodes: Vec::with_capacity((source.len() / 8).min(1024).min(options.max_nodes)),
        documents: Vec::new(),
        comments: Vec::new(),
        anchors: HashMap::new(),
        tags: HashMap::new(),
        remaining_tag_bytes,
        byte_order_marks: Vec::new(),
        document_end_markers: Vec::new(),
        elements: 0,
        deepest_node: 0,
        line_start: 0,
        line_end_cache: Cell::new(None),
        colon_cache: Cell::new(None),
    }
    .stream()?;

    validate_characters(&stream)?;
    Ok(stream)
}

struct Parser<'a, 'budget> {
    source: &'a str,
    pos: usize,
    options: ParseOptions,
    nodes: Vec<Node<'a>>,
    documents: Vec<Document<'a>>,
    comments: Vec<Comment<'a>>,
    anchors: HashMap<&'a str, NodeId>,
    tags: HashMap<&'a str, String>,
    remaining_tag_bytes: &'budget mut usize,
    byte_order_marks: Vec<Span>,
    document_end_markers: Vec<Span>,
    elements: usize,
    deepest_node: usize,
    line_start: usize,
    line_end_cache: Cell<Option<Span>>,
    colon_cache: Cell<Option<KeyLookahead>>,
}

#[derive(Clone, Copy)]
struct KeyLookahead {
    start: usize,
    colon: Option<usize>,
    plain_end: Option<usize>,
}

/// The grammar following a block collection's entry indicator.
#[derive(Clone, Copy)]
enum BlockEntry {
    SequenceItem,
    ExplicitMapping,
    ImplicitMappingValue,
}

impl<'a> Parser<'a, '_> {
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

    #[inline]
    fn column(&self) -> usize {
        self.pos - self.line_start
    }

    fn line_end(&self) -> usize {
        if let Some(line) = self.line_end_cache.get() {
            if line.start <= self.pos && self.pos <= line.end {
                return line.end;
            }
        }

        let end = memchr::memchr2(b'\r', b'\n', &self.source.as_bytes()[self.pos..])
            .map_or(self.source.len(), |n| self.pos + n);
        self.line_end_cache.set(Some(Span {
            start: self.line_start,
            end,
        }));
        end
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
            }
            Some(b'\n') => self.pos += 1,
            _ => return false,
        }

        self.line_start = self.pos;
        true
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

        if crossed_line && (self.marker("---") || self.marker("...")) {
            return Err(self.error("document marker inside flow collection"));
        }

        if crossed_line && self.byte().is_some() && signed(self.leading_spaces()) <= parent {
            return Err(self.error("flow continuation must be indented"));
        }

        Ok(())
    }

    #[inline]
    fn leading_spaces(&self) -> usize {
        self.source[self.line_start..]
            .bytes()
            .take_while(|&b| b == b' ')
            .count()
    }

    #[inline]
    fn document_boundary(&self) -> bool {
        match self.byte() {
            None => true,
            Some(b'-') => self.marker("---"),
            Some(b'.') => self.marker("..."),
            Some(0xef) => self.rest().starts_with('\u{feff}') && self.column() == 0,
            Some(_) => false,
        }
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

    #[inline]
    fn marker(&self, marker: &str) -> bool {
        self.rest().starts_with(marker)
            && self.column() == 0
            && self
                .source
                .as_bytes()
                .get(self.pos + marker.len())
                .is_none_or(|b| is_space(*b))
    }

    fn boundary(&self) -> bool {
        self.document_boundary() || (self.byte() == Some(b'%') && self.column() == 0)
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

    fn check_depth(&mut self, depth: usize) -> Result<()> {
        self.deepest_node = self.deepest_node.max(depth);
        if depth >= self.options.max_depth {
            Err(self.error("nesting limit exceeded"))
        } else {
            Ok(())
        }
    }

    fn document_prefixes(&mut self) -> Result<()> {
        loop {
            self.block_space()?;
            if self.column() != 0 || !self.rest().starts_with('\u{feff}') {
                return Ok(());
            }

            self.reserve_element()?;
            self.byte_order_marks.push(Span {
                start: self.pos,
                end: self.pos + 3,
            });
            self.pos += 3;
            self.line_start = self.pos;
        }
    }

    fn end_marker(&mut self) -> Result<Span> {
        self.reserve_element()?;
        let span = Span {
            start: self.pos,
            end: self.pos + 3,
        };

        self.document_end_markers.push(span);
        self.pos += 3;
        self.finish_line()?;
        Ok(span)
    }

    fn stream(mut self) -> Result<Stream<'a>> {
        let mut previous_ended = true;

        loop {
            self.document_prefixes()?;
            if self.byte().is_none() {
                break;
            }

            if self.marker("...") {
                self.end_marker()?;
                previous_ended = true;
                continue;
            }

            self.anchors.clear();
            self.tags.clear();

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
                Some(self.end_marker()?)
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
            document_end_markers: self.document_end_markers,
        })
    }

    fn directives(&mut self) -> Result<Vec<Directive<'a>>> {
        let mut directives = Vec::new();
        let mut version = false;

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

            let mut words = args.split_ascii_whitespace();

            match name {
                "YAML" => {
                    if version {
                        return Err(self.error("duplicate YAML directive"));
                    }

                    let valid = words
                        .next()
                        .and_then(|word| word.split_once('.'))
                        .is_some_and(|(major, minor)| {
                            major.trim_start_matches('0') == "1"
                                && !minor.is_empty()
                                && minor.bytes().all(|b| b.is_ascii_digit())
                        })
                        && words.next().is_none();
                    if !valid {
                        return Err(self.error("invalid YAML version directive"));
                    }

                    // New minor versions use the 1.2 grammar. 1.1 schema behavior is
                    // not enabled.
                    version = true;
                }
                "TAG" => {
                    let (Some(handle), Some(prefix), None) =
                        (words.next(), words.next(), words.next())
                    else {
                        return Err(self.error("invalid TAG directive"));
                    };
                    if !valid_handle(handle) {
                        return Err(self.error("invalid TAG directive"));
                    }

                    if self.tags.contains_key(handle) {
                        return Err(self.error("duplicate TAG directive"));
                    }

                    let prefix = tag_uri(prefix).ok_or_else(|| self.error("invalid tag URI"))?;
                    if prefix.starts_with([',', '[', ']']) {
                        return Err(self.error("invalid global tag prefix"));
                    }

                    self.tags.insert(handle, prefix);
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
        if prefix.contains('\t') {
            let block_collection = self.indicator(b'-')
                || self.indicator(b'?')
                || self.indicator(b':')
                || self.simple_colon().is_some();
            if block_collection
                || (prefix.bytes().all(is_space) && signed(self.leading_spaces()) <= parent)
            {
                return Err(self.error("tab in block indentation"));
            }
        }

        // A property preceding a mapping key belongs to that key, not the mapping.
        if self.indicator(b'-') {
            return self.block_sequence(indent, depth, existing);
        }

        if self.indicator(b'?') || self.indicator(b':') || self.simple_colon().is_some() {
            return self.block_mapping(indent, depth, existing);
        }

        if existing.is_none() && !matches!(self.byte(), Some(b'!' | b'&' | b'|' | b'>')) {
            let id = self.inline(false, parent, depth)?;
            self.finish_line()?;
            return Ok(id);
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
                // Properties and content belong to the same node.
                return self.block_into(parent, depth, indentless, Some(id));
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
            _ => self.inline_kind(false, parent, depth)?,
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

    fn block_entry(&mut self, indent: usize, depth: usize, entry: BlockEntry) -> Result<NodeId> {
        self.pos += 1;
        self.spaces();

        let at = self.pos;
        let indentless = !matches!(entry, BlockEntry::SequenceItem);
        if matches!(self.byte(), None | Some(b'\r' | b'\n' | b'#')) {
            if self.byte() == Some(b'#') {
                self.comment()?;
            }

            self.newline();
            self.block_space()?;
            let nested = !self.boundary()
                && (self.column() > indent
                    || (indentless && self.column() == indent && self.indicator(b'-')));
            if !nested {
                self.check_depth(depth)?;
                return self.empty(at);
            }
        } else if matches!(entry, BlockEntry::ImplicitMappingValue) {
            return self.implicit_mapping_value(indent, depth);
        }

        self.block(signed(indent), depth, indentless)
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
            let item = self.block_entry(indent, depth + 1, BlockEntry::SequenceItem)?;
            end = end.max(self.nodes[item.0].span.end);
            items.push(item);
            if self.boundary() || self.column() != indent || !self.indicator(b'-') {
                break;
            }

            if self.source[self.line_start..self.pos].contains('\t') {
                return Err(self.error("tab in block indentation"));
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
        if let Some(cached) = self
            .colon_cache
            .get()
            .filter(|cached| cached.start == self.pos)
        {
            return cached.colon;
        }

        let mut lookahead = KeyLookahead {
            start: self.pos,
            colon: None,
            plain_end: None,
        };
        lookahead.colon = self.find_simple_colon(&mut lookahead);
        self.colon_cache.set(Some(lookahead));
        lookahead.colon
    }

    fn find_simple_colon(&self, lookahead: &mut KeyLookahead) -> Option<usize> {
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
        if plain {
            let content_start = i;
            let line_end = self.line_end();

            // Plain keys cannot open quoted or flow syntax. Skip their content
            // in runs, checking only separators and comments on this line.
            loop {
                let remaining = &bytes[i..line_end];
                i += memchr::memchr2(b':', b'#', remaining).unwrap_or(remaining.len());

                let colon = match bytes.get(i) {
                    None | Some(b'\r' | b'\n') => None,
                    Some(b'#') if i == self.pos || is_space(bytes[i - 1]) => None,
                    Some(b':') if bytes.get(i + 1).is_none_or(|b| is_space(*b)) => Some(i),
                    _ => {
                        i += 1;
                        continue;
                    }
                };

                if content_start == self.pos {
                    lookahead.plain_end = Some(i);
                }

                return colon;
            }
        }

        self.find_complex_key_colon(i)
    }

    fn find_complex_key_colon(&self, start: usize) -> Option<usize> {
        let bytes = self.source.as_bytes();
        let mut i = start;
        let mut quote = None;
        let mut level = 0usize;
        // Quotes only start a quoted scalar at a node boundary. Elsewhere they are
        // ordinary characters in a plain scalar.
        let mut plain = false;
        let mut json_node = false;

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
                    if q == b'\'' && bytes.get(i + 1) == Some(&q) {
                        i += 2;
                        continue;
                    }

                    quote = None;
                    plain = true;
                    json_node = true;
                }
            } else {
                match b {
                    b'\'' | b'"' if !plain => quote = Some(b),
                    b'[' | b'{' => {
                        level += 1;
                        plain = false;
                        json_node = false;
                    }
                    b']' | b'}' => {
                        level = level.saturating_sub(1);
                        plain = true;
                        json_node = true;
                    }
                    b',' if level > 0 => {
                        plain = false;
                        json_node = false;
                    }
                    b'#' if i == self.pos || is_space(bytes[i - 1]) => break,
                    b':' if level == 0 && bytes.get(i + 1).is_none_or(|b| is_space(*b)) => {
                        return Some(i);
                    }
                    b':' if level > 0
                        && (json_node
                            || bytes
                                .get(i + 1)
                                .is_none_or(|b| is_space(*b) || b",]}".contains(b))) =>
                    {
                        plain = false;
                        json_node = false;
                    }
                    b'?' if !plain && bytes.get(i + 1).is_none_or(|b| is_space(*b)) => {}
                    b'!' | b'&' | b'*' if !plain => {
                        let alias = b == b'*';
                        let verbatim_tag = b == b'!' && bytes.get(i + 1) == Some(&b'<');
                        i += 1;
                        while bytes.get(i).is_some_and(|b| {
                            !is_space(*b)
                                && if verbatim_tag {
                                    *b != b'>'
                                } else {
                                    !b"[]{},".contains(b)
                                }
                        }) {
                            i += 1;
                        }
                        if verbatim_tag && bytes.get(i) == Some(&b'>') {
                            i += 1;
                        }
                        plain = alias;
                        continue;
                    }
                    b' ' | b'\t' => {}
                    _ => {
                        plain = true;
                        json_node = false;
                    }
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
                self.block_entry(indent, depth + 1, BlockEntry::ExplicitMapping)?
            } else if self.indicator(b':') {
                self.empty(self.pos)?
            } else {
                let before = self.pos;
                let key = self.inline(false, signed(indent), depth + 1)?;
                self.spaces();

                let key_source = &self.source[before..self.pos];
                if (key_source.len() > 1024 && key_source.chars().count() > 1024)
                    || self.line_start > before
                {
                    return Err(self
                        .error("an implicit key must fit on one line of at most 1024 characters"));
                }

                key
            };

            let has_value = self.indicator(b':') && (!explicit || self.column() == indent);
            let value = if has_value {
                let entry = if explicit {
                    BlockEntry::ExplicitMapping
                } else {
                    BlockEntry::ImplicitMappingValue
                };
                self.block_entry(indent, depth + 1, entry)?
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

            if self.source[self.line_start..self.pos].contains('\t') {
                return Err(self.error("tab in block indentation"));
            }
        }

        self.nodes[id.0].span.end = end;
        self.nodes[id.0].kind = NodeKind::Mapping {
            entries,
            style: CollectionStyle::Block,
        };

        Ok(id)
    }

    #[inline]
    fn implicit_mapping_value(&mut self, indent: usize, depth: usize) -> Result<NodeId> {
        if self.simple_colon().is_some() || self.indicator(b'-') || self.indicator(b'?') {
            return Err(self.error("compact mapping is not allowed after an implicit key"));
        }

        if matches!(self.byte(), Some(b'!' | b'&' | b'|' | b'>' | b'?' | b':')) {
            return self.block(signed(indent), depth, true);
        }

        // Lookahead has ruled out a block collection. Parse the inline
        // value without repeating block dispatch.
        let value = self.inline(false, signed(indent), depth)?;
        self.finish_line()?;
        Ok(value)
    }

    fn tag_prefix(&self, handle: &str) -> Option<&str> {
        self.tags.get(handle).map(String::as_str).or(match handle {
            "!" => Some("!"),
            "!!" => Some("tag:yaml.org,2002:"),
            _ => None,
        })
    }

    fn shorthand_tag(&mut self, raw: &str) -> Result<String> {
        if raw == "!" {
            return Ok("!".to_owned());
        }
        let (handle, suffix) = raw[1..]
            .find('!')
            .map_or(("!", &raw[1..]), |n| (&raw[..n + 2], &raw[n + 2..]));
        let prefix = self
            .tag_prefix(handle)
            .ok_or_else(|| self.error("undefined tag handle"))?;
        if suffix.is_empty() || suffix.contains('!') {
            return Err(self.error("invalid tag suffix"));
        }

        let decoded = tag_uri(suffix).ok_or_else(|| self.error("invalid tag URI"))?;
        let remaining = self
            .remaining_tag_bytes
            .checked_sub(prefix.len() + decoded.len())
            .ok_or_else(|| self.error("expanded tag byte limit exceeded"))?;
        let expanded = format!("{prefix}{decoded}");
        *self.remaining_tag_bytes = remaining;
        if !complete_tag(&expanded) {
            return Err(self.error("invalid expanded tag URI"));
        }

        Ok(expanded)
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
                    self.shorthand_tag(raw)?
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

    #[inline]
    fn inline(&mut self, flow: bool, parent: isize, depth: usize) -> Result<NodeId> {
        self.check_depth(depth)?;

        // Scalars and aliases have no children or properties to register before
        // parsing. Append their completed node without an empty placeholder.
        if !matches!(self.byte(), Some(b'!' | b'&' | b'[' | b'{')) {
            self.reserve_element()?;
            let start = self.pos;
            let kind = self.inline_kind(flow, parent, depth)?;
            let id = NodeId(self.nodes.len());
            self.nodes.push(Node {
                span: Span {
                    start,
                    end: self.pos,
                },
                anchor: None,
                tag: None,
                kind,
            });
            return Ok(id);
        }

        let id = self.empty(self.pos)?;
        self.properties(id, flow, parent)?;
        let kind = self.inline_kind(flow, parent, depth)?;
        if matches!(kind, NodeKind::Alias { .. })
            && (self.nodes[id.0].tag.is_some() || self.nodes[id.0].anchor.is_some())
        {
            return Err(self.error("aliases cannot have node properties"));
        }

        self.nodes[id.0].kind = kind;
        self.nodes[id.0].span.end = self.pos;
        Ok(id)
    }

    fn inline_kind(&mut self, flow: bool, parent: isize, depth: usize) -> Result<NodeKind<'a>> {
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
            Some(b'\'' | b'"') => self.quoted(parent),
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
            _ => self.plain(flow, parent),
        }
    }

    fn flow_value_start(&mut self, key: NodeId) -> Result<()> {
        self.pos += 1;
        let json_key = matches!(
            &self.nodes[key.0].kind,
            NodeKind::Sequence { .. }
                | NodeKind::Mapping { .. }
                | NodeKind::Scalar(Scalar {
                    style: ScalarStyle::SingleQuoted | ScalarStyle::DoubleQuoted,
                    ..
                })
        );
        if !json_key
            && self
                .byte()
                .is_some_and(|b| !is_space(b) && !b",]}".contains(&b))
        {
            return Err(self.error("a flow value after a YAML key must be separated by whitespace"));
        }

        Ok(())
    }

    fn flow_pair(
        &mut self,
        start: usize,
        key: NodeId,
        closing: u8,
        parent: isize,
        depth: usize,
    ) -> Result<Entry> {
        let value = if self.byte() == Some(b':') {
            self.flow_value_start(key)?;
            self.flow_space(parent)?;
            if self
                .byte()
                .is_some_and(|byte| byte == b',' || byte == closing)
            {
                self.empty(self.pos)?
            } else {
                self.inline(true, parent, depth)?
            }
        } else {
            self.empty(self.pos)?
        };

        Ok(Entry {
            key,
            value,
            span: Span {
                start,
                end: self.nodes[value.0].span.end,
            },
        })
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

            if !explicit && self.byte() == Some(b',') {
                return Err(self.error("empty flow sequence entry"));
            }

            // A following colon makes this item a key one level deeper. Track its
            // deepest descendant so recognizing the pair does not require a second walk.
            let previous_deepest = std::mem::take(&mut self.deepest_node);
            let key = self.inline(true, parent, depth + 1)?;

            self.flow_space(parent)?;
            if self.byte() == Some(b':') || explicit {
                self.check_depth(self.deepest_node + 1)?;
                if !explicit
                    && (self.source[start..self.pos].contains(['\r', '\n'])
                        || self.source[start..self.pos].chars().count() > 1024)
                {
                    return Err(self.error("invalid implicit flow pair key"));
                }

                let entry = self.flow_pair(start, key, b']', parent, depth + 2)?;
                let pair = self.empty(start)?;
                self.nodes[pair.0].span.end = entry.span.end;
                self.nodes[pair.0].kind = NodeKind::Mapping {
                    entries: vec![entry],
                    style: CollectionStyle::Flow,
                };

                items.push(pair);

                self.flow_space(parent)?;
            } else {
                items.push(key);
            }
            self.deepest_node = self.deepest_node.max(previous_deepest);

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

            let key = self.inline(true, parent, depth + 1)?;

            self.flow_space(parent)?;
            entries.push(self.flow_pair(start, key, b'}', parent, depth + 1)?);

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

    // Mapping lookahead already scanned the first line of a plain scalar,
    // whether it found a key separator or an ordinary value. Advance to its
    // delimiter and return the end of the content, excluding trailing spaces.
    fn reuse_plain_lookahead(&mut self) -> usize {
        let start = self.pos;
        let Some(scanned_end) = self
            .colon_cache
            .get()
            .filter(|cached| cached.start == start)
            .and_then(|cached| cached.plain_end)
        else {
            return start;
        };

        self.pos = scanned_end;
        start
            + self.source[start..scanned_end]
                .trim_end_matches([' ', '\t'])
                .len()
    }

    fn plain(&mut self, flow: bool, parent: isize) -> Result<NodeKind<'a>> {
        let start = self.pos;
        let b = self.byte().expect("plain scalar has a first byte");
        if b"@`|>!&*%{}[],\"'".contains(&b)
            || (b"-?:".contains(&b)
                && self
                    .next_byte()
                    .is_none_or(|b| is_space(b) || (flow && b"[]{},".contains(&b))))
        {
            return Err(self.error("invalid plain scalar start"));
        }

        let mut end = start;
        let mut value: Option<String> = None;
        let mut segment = start;

        if !flow {
            end = self.reuse_plain_lookahead();
            if self.pos > start && self.byte() == Some(b':') {
                self.pos = end;
                return Ok(NodeKind::Scalar(Scalar {
                    value: Cow::Borrowed(&self.source[start..end]),
                    style: ScalarStyle::Plain,
                    span: Span { start, end },
                }));
            }
        }

        loop {
            match self.byte() {
                None => break,
                Some(b'\r' | b'\n') => {
                    let save = self.pos;
                    let saved_line_start = self.line_start;
                    let mut breaks = 0;

                    while self.newline() {
                        breaks += 1;
                        self.spaces();
                    }

                    if self.document_boundary()
                        || self.byte() == Some(b'#')
                        || signed(self.leading_spaces()) <= parent
                        || (self.byte() == Some(b':')
                            && self
                                .next_byte()
                                .is_none_or(|b| is_space(b) || (flow && b",[]{}".contains(&b))))
                        || (flow && self.byte().is_some_and(|b| b",[]{}".contains(&b)))
                    {
                        self.pos = save;
                        self.line_start = saved_line_start;
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
                Some(b' ' | b'\t') => self.pos += 1,
                Some(_) => {
                    // Consume a run of content bytes. Every stopping character is
                    // ASCII, so the next position is also a UTF-8 boundary.
                    self.pos += 1;
                    let remaining = &self.source.as_bytes()[self.pos..];
                    let length = remaining
                        .iter()
                        .position(|&b| {
                            matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'#' | b':')
                                || (flow && b",[]{}".contains(&b))
                        })
                        .unwrap_or(remaining.len());
                    self.pos += length;
                    end = self.pos;
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
        Ok(NodeKind::Scalar(Scalar {
            value,
            style: ScalarStyle::Plain,
            span: Span { start, end },
        }))
    }

    fn quoted_continuation(&self, parent: isize) -> Result<()> {
        // A BOM inside a quoted scalar is content, not a document boundary.
        if self.byte().is_none()
            || self.marker("---")
            || self.marker("...")
            || signed(self.leading_spaces()) <= parent
        {
            return Err(self.error("invalid continuation of quoted scalar"));
        }

        Ok(())
    }

    fn quoted(&mut self, parent: isize) -> Result<NodeKind<'a>> {
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
                        self.spaces();

                        while self.newline() {
                            text.push('\n');
                            self.spaces();
                        }

                        self.quoted_continuation(parent)?;
                    } else {
                        text.push(self.escape()?);
                    }

                    segment = self.pos;
                }
                Some(b'\r' | b'\n') => {
                    let text = output.get_or_insert_with(String::new);
                    text.push_str(self.source[segment..self.pos].trim_end_matches([' ', '\t']));
                    let mut breaks = 0;

                    while self.newline() {
                        breaks += 1;
                        self.spaces();
                    }

                    self.quoted_continuation(parent)?;

                    if breaks == 1 {
                        text.push(' ');
                    } else {
                        text.extend(std::iter::repeat_n('\n', breaks - 1));
                    }

                    segment = self.pos;
                }
                _ => {
                    // Quotes, escapes, and line breaks are ASCII, so a byte
                    // scan also stops at a UTF-8 character boundary.
                    let remaining = &self.source.as_bytes()[self.pos..self.line_end()];
                    let delimiter = if quote == b'"' {
                        memchr::memchr2(quote, b'\\', remaining)
                    } else {
                        memchr::memchr(quote, remaining)
                    };
                    self.pos += delimiter.unwrap_or(remaining.len());
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

                let mut value = self.hex_escape(count)?;

                // JSON encodes non-BMP characters as two adjacent \u escapes.
                if b == b'u' && (0xd800..=0xdbff).contains(&value) {
                    if !self.source[self.pos..].starts_with("\\u") {
                        return Err(self.error("expected a low-surrogate escape"));
                    }

                    self.pos += 2;
                    let low = self.hex_escape(4)?;
                    if !(0xdc00..=0xdfff).contains(&low) {
                        return Err(self.error("expected a low surrogate"));
                    }

                    value = 0x10000 + ((value - 0xd800) << 10) + (low - 0xdc00);
                }

                char::from_u32(value)
                    .ok_or_else(|| self.error("escape is not a Unicode scalar value"))?
            }
            _ => return Err(self.error("unknown escape")),
        };

        Ok(c)
    }

    fn hex_escape(&mut self, count: usize) -> Result<u32> {
        let mut value = 0u32;

        for _ in 0..count {
            let digit = self
                .byte()
                .and_then(|b| char::from(b).to_digit(16))
                .ok_or_else(|| self.error("invalid hexadecimal escape"))?;
            value = (value << 4) | digit;
            self.pos += 1;
        }

        Ok(value)
    }

    fn block_scalar(&mut self, parent: isize) -> Result<NodeKind<'a>> {
        let start = self.pos;
        let folded = self.byte() == Some(b'>');
        self.pos += 1;
        let mut chomp = Chomping::Clip;
        let mut increment = None;

        for _ in 0..2 {
            match self.byte() {
                Some(b'+') if chomp == Chomping::Clip => {
                    chomp = Chomping::Keep;
                    self.pos += 1;
                }
                Some(b'-') if chomp == Chomping::Clip => {
                    chomp = Chomping::Strip;
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

        let minimum = usize::try_from(parent + 1).expect("parent indentation starts at -1");
        let mut indent = increment.map(|n| minimum + (n - 1));
        let mut content = BlockScalarValue::default();
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
                let required = *indent.get_or_insert(spaces.max(minimum));
                if spaces < required || self.document_boundary() {
                    if self.byte() == Some(b'\t') {
                        return Err(self.error("tab in block scalar indentation"));
                    }

                    self.pos = line_start;
                    break;
                }

                if content.text.is_empty() && leading_indent > required {
                    return Err(self.error("leading empty line is over-indented"));
                }
            } else if indent.is_none() {
                leading_indent = leading_indent.max(spaces);
            }

            let text_start = line_start + indent.unwrap_or(spaces).min(spaces);
            let text = &self.source[text_start..line_end];
            content.push(text, folded);
            self.pos = line_end;
            self.newline();
            end = self.pos;
        }

        let value = content.finish(chomp);
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

#[derive(Clone, Copy, Eq, PartialEq)]
enum Chomping {
    Clip,
    Keep,
    Strip,
}

/// Hold line breaks until the next content line determines how they fold,
/// or the chomping indicator determines how many to keep at the end.
#[derive(Default)]
struct BlockScalarValue {
    text: String,
    breaks: usize,
    previous_ordinary: bool,
}

impl BlockScalarValue {
    fn push(&mut self, line: &str, folded: bool) {
        if !line.is_empty() {
            let ordinary = !line.starts_with([' ', '\t']);
            if folded && self.previous_ordinary && ordinary {
                self.breaks -= 1;
                if self.breaks == 0 {
                    self.text.push(' ');
                }
            }
            self.text.extend(std::iter::repeat_n('\n', self.breaks));
            self.text.push_str(line);
            self.previous_ordinary = ordinary;
            self.breaks = 0;
        }
        self.breaks += 1;
    }

    fn finish(mut self, chomp: Chomping) -> String {
        match chomp {
            Chomping::Keep => self.text.extend(std::iter::repeat_n('\n', self.breaks)),
            Chomping::Clip if !self.text.is_empty() => self.text.push('\n'),
            Chomping::Clip | Chomping::Strip => {}
        }
        self.text
    }
}

// JSON-compatible quoted scalars admit non-C0 characters excluded elsewhere.
fn validate_characters(stream: &Stream<'_>) -> Result<()> {
    // A full reduction lets the compiler vectorize this common ASCII check.
    // Unicode and restricted bytes still receive the contextual checks below.
    #[allow(clippy::needless_bitwise_bool)]
    let ascii = stream.source().bytes().fold(true, |valid, byte| {
        valid
            & ((b' '..=b'~').contains(&byte) | (byte == b'\t') | (byte == b'\r') | (byte == b'\n'))
    });
    if ascii {
        return Ok(());
    }

    let mut restricted = stream.source().char_indices().filter(|&(_, c)| {
        c == '\u{feff}'
            || !matches!(
                c,
                '\t' | '\r' | '\n' | '\u{20}'..='\u{7e}' | '\u{85}'
                    | '\u{a0}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}'
                    | '\u{10000}'..='\u{10ffff}'
            )
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
