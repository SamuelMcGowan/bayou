use bayou_diagnostic::DiagnosticKind;

use crate::diagnostics::{DiagnosticEmitter, IntoDiagnostic};
use crate::ir::Interner;
use crate::sourcemap::{SourceId, SourceMap};
use crate::{CompilerError, CompilerResult};

/// Session shared between multiple package compilations.
pub struct Session<D: DiagnosticEmitter> {
    pub sources: SourceMap,
    pub interner: Interner,
    pub diagnostics: D,
}

impl<D: DiagnosticEmitter> Session<D> {
    pub fn new(diagnostics: D) -> Self {
        Self {
            sources: SourceMap::default(),
            interner: Interner::new(),
            diagnostics,
        }
    }

    // TODO: don't take module contexts

    pub fn report(
        &mut self,
        diagnostic: impl IntoDiagnostic,
        source_id: SourceId,
    ) -> CompilerResult<()> {
        let diagnostic = diagnostic.into_diagnostic(source_id, &self.interner);
        let kind = diagnostic.kind;

        self.diagnostics.emit_diagnostic(diagnostic, &self.sources);

        if kind < DiagnosticKind::Error {
            Ok(())
        } else {
            Err(CompilerError::HadErrors)
        }
    }

    pub fn report_all<I>(&mut self, diagnostics: I, source_id: SourceId) -> CompilerResult<()>
    where
        I: IntoIterator,
        I::Item: IntoDiagnostic,
    {
        let mut had_error = false;

        for diagnostic in diagnostics {
            let diagnostic = diagnostic.into_diagnostic(source_id, &self.interner);
            had_error |= diagnostic.kind >= DiagnosticKind::Error;
            self.diagnostics.emit_diagnostic(diagnostic, &self.sources);
        }

        if !had_error {
            Ok(())
        } else {
            Err(CompilerError::HadErrors)
        }
    }
}
