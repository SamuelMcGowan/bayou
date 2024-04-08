use bayou_common::keyvec::{declare_key_type, KeyVec};
use bayou_diagnostic::sources::{Source as _, SourceMap as _};
use bayou_frontend::lexer::Lexer;
use bayou_frontend::parser::Parser;
use bayou_ir::ir::Module;
use bayou_ir::symbols::Symbols;
use bayou_session::diagnostics::DiagnosticEmitter;
use bayou_session::sourcemap::SourceId;
use cranelift_object::ObjectProduct;

use crate::codegen::Codegen;
use crate::passes::entry_point::check_entrypoint;
use crate::passes::type_check::TypeChecker;
use crate::resolver::Resolver;
use crate::session::Session;
use crate::{CompilerError, CompilerResult};

declare_key_type! { pub struct ModuleId; }

impl ModuleId {
    pub fn root() -> Self {
        use bayou_common::keyvec::Key;
        Self::from_usize(0)
    }
}

pub struct ModuleCompilation {
    pub source_id: SourceId,
    pub symbols: Symbols,
}

pub fn compile_package<D: DiagnosticEmitter>(
    session: &mut Session<D>,
    name: impl Into<String>,
    root_source_id: SourceId,
) -> CompilerResult<ObjectProduct> {
    let mut compilation = PackageCompilation {
        name: name.into(),

        module_irs: KeyVec::new(),
        module_compilations: KeyVec::new(),
    };

    compilation.parse_module_and_submodules(session, root_source_id)?;
    compilation.compile(session)
}

struct PackageCompilation {
    name: String,

    // These are separate so that info about other modules
    // can be accessed while mutating a module.
    module_irs: KeyVec<ModuleId, Module>,
    module_compilations: KeyVec<ModuleId, ModuleCompilation>,
}

impl PackageCompilation {
    fn parse_module_and_submodules<D: DiagnosticEmitter>(
        &mut self,
        session: &mut Session<D>,
        source_id: SourceId,
    ) -> CompilerResult<ModuleId> {
        let source = session
            .sources
            .get_source(source_id)
            .expect("source id not in sources");

        let mut had_errors = false;

        let lexer = Lexer::new(source.source_str(), &mut session.interner);
        let (tokens, lexer_errors) = lexer.lex();

        let parser = Parser::new(tokens);
        let (ast, parse_errors) = parser.parse();

        had_errors |= session.report_all(lexer_errors, source_id).is_err();
        had_errors |= session.report_all(parse_errors, source_id).is_err();

        let mut module_compilation = ModuleCompilation {
            source_id,
            symbols: Symbols::default(),
        };

        // name resolution
        let resolver = Resolver::new(&mut module_compilation);
        let ir = match resolver.run(ast) {
            Ok(ir) => ir,
            Err(errors) => {
                let _ = session.report_all(errors, source_id);
                return Err(CompilerError::HadErrors);
            }
        };

        if had_errors {
            return Err(CompilerError::HadErrors);
        }

        let module_id = self.module_irs.insert(ir);
        let _ = self.module_compilations.insert(module_compilation);

        Ok(module_id)
    }

    fn compile<D: DiagnosticEmitter>(
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

        let root_compilation = &self.module_compilations[ModuleId::root()];
        if let Err(err) = check_entrypoint(root_compilation, &session.interner) {
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
