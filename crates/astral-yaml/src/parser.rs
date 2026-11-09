use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;

use crate::{
    CollectionStyle, Comment, Directive, Document, Entry, Error, Node, NodeId, NodeKind, Property,
    Scalar, ScalarStyle, Span, Stream,
};

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum YamlVersion {
    V1_0,
    V1_1,
    V1_2,
    HigherMinor,
}

impl YamlVersion {
    pub(crate) fn parse(arguments: &str) -> Option<Self> {
        let mut words = arguments.split_ascii_whitespace();
        let (major, minor) = words.next()?.split_once('.')?;
        if words.next().is_some()
            || major.trim_start_matches('0') != "1"
            || minor.is_empty()
            || !minor.bytes().all(|b| b.is_ascii_digit())
        {
            return None;
        }

        // Version components can have leading zeros and no upper bound.
        Some(match minor.trim_start_matches('0') {
            "" => Self::V1_0,
            "1" => Self::V1_1,
            "2" => Self::V1_2,
            _ => Self::HigherMinor,
        })
    }
}

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
    fn check_input(self, bytes: usize) -> Result<()> {
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
    options.check_input(source.len())?;

    Parser {
        source,
        pos: 0,
        options,
        // Leave room for a few short keys and collection nodes, which can occupy
        // only a handful of input bytes. Cap unused space on large scalars.
        nodes: Vec::with_capacity(
            (source.len() / 8 + 4)
                .min(source.len())
                .min(1024)
                .min(options.max_nodes),
        ),
        documents: Vec::new(),
        comments: Vec::new(),
        anchors: HashMap::new(),
        tags: HashMap::new(),
        remaining_tag_bytes: options.max_input_bytes,
        byte_order_marks: Vec::new(),
        document_end_markers: Vec::new(),
        elements: 0,
        deepest_node: 0,
        line_start: 0,
        lf_window: Cell::new(Span::default()),
        line_end_cache: Cell::new(None),
        colon_cache: Cell::new(None),
    }
    .stream()
}

struct Parser<'source> {
    source: &'source str,
    pos: usize,
    options: ParseOptions,
    nodes: Vec<Node<'source>>,
    documents: Vec<Document<'source>>,
    comments: Vec<Comment<'source>>,
    anchors: HashMap<&'source str, NodeId>,
    tags: HashMap<&'source str, &'source str>,
    remaining_tag_bytes: usize,
    byte_order_marks: Vec<Span>,
    document_end_markers: Vec<Span>,
    elements: usize,
    deepest_node: usize,
    line_start: usize,
    lf_window: Cell<Span>,
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

/// Whether a block sequence must be indented beyond its parent.
#[derive(Clone, Copy, Eq, PartialEq)]
enum SequenceIndent {
    Required,
    Optional,
}

impl<'source> Parser<'source> {
    fn error(&self, message: &'static str) -> Error {
        Error {
            span: Span {
                start: self.pos,
                end: self.pos + self.rest().chars().next().map_or(0, char::len_utf8),
            },
            message,
        }
    }

