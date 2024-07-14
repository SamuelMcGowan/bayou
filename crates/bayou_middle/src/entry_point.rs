use bayou_interner::Interner;
use bayou_ir::symbols::{GlobalId, Symbols};
use bayou_ir::Type;
use bayou_session::diagnostics::prelude::*;
use bayou_session::sourcemap::SourceSpan;

pub enum EntrypointError {
    Missing,
    WrongSignature {
        expected: Type,

        found: Type,
        found_span: SourceSpan,
    },
}

impl IntoDiagnostic<()> for EntrypointError {
    fn into_diagnostic(self, _cx: &()) -> Diagnostic {
        match self {
            EntrypointError::Missing => Diagnostic::error().with_message("`main` function missing"),

            EntrypointError::WrongSignature {
                expected,
                found,
                found_span,
            } => Diagnostic::error()
                .with_message(format!(
                    "expected main function with return type {expected:?}, \
                    but it returned type {found:?}"
                ))
                .with_snippet(Snippet::primary(
                    "main function has wrong signature",
                    found_span.source_id,
                    found_span.span,
                )),
        }
    }
}

pub fn check_entrypoint(symbols: &Symbols, interner: &Interner) -> Result<(), EntrypointError> {
    let main_ident_str = interner
        .get_interned("main")
        .ok_or(EntrypointError::Missing)?;

    let main_func_id = symbols
        .lookup_global(main_ident_str)
        .and_then(GlobalId::as_func)
        .ok_or(EntrypointError::Missing)?;

    let func = &symbols.funcs[main_func_id];

    if func.ret_ty != Type::I64 {
        return Err(EntrypointError::WrongSignature {
            expected: Type::I64,

            found: func.ret_ty,
            found_span: func.ret_ty_span,
        });
    }

    Ok(())
}
