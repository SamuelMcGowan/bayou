use crate::compiler::{ModuleId, PackageCompilation};
use crate::ir::ir::Type;
use crate::ir::Interner;
use crate::symbols::GlobalId;

// TODO: store spans
pub enum EntrypointError {
    Missing,
    WrongSignature { expected: Type, found: Type },
}

pub fn check_entrypoint(
    pkg: &PackageCompilation,
    interner: &Interner,
) -> Result<(), EntrypointError> {
    let root_module_cx = &pkg.module_cxs[ModuleId::root()];

    let main_ident = interner.get("main").ok_or(EntrypointError::Missing)?;

    let main_func_id = root_module_cx
        .symbols
        .lookup_global(main_ident)
        .and_then(GlobalId::as_func)
        .ok_or(EntrypointError::Missing)?;

    let func = &root_module_cx.symbols.funcs[main_func_id];

    if func.ret_ty != Type::I64 {
        return Err(EntrypointError::WrongSignature {
            expected: Type::I64,
            found: func.ret_ty,
        });
    }

    Ok(())
}
