use bayou_session::target_lexicon::Architecture;

pub mod codegen;
pub mod linker;

pub use cranelift_object::object;

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
