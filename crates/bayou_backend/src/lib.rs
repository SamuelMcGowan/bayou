use bayou_ir::ir::Package;
use bayou_session::CodegenSession;
use codegen::Codegen;
use cranelift_object::object::write::Object;
use target_lexicon::Architecture;

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

pub fn run_codegen<S: CodegenSession>(
    session: &mut S,
    package: &Package,
) -> BackendResult<Object<'static>> {
    // TODO: refactor codegen to fit new model
    let mut codegen = Codegen::new(session.target_triple().clone(), &package.name)?;
    codegen.compile_package(package)?;
    Ok(codegen.finish().object)
}
