use std::fmt;

use crate::parser::YamlVersion;
use crate::{Span, Stream};

/// A YAML compatibility warning from a successful parse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Warning {
    /// The directive or character in the decoded UTF-8 source.
    pub span: Span,
    /// The reason for the warning.
    pub kind: WarningKind,
}

/// A condition for which YAML 1.2 recommends a warning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WarningKind {
    /// An unrecognized directive was ignored.
    UnknownDirective,
    /// A higher YAML 1.x minor version was processed as YAML 1.2.
    HigherMinorVersion,
    /// A literal YAML 1.1 line break was treated as a non-break character.
    NonAsciiLineBreak,
}

impl fmt::Display for WarningKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnknownDirective => "unknown directive ignored",
            Self::HigherMinorVersion => "higher YAML minor version processed as YAML 1.2",
            Self::NonAsciiLineBreak => "YAML 1.1 line break treated as a non-break character",
        })
    }
}

impl Stream<'_> {
    /// Yields compatibility warnings in source order. Only documents that declare
    /// YAML 1.1 are scanned for non-ASCII line breaks.
    ///
    /// ```
    /// use astral_yaml::WarningKind;
    ///
    /// let stream = astral_yaml::parse("%FUTURE option\n--- value")?;
    /// let warning = stream.warnings().next().unwrap();
    /// assert_eq!(warning.kind, WarningKind::UnknownDirective);
    /// assert_eq!(warning.span.slice(stream.source()), Some("%FUTURE option"));
    /// # Ok::<(), astral_yaml::Error>(())
    /// ```
    pub fn warnings(&self) -> impl Iterator<Item = Warning> + '_ {
        (0..self.documents.len()).flat_map(|index| self.document_warnings(index))
    }

    fn document_warnings(&self, index: usize) -> impl Iterator<Item = Warning> + '_ {
        let doc = &self.documents[index];

        // Trailing comments belong to this document until a suffix or the
        // next document's prefix. A BOM can start a prefix before any directive.
        let next = self.documents.get(index + 1).map(|next| next.span);
        let end = doc
            .end_marker
            .or(next)
            .map_or(self.source.len(), Span::start);
        let bom = self
            .byte_order_marks
            .partition_point(|bom| bom.start < doc.span.start);
        let end = self
            .byte_order_marks
            .get(bom)
            .map_or(end, |bom| end.min(bom.start));
        let directives = &doc.directives;
        let version = directives
            .iter()
            .find(|directive| directive.name == "YAML")
            .and_then(|directive| YamlVersion::parse(directive.arguments));

        // Scan from each directive up to the next so line breaks in directives
        // and their comments appear in source order.
        let ends = directives.iter().skip(1).map(|d| d.span.start).chain([end]);
        directives
            .iter()
            .zip(ends)
            .flat_map(move |(directive, end)| {
                let kind = match directive.name {
                    "YAML" if version == Some(YamlVersion::HigherMinor) => {
                        Some(WarningKind::HigherMinorVersion)
                    }
                    "YAML" | "TAG" => None,
                    _ => Some(WarningKind::UnknownDirective),
                };
                let warning = kind.map(|kind| Warning {
                    span: directive.span,
                    kind,
                });

                let start = directive.span.start;
                let text = if version == Some(YamlVersion::V1_1) {
                    &self.source[start..end]
                } else {
                    ""
                };
                warning
                    .into_iter()
                    .chain(non_ascii_line_breaks(text, start))
            })
    }
}

fn non_ascii_line_breaks(text: &str, offset: usize) -> impl Iterator<Item = Warning> + '_ {
    text.match_indices(['\u{85}', '\u{2028}', '\u{2029}'])
        .map(move |(index, character)| {
            let start = offset + index;
            Warning {
                span: Span {
                    start,
                    end: start + character.len(),
                },
                kind: WarningKind::NonAsciiLineBreak,
            }
        })
}
