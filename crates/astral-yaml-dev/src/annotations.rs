use annotate_snippets::{AnnotationKind, Group, Level, Renderer, Snippet};
use astral_yaml::Span;
use std::borrow::Cow;

/// Renders labeled source spans without terminal colors or machine-specific paths.
/// Labels can describe decoded values, field names, or comment ownership.
/// Render overlapping spans separately so each endpoint remains visible. Retain
/// byte-level assertions for line endings and other invisible source differences.
pub fn span_snapshot<'a, L>(source: &'a str, spans: impl IntoIterator<Item = (L, Span)>) -> String
where
    L: Into<Cow<'a, str>>,
{
    let snippet = Snippet::source(source)
        .fold(false)
        .annotations(spans.into_iter().map(|(label, span)| {
            let label: Cow<'a, str> = label.into();
            AnnotationKind::Primary
                .span(span.start()..span.end())
                .label(label)
        }));
    Renderer::plain()
        .term_width(120)
        .render(&[Group::with_level(Level::NOTE).element(snippet)])
}