    fn rest(&self) -> &'source str {
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
        if let Some(line) = self.line_end_cache.get()
            && line.start <= self.pos
            && self.pos <= line.end
        {
            return line.end;
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

    fn check_block_indentation(&self) -> Result<()> {
        if self.source[self.line_start..self.pos].contains('\t') {
            return Err(self.error("tab in block indentation"));
        }
        Ok(())
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
        let scalar = Scalar::empty(offset);

        self.nodes.push(Node {
            span: scalar.span,
            anchor: None,
            tag: None,
            kind: NodeKind::Scalar(scalar),
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

    fn stream(mut self) -> Result<Stream<'source>> {
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

            clear_document_table(&mut self.anchors);
            clear_document_table(&mut self.tags);

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

            let root = if let Some(marker) = start_marker
                && self.boundary()
            {
                self.empty(marker.end)?
            } else {
                self.block(-1, 0, SequenceIndent::Required)?
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

        self.validate_characters()?;

        Ok(Stream {
            source: self.source,
            nodes: self.nodes,
            documents: self.documents,
            comments: self.comments,
            byte_order_marks: self.byte_order_marks,
            document_end_markers: self.document_end_markers,
        })
    }

    // JSON-compatible quoted scalars admit non-C0 characters excluded elsewhere.
    fn validate_characters(&self) -> Result<()> {
        // A full reduction lets the compiler vectorize this common ASCII check.
        // Unicode and restricted bytes still receive the contextual checks below.
        #[allow(clippy::needless_bitwise_bool)]
        let ascii = self.source.bytes().fold(true, |valid, byte| {
            valid
                & ((b' '..=b'~').contains(&byte)
                    | (byte == b'\t')
                    | (byte == b'\r')
                    | (byte == b'\n'))
        });
        if ascii {
            return Ok(());
        }

        let mut restricted = self.source.char_indices().filter(|&(_, c)| {
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

        let mut allowed: Vec<_> = self
            .nodes
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
            .chain(self.byte_order_marks.iter().copied())
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

    fn directives(&mut self) -> Result<Vec<Directive<'source>>> {
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

            let name_end = self.pos;
            let name = &self.source[name_start..name_end];

            while self.pos < end {
                if self.byte() == Some(b'#')
                    && self.source.as_bytes()[self.pos - 1].is_ascii_whitespace()
                {
                    break;
                }

                self.pos += self.rest().chars().next().map_or(0, char::len_utf8);
            }

            let tail = self.source[name_end..self.pos].trim_end_matches([' ', '\t']);
            let args = tail.trim_start_matches([' ', '\t']);
            let span = Span {
                start,
                end: name_end + tail.len(),
            };

            match name {
                "YAML" => {
                    if version {
                        return Err(self.error("duplicate YAML directive"));
                    }

                    if YamlVersion::parse(args).is_none() {
                        return Err(self.error("invalid YAML version directive"));
                    }

                    version = true;
                }
                "TAG" => {
                    let mut words = args.split_ascii_whitespace();
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
                    if !valid_tag(prefix, TagForm::Prefix) {
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

    fn block(
        &mut self,
        parent: isize,
        depth: usize,
        sequence_indent: SequenceIndent,
    ) -> Result<NodeId> {
        self.block_into(parent, depth, sequence_indent, None)
    }

    fn block_into(
        &mut self,
        parent: isize,
        depth: usize,
        sequence_indent: SequenceIndent,
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
            let id = self.inline(CollectionStyle::Block, parent, depth)?;
            self.finish_line()?;
            return Ok(id);
        }

        let id = if let Some(id) = existing {
            id
        } else {
            self.empty(start)?
        };

        let property_end = self.properties(id, CollectionStyle::Block, parent)?;
        if self.byte() == Some(b'#') {
            self.comment()?;
        }

        if self.newline() {
            self.block_space()?;
            if self.starts_nested_block(parent, sequence_indent) {
                // Properties and content belong to the same node.
                return self.block_into(parent, depth, sequence_indent, Some(id));
            }

            return self.finish_node(id, NodeKind::Scalar(Scalar::empty(property_end)));
        }

        // Block scalar scanning already stops at the next content line.
        if matches!(self.byte(), Some(b'|' | b'>')) {
            let kind = self.block_scalar(parent)?;
            return self.finish_node(id, kind);
        }

        let kind = self.inline_kind(CollectionStyle::Block, parent, depth, property_end)?;
        self.finish_node(id, kind)?;
        self.finish_line()?;
        Ok(id)
    }

    fn starts_nested_block(&self, parent: isize, sequence_indent: SequenceIndent) -> bool {
        let column = signed(self.column());
        !self.boundary()
            && (column > parent
                || (sequence_indent == SequenceIndent::Optional
                    && column == parent
                    && self.indicator(b'-')))
    }

    fn block_entry(&mut self, indent: usize, depth: usize, entry: BlockEntry) -> Result<NodeId> {
        self.pos += 1;
        let indicator_end = self.pos;
        self.spaces();

        let sequence_indent = match entry {
            BlockEntry::SequenceItem => SequenceIndent::Required,
            BlockEntry::ExplicitMapping | BlockEntry::ImplicitMappingValue => {
                SequenceIndent::Optional
            }
        };
        if matches!(self.byte(), None | Some(b'\r' | b'\n' | b'#')) {
            if self.byte() == Some(b'#') {
                self.comment()?;
            }

            self.newline();
            self.block_space()?;
            if !self.starts_nested_block(signed(indent), sequence_indent) {
                self.check_depth(depth)?;
                return self.empty(indicator_end);
            }
        } else if matches!(entry, BlockEntry::ImplicitMappingValue) {
            return self.implicit_mapping_value(indent, depth);
        }

        self.block(signed(indent), depth, sequence_indent)
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

            self.check_block_indentation()?;
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

    // Find the next colon, comment indicator or line break. Cache a bounded
    // carriage-return-free range so LF input needs one delimiter search.
    fn plain_key_delimiter(&self, mut start: usize) -> usize {
        const WINDOW_BYTES: usize = 4096;

        let bytes = self.source.as_bytes();
        loop {
            let mut window = self.lf_window.get();
            if start < window.start || start >= window.end {
                let remaining = &bytes[start..];
                let part = &remaining[..remaining.len().min(WINDOW_BYTES)];
                window = Span {
                    start,
                    end: start + memchr::memchr(b'\r', part).unwrap_or(part.len()),
                };
                self.lf_window.set(window);
            }
            if let Some(at) = memchr::memchr3(b':', b'#', b'\n', &bytes[start..window.end]) {
                return start + at;
            }
            start = window.end;
            if start == bytes.len() || bytes[start] == b'\r' {
                return start;
            }
        }
    }

    fn find_simple_colon(&self, lookahead: &mut KeyLookahead) -> Option<usize> {
        let bytes = self.source.as_bytes();
        let mut i = self.pos;

        // Colons in property and alias names are not mapping separators.
        while bytes.get(i).is_some_and(|b| b"!&*".contains(b)) {
            if bytes[i..].starts_with(b"!<") {
                i += memchr::memchr(b'>', &bytes[i..self.line_end()])? + 1;
            } else {
                while bytes.get(i).is_some_and(|b| !is_space(*b)) {
                    i += 1;
                }
            }

            while bytes.get(i).is_some_and(|b| matches!(b, b' ' | b'\t')) {
                i += 1;
            }
        }

        let plain = bytes.get(i).is_some_and(|b| !b"[{'\"".contains(b));
        if plain {
            let content_start = i;
            // Plain keys cannot open quoted or flow syntax. Skip their content
            // in runs, checking only separators and comments on this line.
            loop {
                i = self.plain_key_delimiter(i);

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
                            || CollectionStyle::Flow
                                .is_node_separator(bytes.get(i + 1).copied())) =>
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
                let key = self.inline(CollectionStyle::Block, signed(indent), depth + 1)?;
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
                    self.check_block_indentation()?;
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

            self.check_block_indentation()?;
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
            return self.block(signed(indent), depth, SequenceIndent::Optional);
        }

        // Lookahead has ruled out a block collection. Parse the inline
        // value without repeating block dispatch.
        let value = self.inline(CollectionStyle::Block, signed(indent), depth)?;
        self.finish_line()?;
        Ok(value)
    }

    fn tag_prefix(&self, handle: &str) -> Option<&'source str> {
        self.tags.get(handle).copied().or(match handle {
            "!" => Some("!"),
            "!!" => Some("tag:yaml.org,2002:"),
            _ => None,
        })
    }

    fn shorthand_tag(&mut self, start: usize) -> Result<Cow<'source, str>> {
        let raw = &self.source[start..self.pos];
        if raw == "!" {
            return Ok(Cow::Borrowed(raw));
        }

        let (handle, suffix) = raw[1..]
            .find('!')
            .map_or(("!", &raw[1..]), |n| (&raw[..n + 2], &raw[n + 2..]));
        let prefix = self.tag_prefix(handle).ok_or(Error {
            span: Span {
                start,
                end: start + handle.len(),
            },
            message: "undefined tag handle",
        })?;
        if suffix.is_empty() || suffix.contains('!') {
            return Err(self.error("invalid tag suffix"));
        }

        let suffix = tag_uri(suffix).ok_or_else(|| self.error("invalid tag URI"))?;
        self.remaining_tag_bytes = self
            .remaining_tag_bytes
            .checked_sub(prefix.len() + suffix.len())
            .ok_or_else(|| self.error("expanded tag byte limit exceeded"))?;

        let expanded = if prefix == handle {
            Cow::Borrowed(raw)
        } else {
            Cow::Owned(format!("{prefix}{suffix}"))
        };
        if !valid_tag(&expanded, TagForm::Complete) {
            return Err(self.error("invalid expanded tag URI"));
        }

        Ok(expanded)
    }

    /// Returns the end of the last property, before trailing separation.
    fn properties(&mut self, id: NodeId, context: CollectionStyle, parent: isize) -> Result<usize> {
        let mut end = self.pos;
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
                    if !valid_tag(value, TagForm::Complete) {
                        return Err(self.error("invalid verbatim tag URI"));
                    }
                    Cow::Borrowed(value)
                } else {
                    while self
                        .byte()
                        .is_some_and(|b| !is_space(b) && !b"[]{},".contains(&b))
                    {
                        self.pos += self.rest().chars().next().map_or(0, char::len_utf8);
                    }

                    self.shorthand_tag(start)?
                };

                self.nodes[id.0].tag = Some(Property {
                    span: Span {
                        start,
                        end: self.pos,
                    },
                    value,
                });
            }

            end = self.pos;

            // A property-only key may be followed immediately by its value indicator.
            let separator = match self.byte() {
                Some(b':') => self.next_byte(),
                byte => byte,
            };
            if !context.is_node_separator(separator) {
                return Err(self.error("node properties must be separated by whitespace"));
            }

            if context == CollectionStyle::Flow {
                self.flow_space(parent)?;
            } else {
                self.spaces();
            }
        }

        Ok(end)
    }

    #[inline]
    fn inline(&mut self, context: CollectionStyle, parent: isize, depth: usize) -> Result<NodeId> {
        self.check_depth(depth)?;

        // Scalars and aliases have no children or properties to register before
        // parsing. Append their completed node without an empty placeholder.
        if !matches!(self.byte(), Some(b'!' | b'&' | b'[' | b'{')) {
            self.reserve_element()?;
            let start = self.pos;
            let kind = self.inline_kind(context, parent, depth, start)?;
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
        let property_end = self.properties(id, context, parent)?;
        let kind = self.inline_kind(context, parent, depth, property_end)?;
        self.finish_node(id, kind)
    }

    fn finish_node(&mut self, id: NodeId, kind: NodeKind<'source>) -> Result<NodeId> {
        let node = &mut self.nodes[id.0];
        if matches!(kind, NodeKind::Alias { .. }) && (node.tag.is_some() || node.anchor.is_some()) {
            return Err(self.error("aliases cannot have node properties"));
        }

        node.span.end = match &kind {
            NodeKind::Scalar(scalar) => scalar.span.end,
            _ => self.pos,
        };
        node.kind = kind;
        Ok(id)
    }

    fn inline_kind(
        &mut self,
        context: CollectionStyle,
        parent: isize,
        depth: usize,
        empty_at: usize,
    ) -> Result<NodeKind<'source>> {
        match self.byte() {
            None | Some(b'\r' | b'\n' | b'#' | b',' | b']' | b'}') => {
                Ok(NodeKind::Scalar(Scalar::empty(empty_at)))
            }
            Some(b':') if context.is_node_separator(self.next_byte()) => {
                Ok(NodeKind::Scalar(Scalar::empty(empty_at)))
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
            Some(first) => self.plain(first, context, parent),
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
        if !json_key && !CollectionStyle::Flow.is_node_separator(self.byte()) {
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
            let indicator_end = self.pos;
            self.flow_space(parent)?;
            if self
                .byte()
                .is_some_and(|byte| byte == b',' || byte == closing)
            {
                self.empty(indicator_end)?
            } else {
                self.inline(CollectionStyle::Flow, parent, depth)?
            }
        } else {
            self.empty(self.nodes[key.0].span.end)?
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

    fn flow_sequence(&mut self, parent: isize, depth: usize) -> Result<NodeKind<'source>> {
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
            let key = self.inline(CollectionStyle::Flow, parent, depth + 1)?;

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

    fn flow_mapping(&mut self, parent: isize, depth: usize) -> Result<NodeKind<'source>> {
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

            let key = self.inline(CollectionStyle::Flow, parent, depth + 1)?;

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

    fn plain(
        &mut self,
        first: u8,
        context: CollectionStyle,
        parent: isize,
    ) -> Result<NodeKind<'source>> {
        let start = self.pos;
        let is_flow_indicator = |byte| {
            context == CollectionStyle::Flow && matches!(byte, b',' | b'[' | b']' | b'{' | b'}')
        };
        let is_separator =
            |next: Option<u8>| next.is_none_or(|byte| is_space(byte) || is_flow_indicator(byte));
        if b"@`|>!&*%{}[],\"'".contains(&first)
            || (b"-?:".contains(&first) && is_separator(self.next_byte()))
        {
            return Err(self.error("invalid plain scalar start"));
        }

        let mut end = start;
        let mut value: Option<String> = None;
        let mut segment = start;

        if context == CollectionStyle::Block {
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
                    let breaks = self.scalar_breaks(parent);

                    if self.document_boundary()
                        || self.byte() == Some(b'#')
                        || signed(self.column()) <= parent
                        || (self.byte() == Some(b':') && is_separator(self.next_byte()))
                        || self.byte().is_some_and(is_flow_indicator)
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

                Some(b':') if is_separator(self.next_byte()) => break,
                Some(byte) if is_flow_indicator(byte) => break,
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
                                || is_flow_indicator(b)
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

    /// Consume folded or escaped breaks, leaving a tab in the indentation for
    /// the caller to treat as the end of a plain scalar or an invalid quoted line.
    fn scalar_breaks(&mut self, parent: isize) -> usize {
        let mut breaks = 0;
        while self.newline() {
            breaks += 1;
            while self.byte() == Some(b' ') {
                self.pos += 1;
            }
            if self.byte() == Some(b'\t') {
                if signed(self.column()) <= parent {
                    break;
                }
                self.spaces();
            }
        }
        breaks
    }

    fn quoted_continuation(&self, parent: isize, quote: u8) -> Result<()> {
        // A BOM inside a quoted scalar is content, not a document boundary.
        if self.byte().is_none()
            || self.marker("---")
            || self.marker("...")
            || signed(self.column()) <= parent
        {
            return Err(self.error("invalid continuation of quoted scalar"));
        }

        if quote == b'"' && matches!(self.rest().as_bytes(), [b'\\', b'\r' | b'\n', ..]) {
            return Err(self.error("expected content before another escaped line break"));
        }

        Ok(())
    }

    fn quoted(&mut self, parent: isize) -> Result<NodeKind<'source>> {
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
                    let breaks = self.scalar_breaks(parent);
                    if breaks > 0 {
                        self.quoted_continuation(parent, quote)?;
                        text.extend(std::iter::repeat_n('\n', breaks - 1));
                    } else {
                        text.push(self.escape()?);
                    }

                    segment = self.pos;
                }
                Some(b'\r' | b'\n') => {
                    let text = output.get_or_insert_with(String::new);
                    text.push_str(self.source[segment..self.pos].trim_end_matches([' ', '\t']));
                    let breaks = self.scalar_breaks(parent);
                    self.quoted_continuation(parent, quote)?;

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

    fn block_scalar(&mut self, parent: isize) -> Result<NodeKind<'source>> {
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

        // An under-indented tab can begin stream separation after the document.
        let tab_error =
            (self.byte() == Some(b'\t')).then(|| self.error("tab in block scalar indentation"));
        self.block_space()?;
        if let Some(error) = tab_error
            && !self.document_boundary()
        {
            return Err(error);
        }

        let style = if folded {
            ScalarStyle::Folded
        } else {
            ScalarStyle::Literal
        };

        Ok(NodeKind::Scalar(Scalar {
            value: Cow::Owned(content.finish(chomp)),
            style,
            span: Span { start, end },
        }))
    }
}

fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | b'\n')
}

impl CollectionStyle {
    fn is_node_separator(self, byte: Option<u8>) -> bool {
        byte.is_none_or(|b| is_space(b) || (self == Self::Flow && matches!(b, b',' | b']' | b'}')))
    }
}

/// Reuse storage across similar-sized documents. Clearing a sparse table would
/// repeatedly scan capacity retained from a much larger document.
fn clear_document_table<K, V>(table: &mut HashMap<K, V>) {
    if table.len() < table.capacity() / 4 {
        *table = HashMap::new();
    } else {
        table.clear();
    }
}

fn valid_handle(s: &str) -> bool {
    s == "!"
        || (s.starts_with('!')
            && s.ends_with('!')
            && s[1..s.len() - 1]
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-'))
}

fn tag_uri(s: &str) -> Option<&str> {
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
    Some(s)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum TagForm {
    Prefix,
    Complete,
}

fn valid_tag(s: &str, form: TagForm) -> bool {
    if let Some(local) = s.strip_prefix('!') {
        return form == TagForm::Prefix || !local.is_empty();
    }

    let Some((scheme, rest)) = s.split_once(':') else {
        return form == TagForm::Prefix && uri_scheme(s);
    };
    if !uri_scheme(scheme) {
        return false;
    }

    let tail = if let Some(rest) = rest.strip_prefix("//") {
        let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
        let (authority, tail) = rest.split_at(end);
        if !uri_authority(authority, form == TagForm::Prefix && tail.is_empty()) {
            return false;
        }
        tail
    } else {
        rest
    };

    // tag_uri has checked the character repertoire and percent escapes.
    // RFC 3986 §3 reserves literal brackets for hosts and permits one fragment.
    !tail.contains(['[', ']']) && tail.split_once('#').is_none_or(|(_, f)| !f.contains('#'))
}

fn uri_scheme(scheme: &str) -> bool {
    let mut bytes = scheme.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_alphabetic())
        && bytes.all(|b| b.is_ascii_alphanumeric() || b"+.-".contains(&b))
}

fn uri_authority(authority: &str, open: bool) -> bool {
    // Until the host is delimited, this entire prefix can still be userinfo.
    if open && !authority.contains(['@', '[', ']']) {
        return true;
    }

    let (userinfo, host_port) = authority.split_once('@').unwrap_or(("", authority));
    if userinfo.contains(['[', ']']) || host_port.contains('@') {
        return false;
    }

    let port = if let Some(literal) = host_port.strip_prefix('[') {
        let Some((address, rest)) = literal.split_once(']') else {
            return open && ip_literal(literal, TagForm::Prefix);
        };
        if !ip_literal(address, TagForm::Complete) {
            return false;
        }
        if rest.is_empty() {
            return true;
        }
        let Some(port) = rest.strip_prefix(':') else {
            return false;
        };
        port
    } else {
        let (host, port) = host_port.split_once(':').unwrap_or((host_port, ""));
        if host.contains(['[', ']']) {
            return false;
        }
        port
    };
    port.bytes().all(|b| b.is_ascii_digit())
}

fn ip_literal(address: &str, form: TagForm) -> bool {
    let Some(future) = address.strip_prefix(['v', 'V']) else {
        return address.parse::<std::net::Ipv6Addr>().is_ok()
            || (form == TagForm::Prefix && ipv6_prefix(address));
    };
    // IPvFuture = "v" 1*HEXDIG "." 1*(unreserved / sub-delims / ":")
    let Some((version, host)) = future.split_once('.') else {
        return form == TagForm::Prefix && future.bytes().all(|b| b.is_ascii_hexdigit());
    };
    !version.is_empty()
        && version.bytes().all(|b| b.is_ascii_hexdigit())
        && (form == TagForm::Prefix || !host.is_empty())
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-._~!$&'()*+,;=:".contains(&b))
}

fn ipv6_prefix(address: &str) -> bool {
    // Six hexadecimal groups followed by a dotted IPv4 address take at most 45 bytes.
    if address.len() > 45 {
        return false;
    }

    // Fill out the final IPv4 octets or the missing IPv6 groups, then let the
    // standard parser check the resulting address.
    let mut candidate = address.to_owned();
    let dots = address.matches('.').count();
    if dots > 0 {
        if address.ends_with('.') {
            candidate.push('0');
        }
        for _ in dots..3 {
            candidate.push_str(".0");
        }
    } else if address.ends_with(':') {
        candidate.push(if address.contains("::") { '0' } else { ':' });
    } else {
        candidate.push_str("::");
    }
    candidate.parse::<std::net::Ipv6Addr>().is_ok()
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
