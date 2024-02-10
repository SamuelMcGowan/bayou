use bayou_diagnostic::sources::{Source as _, SourceMap as _};
use bayou_diagnostic::DiagnosticKind;
use cranelift_object::ObjectProduct;
use target_lexicon::Triple;

use crate::codegen::Codegen;
use crate::diagnostics::{DiagnosticEmitter, IntoDiagnostic};
use crate::ir::ast::Module;
use crate::ir::Interner;
use crate::parser::Parser;
use crate::sourcemap::{Source, SourceId, SourceMap};
use crate::symbols::Symbols;
use crate::{CompilerError, CompilerResult};

pub struct Compiler<D: DiagnosticEmitter> {
    pub sources: SourceMap,
    pub triple: Triple,
    pub diagnostics: D,
}

impl<D: DiagnosticEmitter> Compiler<D> {
    pub fn new(diagnostics: D, triple: Triple) -> Self {
        Self {
            sources: SourceMap::default(),
            triple,
            diagnostics,
        }
    }

    pub fn parse_module(
        &mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> CompilerResult<(Module, ModuleContext)> {
        let source_id = self.sources.insert(Source::new(name, source));
        let source = self.sources.get_source(source_id).unwrap();

        let parser = Parser::new(source.source_str());
        let (ast, interner, parse_errors) = parser.parse();

        let module_context = ModuleContext {
            source_id,
            symbols: Symbols::default(),
            interner,
        };

        self.report(parse_errors, &module_context)?;

        Ok((ast, module_context))
    }

    pub fn compile(
        &mut self,
        name: &str,
        module: &Module,
        cx: &ModuleContext,
    ) -> CompilerResult<ObjectProduct> {
        let mut codegen = Codegen::new(self.triple.clone(), name)?;
        codegen.compile_module(module, cx)?;
        let object = codegen.finish();
        Ok(object)
    }

    fn report<I: IntoIterator>(
        &mut self,
        diagnostics: I,
        module_context: &ModuleContext,
    ) -> CompilerResult<()>
    where
        I::Item: IntoDiagnostic,
    {
        let mut had_errors = false;

        for diagnostic in diagnostics {
            let diagnostic = diagnostic.into_diagnostic(module_context);
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

pub struct ModuleContext {
    pub source_id: SourceId,

    pub symbols: Symbols,
    pub interner: Interner,
}
