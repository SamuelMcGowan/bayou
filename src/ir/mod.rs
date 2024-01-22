pub mod token;
pub mod vars;

pub mod ast {
    use super::vars::PlaceRef;
    use crate::session::InternedStr;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Module {
        pub item: Item,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Item {
        FuncDecl(FuncDecl),
        ParseError,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FuncDecl {
        pub name: InternedStr,
        pub statement: Stmt,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Stmt {
        Return(Expr),
        ParseError,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Expr {
        pub kind: ExprKind,

        pub place: Option<PlaceRef>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ExprKind {
        Constant(u64),

        UnOp {
            op: UnOp,
            expr: Box<Expr>,
        },

        BinOp {
            op: BinOp,
            lhs: Box<Expr>,
            rhs: Box<Expr>,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum BinOp {
        CmpOp(CmpOp),

        Add,
        Sub,
        Mul,
        Div,
        Mod,

        BitwiseAnd,
        BitwiseOr,
        BitwiseXor,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum UnOp {
        Neg,
        BitwiseNot,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum CmpOp {
        Eq,
        NotEq,

        Gt,
        Lt,
        GtEq,
        LtEq,
    }
}
