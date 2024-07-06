pub mod diagnostics;
pub mod sourcemap;

use bayou_interner::Interner;
use diagnostics::DiagnosticEmitter;
use diagnostics::*;
use sourcemap::SourceMap;
use target_lexicon::Triple;

pub struct ErrorsEmitted;

/// Session shared between multiple package compilations.
pub struct Session<D: DiagnosticEmitter> {
    pub target: Triple,
    pub sources: SourceMap,
    pub diagnostics: D,
}

impl<D: DiagnosticEmitter> Session<D> {
    pub fn new(target: Triple, diagnostics: D) -> Self {
        Self {
            target,
            sources: SourceMap::default(),
            diagnostics,
        }
    }

    pub fn report(
        &mut self,
        diagnostic: impl IntoDiagnostic,
        interner: &Interner,
    ) -> Result<(), ErrorsEmitted> {
        let diagnostic = diagnostic.into_diagnostic(interner);
        let kind = diagnostic.severity;

        self.diagnostics.emit_diagnostic(diagnostic, &self.sources);

        if kind < Severity::Error {
            Ok(())
        } else {
            Err(ErrorsEmitted)
        }
    }

    pub fn report_all<I>(
        &mut self,
        diagnostics: I,
        interner: &Interner,
    ) -> Result<(), ErrorsEmitted>
    where
        I: IntoIterator,
        I::Item: IntoDiagnostic,
    {
        let mut had_error = false;

        for diagnostic in diagnostics {
            let diagnostic = diagnostic.into_diagnostic(interner);
            had_error |= diagnostic.severity >= Severity::Error;
            self.diagnostics.emit_diagnostic(diagnostic, &self.sources);
        }

        if !had_error {
            Ok(())
        } else {
            Err(ErrorsEmitted)
        }
    }
}
