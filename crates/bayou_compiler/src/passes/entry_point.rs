use bayou_ir::symbols::GlobalId;
use bayou_ir::Type;
use bayou_session::diagnostics::prelude::*;

use crate::compilation::ModuleCompilation;

// TODO: store spans
pub enum EntrypointError {
    Missing,
    WrongSignature { expected: Type, found: Type },
}

impl IntoDiagnostic for EntrypointError {
    fn into_diagnostic(self, _source_id: SourceId, _interner: &Interner) -> Diagnostic {
        match self {
            EntrypointError::Missing => Diagnostic::error().with_message("`main` function missing"),
            EntrypointError::WrongSignature { expected, found } => Diagnostic::error()
                .with_message(format!(
                    "expected main function with return type {expected:?}, but it returned type {found:?}"
                )),
        }
    }
}

pub fn check_entrypoint(
    root_module_compilation: &ModuleCompilation,
    interner: &Interner,
) -> Result<(), EntrypointError> {
    let main_ident = interner.get("main").ok_or(EntrypointError::Missing)?;

    let main_func_id = root_module_compilation
        .symbols
        .lookup_global(main_ident)
        .and_then(GlobalId::as_func)
        .ok_or(EntrypointError::Missing)?;

    let func = &root_module_compilation.symbols.funcs[main_func_id];

    if func.ret_ty != Type::I64 {
        return Err(EntrypointError::WrongSignature {
            expected: Type::I64,
            found: func.ret_ty,
        });
    }

    Ok(())
}
