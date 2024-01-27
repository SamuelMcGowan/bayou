use bayou_diagnostic::sources::Cached;
use bayou_diagnostic::termcolor::{ColorChoice, StandardStream};
use bayou_diagnostic::{Config, Snippet};

use crate::frontend::parser::ParseError;
use crate::ir::Interner;

type Sources = Vec<Cached<(String, String)>>;

pub type Diagnostic = bayou_diagnostic::Diagnostic<Sources>;

pub trait DiagnosticEmitter {
    fn emit_diagnostic(&mut self, diagnostic: Diagnostic, sources: &Sources);
}

impl DiagnosticEmitter for Vec<Diagnostic> {
    fn emit_diagnostic(&mut self, diagnostic: Diagnostic, _sources: &Sources) {
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
    fn emit_diagnostic(&mut self, diagnostic: Diagnostic, sources: &Sources) {
        diagnostic
            .write_to_stream(sources, &self.config, &mut self.stream)
            .expect("failed to emit diagnostic");
    }
}

pub trait IntoDiagnostic {
    // TODO: take reference to source context
    fn into_diagnostic(self, source_id: usize, interner: &Interner) -> Diagnostic;
}

impl IntoDiagnostic for ParseError {
    fn into_diagnostic(self, source_id: usize, _interner: &Interner) -> Diagnostic {
        match self {
            ParseError::Expected { expected, found } => match found {
                Some(token) => Diagnostic::error()
                    .with_message(format!("expected {expected}"))
                    .with_snippet(Snippet::primary(
                        format!("expected {expected} here"),
                        source_id,
                        token.span,
                    )),

                None => Diagnostic::error()
                    .with_message(format!("expected {expected}, but reached end of source",))
                    // TODO: eof spans

                    // .with_snippet(Snippet::primary(
                    //     format!("expected {expected} here"),
                    //     source_id,
                    //     self.lexer.eof_span(),
                    // )),
            },

            ParseError::Lexer(error) => Diagnostic::error()
                .with_message(error.kind.to_string())
                .with_snippet(Snippet::primary("this token", source_id, error.span)),
        }
    }
}
