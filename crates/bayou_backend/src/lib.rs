use target_lexicon::Architecture;

pub mod codegen;
pub mod linker;

// Re-exporting here instead of using workspace dependencies
// so that we stay in sync with the version that cranelift uses.
pub use cranelift_object::{object, ObjectProduct};

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
