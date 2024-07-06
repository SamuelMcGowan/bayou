use bayou_diagnostic::termcolor::{ColorChoice, StandardStream};
use bayou_diagnostic::Config;

use crate::sourcemap::SourceMap;

pub mod prelude {
    pub use bayou_diagnostic::span::Span;
    pub use bayou_diagnostic::{Severity, Snippet, SnippetKind};

    pub use super::{Diagnostic, IntoDiagnostic};
    pub use crate::sourcemap::SourceId;
}

use bayou_interner::Interner;

pub type Diagnostic = bayou_diagnostic::Diagnostic<SourceMap>;

pub trait DiagnosticEmitter {
    fn emit_diagnostic(&mut self, diagnostic: Diagnostic, sources: &SourceMap);
}

impl DiagnosticEmitter for Vec<Diagnostic> {
    fn emit_diagnostic(&mut self, diagnostic: Diagnostic, _sources: &SourceMap) {
        self.push(diagnostic);
    }
}

pub struct PrettyDiagnosticEmitter {
    pub stream: StandardStream,
    pub config: Config,
}

impl Default for PrettyDiagnosticEmitter {
    fn default() -> Self {
        Self {
            stream: StandardStream::stderr(ColorChoice::Auto),
            config: Default::default(),
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

pub trait IntoDiagnostic {
    fn into_diagnostic(self, interner: &Interner) -> Diagnostic;
}

impl IntoDiagnostic for Diagnostic {
    fn into_diagnostic(self, _interner: &Interner) -> Diagnostic {
        self
    }
}
