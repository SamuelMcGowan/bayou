use bayou_diagnostic::sources::{Source as _, SourceMap as _};
use bayou_diagnostic::DiagnosticKind;
use cranelift_object::ObjectProduct;
use target_lexicon::Triple;

use crate::codegen::Codegen;
use crate::diagnostics::{DiagnosticEmitter, IntoDiagnostic};
use crate::ir::ir::Module;
use crate::ir::Interner;
use crate::parser::Parser;
use crate::passes::entry_point::check_entrypoint;
use crate::passes::type_check::TypeChecker;
use crate::platform::Platform;
use crate::resolver::Resolver;
use crate::sourcemap::{Source, SourceId, SourceMap};
use crate::symbols::Symbols;
use crate::utils::keyvec::{declare_key_type, KeyVec};
use crate::{CompilerError, CompilerResult};

declare_key_type! { pub struct ModuleId; }

impl ModuleId {
    pub fn root() -> Self {
        use crate::utils::keyvec::Key;
        Self::from_usize(0)
    }
}

pub struct Session<D: DiagnosticEmitter> {
    pub sources: SourceMap,
    pub diagnostics: D,

    pub platform: Platform,
}

impl<D: DiagnosticEmitter> Session<D> {
    pub fn new(diagnostics: D, target: Triple) -> CompilerResult<Self> {
        Ok(Self {
            sources: SourceMap::default(),
            diagnostics,

            platform: Platform::new(target)?,
        })
    }

    pub fn report_all<Errs>(
        &mut self,
        diagnostics: Errs,
        module_cx: &ModuleCx,
    ) -> CompilerResult<()>
    where
        Errs: IntoIterator,
        Errs::Item: IntoDiagnostic,
    {
        let mut had_errors = false;

        for diagnostic in diagnostics {
            let diagnostic = diagnostic.into_diagnostic(module_cx);
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

pub struct ModuleCx {
    pub source_id: SourceId,

    pub symbols: Symbols,
    pub interner: Interner,
}

/// A package that is being compiled.
pub struct PackageCompilation {
    // FIXME: make these private
    pub name: String,

    pub irs: KeyVec<ModuleId, Module>,
    pub module_cxs: KeyVec<ModuleId, ModuleCx>,
}

impl PackageCompilation {
    pub fn parse<D>(
        session: &mut Session<D>,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> CompilerResult<Self>
    where
        D: DiagnosticEmitter,
    {
        let name = name.into();

        let source_id = session.sources.insert(Source::new(&name, source));
        let source = session.sources.get_source(source_id).unwrap();

        let parser = Parser::new(source.source_str());
        let (ast, interner, parse_errors) = parser.parse();

        let mut module_cx = ModuleCx {
            source_id,
            symbols: Symbols::default(),
            interner,
        };

        session.report_all(parse_errors, &module_cx)?;

        let mut irs = KeyVec::new();
        let mut module_cxs = KeyVec::new();

        // name resolution
        let resolver = Resolver::new(&mut module_cx);
        let ir = match resolver.run(ast) {
            Ok(ir) => ir,
            Err(errors) => {
                let _ = session.report_all(errors, &module_cx);
                return Err(CompilerError::HadErrors);
            }
        };

        // will have root id
        let _ = irs.insert(ir);
        let _ = module_cxs.insert(module_cx);

        Ok(PackageCompilation {
            name,
            irs,
            module_cxs,
        })
    }

    pub fn compile<D: DiagnosticEmitter>(
        mut self,
        session: &mut Session<D>,
    ) -> CompilerResult<ObjectProduct> {
        let mut codegen = Codegen::new(session.platform.target.clone(), &self.name)?;

        // type checking
        for (ir, module_cx) in self.irs.iter_mut().zip(&mut self.module_cxs) {
            let type_checker = TypeChecker::new(module_cx);
            let type_errors = type_checker.run(ir);
            session.report_all(type_errors, module_cx)?;
        }

        if let Err(err) = check_entrypoint(&self) {
            let module_cx = &self.module_cxs[ModuleId::root()];
            session.report_all([err], module_cx)?;
        }

        // codegen
        for (ir, module_cx) in self.irs.iter().zip(&self.module_cxs) {
            codegen.compile_module(ir, module_cx)?;
        }

        let object = codegen.finish()?;

        Ok(object)
    }
}
