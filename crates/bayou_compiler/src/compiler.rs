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
        module_cx: &ModuleCx,
    ) -> CompilerResult<()> {
        let diagnostic = diagnostic.into_diagnostic(module_cx.source_id, &self.interner);
        let kind = diagnostic.kind;

        self.diagnostics.emit_diagnostic(diagnostic, &self.sources);

        if kind < DiagnosticKind::Error {
            Ok(())
        } else {
            Err(CompilerError::HadErrors)
        }
    }

    pub fn report_all<I>(&mut self, diagnostics: I, module_cx: &ModuleCx) -> CompilerResult<()>
    where
        I: IntoIterator,
        I::Item: IntoDiagnostic,
    {
        let mut had_error = false;

        for diagnostic in diagnostics {
            let diagnostic = diagnostic.into_diagnostic(module_cx.source_id, &self.interner);
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

// TODO: rename
pub struct ModuleCx {
    pub source_id: SourceId,
    pub symbols: Symbols,
}

/// A package that is being compiled.
pub struct PackageCompilation {
    pub name: String,
    pub target: Triple,

    // These are separate so that info about other modules
    // can be accessed while mutating a module.
    pub module_irs: KeyVec<ModuleId, Module>,
    pub module_cxs: KeyVec<ModuleId, ModuleCx>,
}

impl PackageCompilation {
    /// Parse all modules and resolve names.
    pub fn start<D>(
        session: &mut Session<D>,
        target: Triple,

        name: impl Into<String>,
        source: impl Into<String>,
    ) -> CompilerResult<Self>
    where
        D: DiagnosticEmitter,
    {
        let name = name.into();

        let source_id = session.sources.insert(Source::new(&name, source));
        let source = session.sources.get_source(source_id).unwrap();

        let parser = Parser::new(source.source_str(), &mut session.interner);
        let (ast, parse_errors) = parser.parse();

        let mut module_cx = ModuleCx {
            source_id,
            symbols: Symbols::default(),
        };

        session.report_all(parse_errors, &module_cx)?;

        let mut module_irs = KeyVec::new();
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
        let _ = module_irs.insert(ir);
        let _ = module_cxs.insert(module_cx);

        Ok(Self {
            name,
            target,

            module_irs,
            module_cxs,
        })
    }

    pub fn compile<D: DiagnosticEmitter>(
        mut self,
        session: &mut Session<D>,
    ) -> CompilerResult<ObjectProduct> {
        // type checking
        for (ir, module_cx) in self.module_irs.iter_mut().zip(&mut self.module_cxs) {
            let type_checker = TypeChecker::new(module_cx);
            let type_errors = type_checker.run(ir);

            session.report_all(type_errors, module_cx)?;
        }

        if let Err(err) = check_entrypoint(&self, &session.interner) {
            let module_cx = &self.module_cxs[ModuleId::root()];

            let _ = session.report(err, module_cx);
            return Err(CompilerError::HadErrors);
        }

        // codegen
        let mut codegen = Codegen::new(session, self.target, &self.name)?;
        for (ir, module_cx) in self.module_irs.iter().zip(&self.module_cxs) {
            codegen.compile_module(ir, module_cx)?;
        }

        let object = codegen.finish()?;

        Ok(object)
    }
}
