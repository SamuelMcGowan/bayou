use bayou_diagnostic::sources::Cached;
use bayou_diagnostic::termcolor::{ColorChoice, StandardStream};
use bayou_diagnostic::{Config, Diagnostic};

pub type Sources = Vec<Cached<(String, String)>>;

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
