use bayou_backend::codegen::Codegen;
use bayou_backend::ObjectProduct;
use bayou_common::keyvec::{declare_key_type, KeyVec};
use bayou_diagnostic::sources::{Source as _, SourceMap as _};
use bayou_frontend::lexer::Lexer;
use bayou_frontend::parser::Parser;
use bayou_ir::ir::{Module, ModuleContext};
use bayou_ir::symbols::Symbols;
use bayou_middle::passes::entry_point::check_entrypoint;
use bayou_middle::passes::type_check::TypeChecker;
use bayou_middle::resolver::Resolver;
use bayou_session::diagnostics::DiagnosticEmitter;
use bayou_session::sourcemap::SourceId;
use bayou_session::Session;

use crate::{CompilerError, CompilerResult};

declare_key_type! { pub struct ModuleId; }

impl ModuleId {
    pub fn root() -> Self {
        use bayou_common::keyvec::Key;
        Self::from_usize(0)
    }
}

pub fn compile_package<D: DiagnosticEmitter>(
    session: &mut Session<D>,
    name: impl Into<String>,
    root_source_id: SourceId,
) -> CompilerResult<ObjectProduct> {
    let mut compilation = PackageCompilation {
        name: name.into(),

        module_irs: KeyVec::new(),
        module_contexts: KeyVec::new(),
    };

    compilation.parse_module_and_submodules(session, root_source_id)?;
    compilation.compile(session)
}

struct PackageCompilation {
    name: String,

    // These are separate so that info about other modules
    // can be accessed while mutating a module.
    module_irs: KeyVec<ModuleId, Module>,
    module_contexts: KeyVec<ModuleId, ModuleContext>,
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

        let mut module_cx = ModuleContext {
            source_id,
            symbols: Symbols::default(),
        };

        // name resolution
        let resolver = Resolver::new(&mut module_cx);
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
        let _ = self.module_contexts.insert(module_cx);

        Ok(module_id)
    }

    fn compile<D: DiagnosticEmitter>(
        mut self,
        session: &mut Session<D>,
    ) -> CompilerResult<ObjectProduct> {
        // type checking
        for (ir, cx) in self.module_irs.iter_mut().zip(&mut self.module_contexts) {
            let type_checker = TypeChecker::new(cx);
            let type_errors = type_checker.run(ir);

            session.report_all(type_errors, cx.source_id)?;
        }

        let root_cx = &self.module_contexts[ModuleId::root()];
        if let Err(err) = check_entrypoint(root_cx, &session.interner) {
            let source_id = self.module_contexts[ModuleId::root()].source_id;

            let _ = session.report(err, source_id);
            return Err(CompilerError::HadErrors);
        }

        // codegen
        let mut codegen = Codegen::new(session, session.target.clone(), &self.name)?;
        for (ir, cx) in self.module_irs.iter().zip(&self.module_contexts) {
            codegen.compile_module(ir, cx)?;
        }

        let object = codegen.finish()?;

        Ok(object)
    }
}
