use bayou_diagnostic::sources::{Source as _, SourceMap as _};
use cranelift_object::ObjectProduct;

use crate::codegen::Codegen;
use crate::diagnostics::DiagnosticEmitter;
use crate::ir::ir::Module;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::passes::entry_point::check_entrypoint;
use crate::passes::type_check::TypeChecker;
use crate::resolver::Resolver;
use crate::session::Session;
use crate::sourcemap::{Source, SourceId};
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

pub struct ModuleCompilation {
    pub source_id: SourceId,
    pub symbols: Symbols,
}

/// A package that is being compiled.
pub struct PackageCompilation {
    pub name: String,

    // These are separate so that info about other modules
    // can be accessed while mutating a module.
    pub module_irs: KeyVec<ModuleId, Module>,
    pub module_compilations: KeyVec<ModuleId, ModuleCompilation>,
}

impl PackageCompilation {
    /// Parse all modules and resolve names.
    pub fn start<D>(
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

        let lexer = Lexer::new(source.source_str(), &mut session.interner);
        let (tokens, lexer_errors) = lexer.lex();
        session.report_all(lexer_errors, source_id)?;

        let parser = Parser::new(tokens);
        let (ast, parse_errors) = parser.parse();
        session.report_all(parse_errors, source_id)?;

        let mut module_compilation = ModuleCompilation {
            source_id,
            symbols: Symbols::default(),
        };

        let mut module_irs = KeyVec::new();
        let mut module_compilations = KeyVec::new();

        // name resolution
        let resolver = Resolver::new(&mut module_compilation);
        let ir = match resolver.run(ast) {
            Ok(ir) => ir,
            Err(errors) => {
                let _ = session.report_all(errors, source_id);
                return Err(CompilerError::HadErrors);
            }
        };

        // will have root id
        let _ = module_irs.insert(ir);
        let _ = module_compilations.insert(module_compilation);

        Ok(Self {
            name,

            module_irs,
            module_compilations,
        })
    }

    pub fn compile<D: DiagnosticEmitter>(
        mut self,
        session: &mut Session<D>,
    ) -> CompilerResult<ObjectProduct> {
        // type checking
        for (ir, compilation) in self
            .module_irs
            .iter_mut()
            .zip(&mut self.module_compilations)
        {
            let type_checker = TypeChecker::new(compilation);
            let type_errors = type_checker.run(ir);

            session.report_all(type_errors, compilation.source_id)?;
        }

        if let Err(err) = check_entrypoint(&self, &session.interner) {
            let source_id = self.module_compilations[ModuleId::root()].source_id;

            let _ = session.report(err, source_id);
            return Err(CompilerError::HadErrors);
        }

        // codegen
        let mut codegen = Codegen::new(session, session.target.clone(), &self.name)?;
        for (ir, compilation) in self.module_irs.iter().zip(&self.module_compilations) {
            codegen.compile_module(ir, compilation)?;
        }

        let object = codegen.finish()?;

        Ok(object)
    }
}
