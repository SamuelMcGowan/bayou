pub mod diagnostics;
pub mod platform;
pub mod sourcemap;

use diagnostics::prelude::*;
use diagnostics::DiagnosticEmitter;
pub use lasso;
use sourcemap::SourceMap;
use target_lexicon::Triple;

pub type InternedStr = lasso::Spur;
pub type Interner = lasso::Rodeo;

#[derive(thiserror::Error, Debug)]
#[error("had errors")]
pub struct HadErrors;

/// Session shared between multiple package compilations.
pub struct Session<D: DiagnosticEmitter> {
    pub target: Triple,

    pub sources: SourceMap,
    pub interner: Interner,

    pub diagnostics: D,
}

impl<D: DiagnosticEmitter> Session<D> {
    pub fn new(target: Triple, diagnostics: D) -> Self {
        Self {
            target,

            sources: SourceMap::default(),
            interner: Interner::new(),

            diagnostics,
        }
    }

    pub fn report(
        &mut self,
        diagnostic: impl IntoDiagnostic,
        source_id: SourceId,
    ) -> Result<(), HadErrors> {
        let diagnostic = diagnostic.into_diagnostic(source_id, &self.interner);
        let kind = diagnostic.severity;

        self.diagnostics.emit_diagnostic(diagnostic, &self.sources);

        if kind < Severity::Error {
            Ok(())
        } else {
            Err(HadErrors)
        }
    }

    pub fn report_all<I>(&mut self, diagnostics: I, source_id: SourceId) -> Result<(), HadErrors>
    where
        I: IntoIterator,
        I::Item: IntoDiagnostic,
    {
        let mut had_error = false;

        for diagnostic in diagnostics {
            let diagnostic = diagnostic.into_diagnostic(source_id, &self.interner);
            had_error |= diagnostic.severity >= Severity::Error;
            self.diagnostics.emit_diagnostic(diagnostic, &self.sources);
        }

        if !had_error {
            Ok(())
        } else {
            Err(HadErrors)
        }
    }
}
