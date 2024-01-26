use std::io;

use bayou_diagnostic::sources::Cached;
use bayou_diagnostic::termcolor::WriteColor;
use bayou_diagnostic::{Config, Diagnostic, DiagnosticKind};

pub type Sources = Vec<Source>;
pub type Source = Cached<(String, String)>;

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

    pub fn flush(
        &mut self,
        sources: &Sources,
        config: &Config,
        stream: &mut impl WriteColor,
    ) -> io::Result<()> {
        for diagnostic in self.diagnostics.drain(..) {
            diagnostic.write_to_stream(sources, config, stream)?;
        }
        Ok(())
    }

    pub fn join(&mut self, diagnostics: Diagnostics) {
        self.diagnostics.extend(diagnostics.diagnostics);
        self.had_errors |= diagnostics.had_errors;
    }
}
