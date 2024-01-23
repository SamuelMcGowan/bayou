use std::ops::Range;

use self::sources::Sources;

mod render;
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
