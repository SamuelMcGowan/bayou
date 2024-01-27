use bayou_diagnostic::sources::{Cached, Source};
use bayou_diagnostic::DiagnosticKind;

use crate::diagnostics::{DiagnosticEmitter, IntoDiagnostic};
use crate::frontend::parser::Parser;
use crate::ir::Interner;
use crate::{CompilerError, CompilerResult};

pub struct Compiler<D: DiagnosticEmitter> {
    pub sources: Vec<Cached<(String, String)>>,
    pub diagnostics: D,
}

impl<D: DiagnosticEmitter> Compiler<D> {
    pub fn new(diagnostics: D) -> Self {
        Self {
            sources: vec![],
            diagnostics,
        }
    }

    pub fn parse_module(
        &mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> CompilerResult<()> {
        let source_id = self.sources.len();
        self.sources.push(Cached::new((name.into(), source.into())));
        let source = self.sources.last().unwrap();

        let parser = Parser::new(source.source_str());
        let (_ast, interner, parse_errors) = parser.parse();
        self.report(parse_errors, source_id, &interner)?;

        Ok(())
    }

    fn report<I: IntoIterator>(
        &mut self,
        diagnostics: I,
        source_id: usize,
        interner: &Interner,
    ) -> CompilerResult<()>
    where
        I::Item: IntoDiagnostic,
    {
        let mut had_errors = false;

        for diagnostic in diagnostics {
            let diagnostic = diagnostic.into_diagnostic(source_id, interner);
            had_errors |= diagnostic.kind >= DiagnosticKind::Error;
            self.diagnostics.emit_diagnostic(diagnostic, &self.sources);
        }

        if had_errors {
            Err(CompilerError::HadErrors)
        } else {
            Ok(())
        }
    }
}

// pub struct ModuleContext {
//     pub source_id: usize,

//     pub symbols: Symbols,
//     pub interner: Interner,
// }
