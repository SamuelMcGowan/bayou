pub mod ast;
#[allow(clippy::module_inception)]
pub mod ir;
pub mod token;

pub type InternedStr = lasso::Spur;
pub type Interner = lasso::Rodeo;

derive_alias! {
    #[derive(Node!)] = #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)];
    #[derive(NodeCopy!)] = #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)];
}
