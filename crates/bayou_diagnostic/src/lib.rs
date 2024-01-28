mod render;
pub mod sources;
pub mod span;

use std::fmt;

use serde::ser::SerializeStruct;
use span::AsSpan;
pub use termcolor;
use termcolor::{Color, ColorSpec};

use self::sources::SourceMap;
use self::span::Span;

pub struct Diagnostic<S: SourceMap> {
    pub kind: DiagnosticKind,

    pub message: Option<String>,
    pub id: Option<String>,

    pub snippets: Vec<Snippet<S>>,
}

impl<S: SourceMap> Diagnostic<S> {
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

impl<S: SourceMap> fmt::Debug for Diagnostic<S>
where
    S::SourceId: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Diagnostic")
            .field("kind", &self.kind)
            .field("message", &self.message)
            .field("id", &self.id)
            .field("snippets", &self.snippets)
            .finish()
    }
}

#[cfg(feature = "serialize")]
impl<Srcs: SourceMap> serde::Serialize for Diagnostic<Srcs>
where
    Srcs::SourceId: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Diagnostic", 4)?;

        s.serialize_field("kind", &self.kind)?;
        s.serialize_field("message", &self.message)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("snippets", &self.snippets)?;

        s.end()
    }
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticKind {
    Warning,
    Error,
}

pub struct Snippet<S: SourceMap> {
    label: String,
    kind: SnippetKind,

    source_id: S::SourceId,
    span: Span,
}

impl<S: SourceMap> Snippet<S> {
    pub fn new(
        kind: SnippetKind,
        label: impl Into<String>,
        source_id: S::SourceId,
        span: impl AsSpan,
    ) -> Self {
        Self {
            label: label.into(),
            kind,

            source_id,
            span: span.as_span(),
        }
    }

    pub fn primary(label: impl Into<String>, source_id: S::SourceId, span: impl AsSpan) -> Self {
        Self::new(SnippetKind::Primary, label, source_id, span)
    }

    pub fn secondary(label: impl Into<String>, source_id: S::SourceId, span: impl AsSpan) -> Self {
        Self::new(SnippetKind::Secondary, label, source_id, span)
    }
}

impl<S: SourceMap> fmt::Debug for Snippet<S>
where
    S::SourceId: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Snippet")
            .field("label", &self.label)
            .field("kind", &self.kind)
            .field("source_id", &self.source_id)
            .field("span", &self.span)
            .finish()
    }
}

#[cfg(feature = "serialize")]
impl<Srcs: SourceMap> serde::Serialize for Snippet<Srcs>
where
    Srcs::SourceId: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Snippet", 4)?;

        s.serialize_field("label", &self.label)?;
        s.serialize_field("kind", &self.kind)?;
        s.serialize_field("source_id", &self.source_id)?;
        s.serialize_field("span", &self.span)?;

        s.end()
    }
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SnippetKind {
    Primary,
    Secondary,
}

#[derive(Debug)]
pub struct Config {
    pub context_size: usize,

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
        // emphasis.set_fg(Some(Color::Blue));
        emphasis.set_bold(true);

        Self {
            context_size: 2,

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
