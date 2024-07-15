mod render;
pub mod sources;
pub mod span;

use std::fmt;

use derive_where::derive_where;
#[cfg(feature = "serialize")]
use serde::Serialize;
use span::AsSpan;
pub use termcolor;
use termcolor::{Color, ColorSpec};

use self::sources::SourceMap;
use self::span::Span;

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive_where(Debug, Clone; S::SourceId)]
pub struct Diagnostic<S: SourceMap> {
    pub severity: Severity,

    pub message: Option<String>,
    pub id: Option<String>,

    #[cfg_attr(
        feature = "serialize",
        serde(bound(serialize = "S::SourceId: Serialize"))
    )]
    pub snippets: Vec<Snippet<S>>,

    pub tags: Vec<(TagKind, String)>,
}

impl<S: SourceMap> Diagnostic<S> {
    pub fn new(kind: Severity) -> Self {
        Self {
            severity: kind,
            message: None,
            id: None,
            snippets: vec![],
            tags: vec![],
        }
    }

    pub fn warning() -> Self {
        Self::new(Severity::Warning)
    }

    pub fn error() -> Self {
        Self::new(Severity::Error)
    }

    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    #[must_use]
    pub fn with_snippet(mut self, snippet: Snippet<S>) -> Self {
        self.snippets.push(snippet);
        self
    }

    #[must_use]
    pub fn with_snippets(mut self, snippets: impl IntoIterator<Item = Snippet<S>>) -> Self {
        self.snippets.extend(snippets);
        self
    }

    #[must_use]
    pub fn with_note(self, note: impl Into<String>) -> Self {
        self.with_tag(TagKind::Note, note)
    }

    #[must_use]
    pub fn with_notes<N: Into<String>>(self, notes: impl IntoIterator<Item = N>) -> Self {
        self.with_tags(TagKind::Note, notes)
    }

    #[must_use]
    pub fn with_suggestion(self, suggestion: impl Into<String>) -> Self {
        self.with_tag(TagKind::Suggestion, suggestion)
    }

    #[must_use]
    pub fn with_suggestions<T: Into<String>>(
        self,
        suggestions: impl IntoIterator<Item = T>,
    ) -> Self {
        self.with_tags(TagKind::Suggestion, suggestions)
    }

    #[must_use]
    pub fn with_tag(mut self, kind: TagKind, message: impl Into<String>) -> Self {
        self.tags.push((kind, message.into()));
        self
    }

    #[must_use]
    pub fn with_tags<N: Into<String>>(
        mut self,
        kind: TagKind,
        messages: impl IntoIterator<Item = N>,
    ) -> Self {
        self.tags
            .extend(messages.into_iter().map(|msg| (kind, msg.into())));
        self
    }
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Warning,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Warning => "Warning",
            Self::Error => "Error",
        };
        write!(f, "{s}")
    }
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive_where(Debug, Clone; S::SourceId)]
pub struct Snippet<S: SourceMap> {
    label: String,
    kind: SnippetKind,

    #[cfg_attr(
        feature = "serialize",
        serde(bound(serialize = "S::SourceId: Serialize"))
    )]
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

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SnippetKind {
    Primary,
    Secondary,
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TagKind {
    Note,
    Suggestion,
}

impl fmt::Display for TagKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Note => "Note",
            Self::Suggestion => "Suggestion",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub context_size: usize,

    pub error_color: ColorSpec,
    pub warning_color: ColorSpec,

    pub note_color: ColorSpec,
    pub suggestion_color: ColorSpec,

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

    pub before_tag: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        let mut error_color = ColorSpec::new();
        error_color.set_fg(Some(Color::Red));
        error_color.set_bold(true);

        let mut warning_color = ColorSpec::new();
        warning_color.set_fg(Some(Color::Yellow));
        warning_color.set_bold(true);

        let mut note_color = ColorSpec::new();
        note_color.set_fg(Some(Color::Blue));
        note_color.set_bold(true);

        let mut suggestion_color = ColorSpec::new();
        suggestion_color.set_fg(Some(Color::Green));
        suggestion_color.set_bold(true);

        let mut emphasis = ColorSpec::new();
        // emphasis.set_fg(Some(Color::Blue));
        emphasis.set_bold(true);

        let mut subtle = ColorSpec::new();
        subtle.set_italic(true);
        subtle.set_dimmed(true);

        Self {
            context_size: 2,

            error_color,
            warning_color,

            note_color,
            suggestion_color,

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

            before_tag: "  + ",
        }
    }
}
