//! Locate source positions without rescanning a flow line for every node on it.

use std::collections::BTreeMap;

/// A physical line in the editable document's LF-normalized source.
#[derive(Clone, Copy)]
pub(super) struct Line {
    start: usize,
    end: usize,
    column_zero: usize,
    content: usize,
}

impl Line {
    pub(super) fn at(source: &str, at: usize) -> Self {
        let bytes = source.as_bytes();
        let start = memchr::memrchr(b'\n', &bytes[..at]).map_or(0, |newline| newline + 1);
        let end = memchr::memchr(b'\n', &bytes[at..]).map_or(bytes.len(), |newline| at + newline);
        let text = &source[start..end];
        let content = text.trim_start_matches('\u{feff}');
        let column_zero = end - content.len();
        Self {
            start,
            end,
            column_zero,
            content: end - content.trim_start_matches(' ').len(),
        }
    }

    pub(super) fn column(self, at: usize) -> usize {
        at.saturating_sub(self.column_zero)
    }

    pub(super) fn indent(self, at: usize) -> usize {
        self.column(at.min(self.content))
    }

    pub(super) fn own_line(self, at: usize) -> bool {
        at <= self.content
    }
}

/// Index only lines visited by the planner. A multiline scalar or a run of blank
/// lines can occupy most of the input while contributing just one node.
pub(super) struct LineIndex<'a> {
    source: &'a str,
    by_end: BTreeMap<usize, Line>,
}

impl<'a> LineIndex<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self {
            source,
            by_end: BTreeMap::new(),
        }
    }

    pub(super) fn at(&mut self, at: usize) -> Line {
        if let Some((_, &line)) = self.by_end.range(at..).next()
            && line.start <= at
        {
            return line;
        }
        let line = Line::at(self.source, at);
        self.by_end.insert(line.end, line);
        line
    }
}
