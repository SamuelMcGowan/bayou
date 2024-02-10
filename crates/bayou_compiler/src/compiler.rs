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
use crate::utils::keyvec::{declare_key_type, KeyVec};
use crate::{CompilerError, CompilerResult};

declare_key_type! { pub struct ModuleId; }

pub struct Compiler<D: DiagnosticEmitter> {
    pub sources: SourceMap,
    pub diagnostics: D,

    modules: KeyVec<ModuleId, Module>,
    module_cxts: KeyVec<ModuleId, ModuleContext>,

    triple: Triple,
}

impl<D: DiagnosticEmitter> Compiler<D> {
    pub fn new(diagnostics: D, triple: Triple) -> Self {
        Self {
            sources: SourceMap::default(),
            diagnostics,

            modules: KeyVec::new(),
            module_cxts: KeyVec::new(),

            triple,
        }
    }

    pub fn add_module(
        &mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> CompilerResult<ModuleId> {
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

        let id = self.modules.insert(ast);
        let _ = self.module_cxts.insert(module_context);

        Ok(id)
    }

    pub fn compile(&mut self) -> CompilerResult<ObjectProduct> {
        // FIXME: choose proper name
        let mut codegen = Codegen::new(self.triple.clone(), "replaceme")?;

        for (module, cx) in self.modules.iter().zip(self.module_cxts.iter()) {
            codegen.compile_module(module, cx)?;
        }

        Ok(codegen.finish())
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
