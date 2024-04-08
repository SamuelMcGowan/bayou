use bayou_ir::symbols::Symbols;
use bayou_session::sourcemap::SourceId;

pub mod passes;
pub mod resolver;

pub struct ModuleCompilation {
    pub source_id: SourceId,
    pub symbols: Symbols,
}
