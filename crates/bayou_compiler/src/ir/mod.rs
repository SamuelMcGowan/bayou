pub mod ast;
pub mod token;
pub mod vars;

pub type InternedStr = lasso::Spur;
pub type Interner = lasso::Rodeo;

derive_alias! {
    #[derive(Node!)] = #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)];
    #[derive(NodeCopy!)] = #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)];
}
