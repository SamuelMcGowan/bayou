use crate::sourcemap::SourceMap;
use bayou_diagnostic::termcolor::{ColorChoice, StandardStream};

pub use bayou_diagnostic::*;

pub mod prelude {
    pub use super::{Diagnostic, IntoDiagnostic};
    pub use crate::sourcemap::SourceId;
    pub use bayou_diagnostic::span::Span;
    pub use bayou_diagnostic::{Severity, Snippet, SnippetKind};
}

pub type Diagnostic = bayou_diagnostic::Diagnostic<SourceMap>;

pub trait DiagnosticEmitter {
    fn emit_diagnostic(&mut self, diagnostic: Diagnostic, sources: &SourceMap);
}

impl DiagnosticEmitter for Vec<Diagnostic> {
    fn emit_diagnostic(&mut self, diagnostic: Diagnostic, _sources: &SourceMap) {
        self.push(diagnostic);
    }
}

#[derive(Debug)]
pub struct PrettyDiagnosticEmitter {
    pub stream: StandardStream,
    pub config: Config,
}

impl Default for PrettyDiagnosticEmitter {
    fn default() -> Self {
        Self {
            stream: StandardStream::stderr(ColorChoice::Auto),
            config: Config::default(),
        }
    }
}

impl DiagnosticEmitter for PrettyDiagnosticEmitter {
    fn emit_diagnostic(&mut self, diagnostic: Diagnostic, sources: &SourceMap) {
        diagnostic
            .write_to_stream(sources, &self.config, &mut self.stream)
            .expect("failed to emit diagnostic");
    }
}

pub trait IntoDiagnostic<Context: ?Sized> {
    fn into_diagnostic(self, cx: &Context) -> Diagnostic;
}

impl IntoDiagnostic<()> for Diagnostic {
    fn into_diagnostic(self, _cx: &()) -> Diagnostic {
        self
    }
}
