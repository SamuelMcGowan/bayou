use bayou_diagnostic::termcolor::{ColorChoice, StandardStream};
use bayou_diagnostic::{Config, Snippet};

use crate::compiler::ModuleCx;
use crate::parser::ParseError;
use crate::passes::type_check::TypeError;
use crate::resolver::ResolverError;
use crate::sourcemap::SourceMap;

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
    // TODO: take reference to source context
    fn into_diagnostic(self, module_context: &ModuleCx) -> Diagnostic;
}

impl IntoDiagnostic for ParseError {
    fn into_diagnostic(self, module_context: &ModuleCx) -> Diagnostic {
        match self {
            ParseError::Expected { expected, span } => Diagnostic::error()
                .with_message("syntax error")
                .with_snippet(Snippet::primary(
                    format!("expected {expected} here"),
                    module_context.source_id,
                    span,
                )),

            ParseError::Lexer(error) => Diagnostic::error()
                .with_message("syntax error")
                .with_snippet(Snippet::primary(
                    error.kind.to_string(),
                    module_context.source_id,
                    error.span,
                )),
        }
    }
}

impl IntoDiagnostic for ResolverError {
    fn into_diagnostic(self, module_context: &ModuleCx) -> Diagnostic {
        match self {
            ResolverError::DuplicateGlobal { first, second } => {
                let name_str = module_context.interner.resolve(&first.ident);
                Diagnostic::error()
                    .with_message(format!("duplicate global `{name_str}`"))
                    .with_snippet(Snippet::secondary(
                        "first definition",
                        module_context.source_id,
                        first.span,
                    ))
                    .with_snippet(Snippet::primary(
                        "second definition",
                        module_context.source_id,
                        second.span,
                    ))
            }

            ResolverError::LocalUndefined(ident) => {
                let name_str = module_context.interner.resolve(&ident.ident);
                Diagnostic::error()
                    .with_message(format!("undefined variable `{name_str}`"))
                    .with_snippet(Snippet::primary(
                        "undefined variable here",
                        module_context.source_id,
                        ident.span,
                    ))
            }
        }
    }
}

impl IntoDiagnostic for TypeError {
    fn into_diagnostic(self, module_context: &ModuleCx) -> Diagnostic {
        match self {
            TypeError::TypeMismatch {
                expected,
                expected_span,
                found,
                found_span,
            } => {
                let mut diagnostic = Diagnostic::error()
                    .with_message(format!("expected type {expected:?}, found type {found:?}"))
                    .with_snippet(Snippet::primary(
                        "unexpected type",
                        module_context.source_id,
                        found_span,
                    ));

                if let Some(expected_span) = expected_span {
                    diagnostic = diagnostic.with_snippet(Snippet::secondary(
                        "expected due to this type",
                        module_context.source_id,
                        expected_span,
                    ));
                }

                diagnostic
            }

            TypeError::MissingReturn { ty, span } => Diagnostic::error()
                .with_message(format!(
                    "missing return statement in function that returns type {ty:?}"
                ))
                .with_snippet(Snippet::primary(
                    "expected due to this return type",
                    module_context.source_id,
                    span,
                )),
        }
    }
}
