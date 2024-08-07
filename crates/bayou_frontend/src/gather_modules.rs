use bayou_interner::Interner;
use bayou_ir::IdentWithSource;
use bayou_session::{
    diagnostics::{prelude::*, sources::Source as _},
    module_loader::{ModuleLoader, ModuleLoaderError, ModulePath},
    sourcemap::{Source, SourceId, SourceSpan},
    PackageSession, Session,
};

use crate::{
    ast,
    lexer::Lexer,
    module_tree::{GlobalId, ModuleId, ModuleTree},
    parser::Parser,
    LexerError, ParseError,
};

#[derive(Debug, serde::Serialize)]
pub enum GatherModulesError {
    ModuleLoaderError(ModuleLoaderError, Option<SourceSpan>),
    InvalidModuleName(IdentWithSource),

    LexerError(LexerError, SourceId),
    ParseError(ParseError, SourceId),

    DuplicateGlobal(IdentWithSource),
}

impl IntoDiagnostic<Interner> for GatherModulesError {
    fn into_diagnostic(self, interner: &Interner) -> bayou_session::diagnostics::Diagnostic {
        match self {
            Self::ModuleLoaderError(err, source_span) => {
                err.into_diagnostic(&(source_span, interner))
            }

            Self::InvalidModuleName(name) => Diagnostic::error()
                .with_message("invalid module name")
                .with_snippet(Snippet::primary(
                    format!(
                        "`{}` is not a valid name for a module",
                        interner.get_str(name.istr).unwrap()
                    ),
                    name.span.source_id,
                    name.span.span,
                )),

            Self::LexerError(err, source_id) => err.into_diagnostic(&source_id),
            Self::ParseError(err, source_id) => err.into_diagnostic(&source_id),

            Self::DuplicateGlobal(_err) => {
                // TODO: store source spans in duplicate global error
                todo!()
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ParsedModule {
    pub module_id: ModuleId,
    pub source_id: SourceId,

    pub ast: ast::Module,
}

pub struct ModuleGatherer<'a, S: Session> {
    session: &'a mut S,
    package_session: &'a mut PackageSession<S>,

    errors: Vec<GatherModulesError>,
}

impl<'a, S: Session> ModuleGatherer<'a, S> {
    pub fn new(session: &'a mut S, package_session: &'a mut PackageSession<S>) -> Self {
        Self {
            session,
            package_session,

            errors: vec![],
        }
    }

    pub fn run(mut self) -> (ModuleTree, Vec<ParsedModule>, Vec<GatherModulesError>) {
        let mut module_tree = ModuleTree::new();
        let mut parsed_modules = vec![];

        let mut modules_to_load = vec![(module_tree.root_id(), None)];

        while let Some((module_id, span)) = modules_to_load.pop() {
            let module_path = &module_tree.entry(module_id).path;

            let Some((source_id, ast)) = self.parse_module(module_path, span) else {
                continue;
            };

            let submodule_names = ast.items.iter().filter_map(|item| match item {
                ast::Item::Submodule(name) => Some(name.with_source(source_id)),
                _ => None,
            });

            for submodule_name in submodule_names {
                // A submodule of the root module can't be called `main` because it would
                // clash with the root module. For now just check no modules are called `main`.
                // TODO: handle cyclic modules properly?
                if &self.package_session.interner[submodule_name.istr] == "main" {
                    self.errors
                        .push(GatherModulesError::InvalidModuleName(submodule_name));
                    continue;
                }

                let submodule_id = match module_tree.insert_module(module_id, submodule_name) {
                    Ok(id) => id,

                    Err(GlobalId::Module(first_module_id)) => {
                        // module must have an identifier, otherwise there would be no error
                        let first_module_ident = module_tree.entry(first_module_id).ident.unwrap();

                        self.errors
                            .push(GatherModulesError::DuplicateGlobal(first_module_ident));

                        continue;
                    }

                    Err(_) => unreachable!(),
                };

                modules_to_load.push((submodule_id, Some(submodule_name.span)));
            }

            parsed_modules.push(ParsedModule {
                module_id,
                source_id,
                ast,
            });
        }

        (module_tree, parsed_modules, self.errors)
    }

    fn parse_module(
        &mut self,
        module_path: &ModulePath,
        def_source_span: Option<SourceSpan>,
    ) -> Option<(SourceId, ast::Module)> {
        let source_string = match self
            .package_session
            .module_loader
            .load_module(module_path, &self.package_session.interner)
        {
            Ok(s) => s,
            Err(err) => {
                self.errors
                    .push(GatherModulesError::ModuleLoaderError(err, def_source_span));
                return None;
            }
        };

        let (source_id, source) = self.session.source_map_mut().insert_and_get(Source {
            name: module_path
                .display(&self.package_session.interner)
                .to_string(),
            source: source_string,
        });

        let (tokens, lexer_errors) =
            Lexer::new(source.source_str(), &self.package_session.interner).lex();
        self.errors.extend(
            lexer_errors
                .into_iter()
                .map(|err| GatherModulesError::LexerError(err, source_id)),
        );

        let (ast, parse_errors) = Parser::new(tokens).parse();
        self.errors.extend(
            parse_errors
                .into_iter()
                .map(|err| GatherModulesError::ParseError(err, source_id)),
        );

        Some((source_id, ast))
    }
}

#[cfg(test)]
mod tests {
    use bayou_ir::{ir::PackageIr, symbols::Symbols};
    use bayou_session::{Session, TestSession, TestSessionConfig};
    use bayou_utils::assert_yaml_snapshot_with_source;

    use crate::NameError;

    fn lower(source: &str) -> (PackageIr, Symbols, Vec<NameError>) {
        let mut session = TestSession::new();
        let mut package_session = session.build_package_session(TestSessionConfig::new(
            "test_package",
            [(String::from("package"), String::from(source))],
        ));

        let (mut module_tree, modules, errors) =
            crate::load_and_parse_modules(&mut session, &mut package_session);

        assert!(
            errors.is_empty(),
            "non-lowering errors while testing lowering"
        );

        crate::lower(&modules, &mut module_tree, &package_session.interner)
    }

    macro_rules! assert_lower {
        ($source:expr) => {{
            let source = $source;
            assert_yaml_snapshot_with_source!(source => lower(source));
        }};
    }

    #[test]
    fn basic_lower() {
        assert_lower!("func main() -> i64 { return 0; }");
    }
}
