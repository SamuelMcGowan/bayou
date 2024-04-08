pub mod diagnostics;
pub mod sourcemap;

pub use lasso;

pub type InternedStr = lasso::Spur;
pub type Interner = lasso::Rodeo;
