use crate::CompilerResult;
use bayou_backend::object::write::Object;
use bayou_ir::ir::Package;
use bayou_middle::type_check::TypeChecker;
use bayou_session::Session;

pub fn compile_package<S: Session>(
    session: &mut S,
    config: S::PackageConfig,
) -> CompilerResult<Object<'static>> {
    let mut package_session = session.build_package_session(config);

    let (mut module_tree, parsed_modules, errors) =
        bayou_frontend::load_and_parse_modules(session, &mut package_session);
    session.report_all(errors, &package_session.interner)?;

    let (mut ir, mut symbols, errors) =
        bayou_frontend::lower(&parsed_modules, &mut module_tree, &package_session.interner);
    session.report_all(errors, &package_session.interner)?;

    let type_checker = TypeChecker::new(&mut symbols);

    // TODO: does this need mutable access to the IR?
    let type_errors = type_checker.run(&mut ir);
    session.report_all(type_errors, &())?;

    if let Err(err) = bayou_middle::entry_point::check_entrypoint(&ir, &symbols) {
        session.report(err, &())?;
    }

    // TODO: remove `Package` type.
    let package = Package {
        name: package_session.name,
        ir,
        symbols,
        interner: package_session.interner,
    };
    let object = bayou_backend::run_codegen(session, &package)?;

    Ok(object)
}
