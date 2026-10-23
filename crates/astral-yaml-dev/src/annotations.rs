use annotate_snippets::{Annotation, AnnotationKind, Group, Level, Renderer, Snippet};
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
    render(snippet)
}

/// Renders a located diagnostic with its YAML path and the lines leading up to the error.
pub fn diagnostic_snapshot(source: &str, error: impl Into<astral_yaml_serde::Error>) -> String {
    let error = error.into();
    let span = error.span.expect("diagnostic must have a source span");
    let label = match error.path {
        Some(path) => format!("{path}: {}", error.message),
        None => error.message,
    };
    let context_start = source[..span.start()]
        .rmatch_indices('\n')
        .nth(2)
        .map_or(0, |(newline, _)| newline + 1);
    let snippet = Snippet::source(source)
        .annotation(AnnotationKind::Visible.span(context_start..span.start()))
        .annotation(
            AnnotationKind::Primary
                .span(span.start()..span.end())
                .label(label),
        );
    render(snippet)
}

fn render(snippet: Snippet<'_, Annotation<'_>>) -> String {
    Renderer::plain()
        .term_width(120)
        .render(&[Group::with_level(Level::NOTE).element(snippet)])
}
