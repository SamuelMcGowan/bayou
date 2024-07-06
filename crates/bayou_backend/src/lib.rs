use bayou_ir::ir::{Module, ModuleContext, Package};
use bayou_session::diagnostics::DiagnosticEmitter;
use bayou_session::Session;
use codegen::Codegen;
use cranelift_object::object::write::Object;
use target_lexicon::{Architecture, Triple};

mod codegen;
mod layout;
mod linker;

// Re-exporting `object` here instead of using workspace dependencies
// so that we stay in sync with the version that cranelift uses.
pub use cranelift_object::object;
pub use linker::{Linker, LinkerError};

#[derive(thiserror::Error, Debug)]
pub enum BackendError {
    #[error("unsupported architecture: {0}")]
    UnsupportedArch(Architecture),

    #[error(transparent)]
    Module(#[from] cranelift_module::ModuleError),

    #[error(transparent)]
    Codegen(#[from] cranelift::codegen::CodegenError),
}

pub type BackendResult<T> = Result<T, BackendError>;

pub fn run_codegen<
    'sess,
    'm,
    D: DiagnosticEmitter,
    M: IntoIterator<Item = (&'m Module, &'m ModuleContext)>,
>(
    session: &'sess mut Session<D>,
    package_name: &str,
    target: Triple,

    modules: M,
) -> BackendResult<Object<'static>> {
    let mut codegen = Codegen::new(session, target, package_name)?;

    for (module, module_cx) in modules {
        codegen.compile_module(module, module_cx)?;
    }

    codegen.finish().map(|obj| obj.object)
}

pub fn run_codegen_new<D: DiagnosticEmitter>(
    session: &mut Session<D>,
    package: &Package,
) -> BackendResult<Object<'static>> {
    // TODO: refactor codegen to fit new model
    let mut codegen = Codegen::new(session, session.target.clone(), &package.name)?;
    codegen.compile_package(package)?;
    codegen.finish().map(|obj| obj.object)
}
