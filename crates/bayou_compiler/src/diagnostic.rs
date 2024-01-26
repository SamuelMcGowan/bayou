use bayou_diagnostic::sources::Cached;
use bayou_diagnostic::termcolor::{ColorChoice, StandardStream, WriteColor};
use bayou_diagnostic::{Config, Diagnostic, DiagnosticKind};

use crate::{CompilerError, CompilerResult};

pub type Sources = Vec<Source>;
pub type Source = Cached<(String, String)>;

#[derive(Debug)]
pub enum DiagnosticOutput {
    Owned(Vec<Diagnostic<Sources>>),
    StandardStream {
        stream: StandardStream,
        config: Box<Config>,
        had_errors: bool,
    },
}

impl DiagnosticOutput {
    pub fn owned() -> Self {
        Self::Owned(vec![])
    }

    pub fn stderr() -> Self {
        Self::StandardStream {
            stream: StandardStream::stderr(ColorChoice::Auto),
            config: Box::default(),
            had_errors: false,
        }
    }

    pub fn report(&mut self, diagnostic: impl IntoDiagnostic, sources: &Sources) {
        let diagnostic = diagnostic.into_diagnostic();
        match self {
            DiagnosticOutput::Owned(diagnostics) => diagnostics.push(diagnostic),
            DiagnosticOutput::StandardStream {
                stream,
                config,
                had_errors,
            } => {
                diagnostic
                    .write_to_stream(sources, config, stream)
                    .expect("failed to write diagnostic");
                *had_errors = true;
            }
        }
    }

    pub fn had_errors(&self) -> bool {
        match self {
            Self::Owned(diagnostics) => !diagnostics.is_empty(),
            Self::StandardStream { had_errors, .. } => *had_errors,
        }
    }
}

pub trait IntoDiagnostic {
    fn into_diagnostic(self) -> Diagnostic<Sources>;
}

impl IntoDiagnostic for Diagnostic<Sources> {
    fn into_diagnostic(self) -> Diagnostic<Sources> {
        self
    }
}

#[derive(Default)]
#[must_use = "diagnostics must be emitted"]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic<Sources>>,
    had_errors: bool,
}

impl Diagnostics {
    pub fn report(&mut self, diagnostic: impl IntoDiagnostic) {
        let diagnostic = diagnostic.into_diagnostic();
        self.had_errors |= diagnostic.kind >= DiagnosticKind::Error;
        self.diagnostics.push(diagnostic);
    }

    pub fn had_errors(&self) -> bool {
        self.had_errors
    }

    pub fn emit(
        self,
        sources: &Sources,
        config: &Config,
        stream: &mut impl WriteColor,
    ) -> CompilerResult<()> {
        for diagnostic in self.diagnostics {
            diagnostic.write_to_stream(sources, config, stream)?;
        }

        if self.had_errors {
            Err(CompilerError::HadErrors)
        } else {
            Ok(())
        }
    }

    pub fn join(&mut self, diagnostics: Diagnostics) {
        self.diagnostics.extend(diagnostics.diagnostics);
        self.had_errors |= diagnostics.had_errors;
    }
}
