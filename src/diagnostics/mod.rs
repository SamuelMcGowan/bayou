use termcolor::{Color, ColorSpec};

use self::sources::Sources;
use self::span::Span;

mod render;
mod sources;
mod span;

pub struct Diagnostic<S: Sources> {
    kind: DiagnosticKind,

    message: Option<String>,
    id: Option<String>,

    snippets: Vec<Snippet<S>>,
}

impl<S: Sources> Diagnostic<S> {
    pub fn new(kind: DiagnosticKind) -> Self {
        Self {
            kind,
            message: None,
            id: None,
            snippets: vec![],
        }
    }

    pub fn warning() -> Self {
        Self::new(DiagnosticKind::Warning)
    }

    pub fn error() -> Self {
        Self::new(DiagnosticKind::Error)
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_snippet(mut self, snippet: Snippet<S>) -> Self {
        self.snippets.push(snippet);
        self
    }

    pub fn with_snippets(mut self, snippets: impl IntoIterator<Item = Snippet<S>>) -> Self {
        self.snippets.extend(snippets);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticKind {
    Warning,
    Error,
}

pub struct Snippet<S: Sources> {
    label: String,
    kind: SnippetKind,

    source_id: S::SourceId,
    span: Span,
}

impl<S: Sources> Snippet<S> {
    pub fn new(
        kind: SnippetKind,
        label: impl Into<String>,
        source_id: S::SourceId,
        span: impl Into<Span>,
    ) -> Self {
        Self {
            label: label.into(),
            kind,

            source_id,
            span: span.into(),
        }
    }

    pub fn primary(
        label: impl Into<String>,
        source_id: S::SourceId,
        span: impl Into<Span>,
    ) -> Self {
        Self::new(SnippetKind::Primary, label, source_id, span)
    }

    pub fn secondary(
        label: impl Into<String>,
        source_id: S::SourceId,
        span: impl Into<Span>,
    ) -> Self {
        Self::new(SnippetKind::Secondary, label, source_id, span)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SnippetKind {
    Primary,
    Secondary,
}

pub struct Config {
    pub error_color: ColorSpec,
    pub warning_color: ColorSpec,

    pub emphasis: ColorSpec,
    pub subtle: ColorSpec,

    pub gutter: &'static str,

    pub multiline_top: &'static str,
    pub multiline_main: &'static str,
    pub multiline_bottom: &'static str,
    pub multiline_very_bottom: &'static str,
    pub multiline_empty: &'static str,

    pub underline: &'static str,
    pub underline_after: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        let mut error_color = ColorSpec::new();
        error_color.set_fg(Some(Color::Red));
        error_color.set_bold(true);

        let mut warning_color = ColorSpec::new();
        warning_color.set_fg(Some(Color::Yellow));
        warning_color.set_bold(true);

        let mut subtle = ColorSpec::new();
        subtle.set_italic(true);
        subtle.set_dimmed(true);

        let mut emphasis = ColorSpec::new();
        emphasis.set_fg(Some(Color::Blue));
        emphasis.set_bold(true);

        Self {
            error_color,
            warning_color,
            emphasis,
            subtle,

            gutter: "│",

            multiline_top: "╭─▷",
            multiline_main: "│  ",
            multiline_bottom: "├─▷",
            multiline_very_bottom: "╰─◎",
            multiline_empty: "   ",

            underline: "^",
            underline_after: "  ",
        }
    }
}
