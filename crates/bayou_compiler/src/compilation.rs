use bayou_backend::object::write::Object;
use bayou_diagnostic::sources::{Source as _, SourceMap as _};
use bayou_interner::Interner;
use bayou_ir::{ir::Package, symbols::Symbols};
use bayou_middle::{entry_point::check_entrypoint, type_check::TypeChecker};
use bayou_session::{diagnostics::DiagnosticEmitter, sourcemap::SourceId, Session};

use crate::{CompilerError, CompilerResult};

pub fn compile_package<D: DiagnosticEmitter>(
    session: &mut Session<D>,
    name: impl Into<String>,
    root_source_id: SourceId,
) -> CompilerResult<Object<'static>> {
    let mut compiler = PackageCompiler {
        package: Package {
            name: name.into(),

            items: vec![],
            symbols: Symbols::default(),
            interner: Interner::new(),
        },

        session,
    };

    compiler.parse_module_and_submodules(root_source_id)?;
    compiler.compile()
}

struct PackageCompiler<'a, D: DiagnosticEmitter> {
    package: Package,
    session: &'a mut Session<D>,
}

impl<D: DiagnosticEmitter> PackageCompiler<'_, D> {
    fn parse_module_and_submodules(&mut self, source_id: SourceId) -> CompilerResult<()> {
        let source = self
            .session
            .sources
            .get_source(source_id)
            .expect("source id not in sources");

        let (tokens, lexer_errors) =
            bayou_frontend::lex(source.source_str(), &mut self.package.interner);

        let (ast, parse_errors) = bayou_frontend::parse(tokens);

        let mut had_errors = false;
        had_errors |= self
            .session
            .report_all(lexer_errors, &self.package.interner)
            .is_err();
        had_errors |= self
            .session
            .report_all(parse_errors, &self.package.interner)
            .is_err();

        if had_errors {
            return Err(CompilerError::HadErrors);
        }

        let ir = bayou_frontend::lower_new(ast, &mut self.package.symbols).map_err(|errors| {
            let _ = self.session.report_all(errors, &self.package.interner);
            CompilerError::HadErrors
        })?;

        self.package.items = ir.items;

        Ok(())
    }

    fn compile(mut self) -> CompilerResult<Object<'static>> {
        // type checking
        let type_checker = TypeChecker::new(&mut self.package.symbols);
        let type_errors = type_checker.run(&mut self.package.items);
        self.session
            .report_all(type_errors, &self.package.interner)?;

        // check entrypoint
        // FIXME: ensure `main` function is in root module
        if let Err(err) = check_entrypoint(&self.package.symbols, &self.package.interner) {
            let _ = self.session.report(err, &self.package.interner);
            return Err(CompilerError::HadErrors);
        }

        let object = bayou_backend::run_codegen(self.session, &self.package)?;

        Ok(object)
    }
}
