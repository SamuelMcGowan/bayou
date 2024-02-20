use bayou_diagnostic::sources::{Source as _, SourceMap as _};
use bayou_diagnostic::DiagnosticKind;
use cranelift_object::ObjectProduct;
use target_lexicon::Triple;

use crate::codegen::Codegen;
use crate::diagnostics::{DiagnosticEmitter, IntoDiagnostic};
use crate::ir::ast::Module;
use crate::ir::Interner;
use crate::parser::Parser;
use crate::passes::type_check::TypeChecker;
use crate::resolver::Resolver;
use crate::sourcemap::{Source, SourceId, SourceMap};
use crate::symbols::Symbols;
use crate::utils::keyvec::{declare_key_type, KeyVec};
use crate::{CompilerError, CompilerResult};

declare_key_type! { pub struct ModuleId; }

pub struct Compiler<D: DiagnosticEmitter> {
    name: String,
    sources: SourceMap,
    diagnostics: D,

    modules: KeyVec<ModuleId, Module>,
    module_cxs: KeyVec<ModuleId, ModuleContext>,

    triple: Triple,
}

impl<D: DiagnosticEmitter> Compiler<D> {
    pub fn new(name: impl Into<String>, diagnostics: D, triple: Triple) -> Self {
        Self {
            name: name.into(),
            sources: SourceMap::default(),
            diagnostics,

            modules: KeyVec::new(),
            module_cxs: KeyVec::new(),

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

        let _ = self.module_cxs.insert(module_context);
        let module_id = self.modules.insert(ast);

        Ok(module_id)
    }

    pub fn compile(mut self) -> CompilerResult<ObjectProduct> {
        use std::mem::take;

        let mut codegen = Codegen::new(self.triple.clone(), &self.name)?;

        for (ast, mut module_cx) in take(&mut self.modules)
            .into_iter()
            .zip(take(&mut self.module_cxs))
        {
            let resolver = Resolver::new(&mut module_cx);
            let mut ir = match resolver.run(ast) {
                Ok(ir) => ir,
                Err(errors) => {
                    let _ = self.report(errors, &module_cx);
                    return Err(CompilerError::HadErrors);
                }
            };

            let type_checker = TypeChecker::new(&mut module_cx);
            let type_errors = type_checker.run(&mut ir);
            self.report(type_errors, &module_cx)?;

            codegen.compile_module(&ir, &module_cx)?;
        }

        let object = codegen.finish()?;

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
