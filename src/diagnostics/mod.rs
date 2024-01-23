use std::ops::Range;

use termcolor::{Color, ColorSpec};

use self::sources::Sources;

// mod render;
mod render2;
mod sources;

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

    source_id: S::SourceId,
    span: Range<usize>,
}

impl<S: Sources> Snippet<S> {
    pub fn new(label: impl Into<String>, source_id: S::SourceId, span: Range<usize>) -> Self {
        Self {
            label: label.into(),

            source_id,
            span,
        }
    }
}

pub struct Config {
    pub error_color: ColorSpec,
    pub warning_color: ColorSpec,

    pub emphasis: ColorSpec,
    pub subtle: ColorSpec,

    pub gutter_top: &'static str,
    pub gutter_main: &'static str,
    pub gutter_bottom: &'static str,
    pub gutter_trace: &'static str,
    pub gutter_empty: &'static str,

    pub underline: &'static str,
    pub underline_trace: &'static str,
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

            gutter_top: "╭─▷",
            gutter_main: "│  ",
            gutter_bottom: "├─▷",
            gutter_trace: "│  ",
            gutter_empty: "   ",

            underline: "-",
            underline_trace: "  ",
        }
    }
}
