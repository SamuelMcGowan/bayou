use super::{BinOp, ParseError, ParseResult, Parser};
use crate::frontend::lexer::Peek;
use crate::ir::ast::*;
use crate::ir::token::Token;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    Lowest,

    LogicalOr,
    LogicalAnd,

    Equality,
    Comparison,

    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,

    Term,
    Factor,

    Unary,
    // Field,
    // Call,
}

impl BinOp {
    fn should_parse_in_prec(&self, in_prec: Prec) -> bool {
        let prec = self.prec();
        prec > in_prec || self.r_assoc() && prec == in_prec
    }

    fn prec(&self) -> Prec {
        match self {
            // Self::LogicalOr => Prec::LogicalOr,
            // Self::LogicalAnd => Prec::LogicalAnd,

            // Self::Equal | Self::NotEqual => Prec::Equality,
            // Self::Gt | Self::Lt | Self::GtEq | Self::LtEq => Prec::Comparison,
            Self::BitwiseAnd => Prec::BitwiseAnd,
            Self::BitwiseXor => Prec::BitwiseXor,
            Self::BitwiseOr => Prec::BitwiseOr,

            Self::Add | Self::Sub => Prec::Term,
            Self::Mul | Self::Div | Self::Mod => Prec::Factor,
            // Self::Field => Prec::Field,
            // Self::Call | Self::Index => Prec::Call,
        }
    }

    fn r_assoc(&self) -> bool {
        // nothing for now, but best to keep the framework in place
        false
    }
}

impl Parser<'_> {
    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_prec(Prec::Lowest)
    }

    fn parse_prec(&mut self, prec: Prec) -> ParseResult<Expr> {
        let mut expr = self.parse_lhs()?;

        while let Some(op) = self.peek_bin_op(prec) {
            // `get_op` doesn't consume a token because
            // some (as of yet unimplemented) operations need to consume
            // the token themselves
            self.lexer.next();

            let rhs = self.parse_prec(op.prec())?;

            expr = Expr::new(ExprKind::BinOp {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            });
        }

        Ok(expr)
    }

    fn parse_lhs(&mut self) -> ParseResult<Expr> {
        match self.lexer.next() {
            Some(Token::Integer(n)) => Ok(Expr::new(ExprKind::Constant(n))),

            Some(Token::Sub) => {
                let expr = self.parse_prec(Prec::Unary)?;
                Ok(Expr::new(ExprKind::UnOp {
                    op: UnOp::Negate,
                    expr: Box::new(expr),
                }))
            }

            Some(Token::BitwiseInvert) => {
                let expr = self.parse_prec(Prec::Unary)?;
                Ok(Expr::new(ExprKind::UnOp {
                    op: UnOp::BitwiseInvert,
                    expr: Box::new(expr),
                }))
            }

            Some(Token::LParen) => {
                let expr = self.parse_or_recover(Self::parse_expr, |parser| {
                    parser.recover_until(Token::RParen);
                    Expr::new(ExprKind::ParseError)
                });

                self.expect(Token::RParen)?;

                Ok(expr)
            }

            other => Err(ParseError::expected("an expression", other)),
        }
    }

    fn peek_bin_op(&self, prec: Prec) -> Option<BinOp> {
        let op = match self.lexer.peek()? {
            Token::Add => BinOp::Add,
            Token::Sub => BinOp::Sub,
            Token::Mul => BinOp::Mul,
            Token::Div => BinOp::Div,
            Token::Mod => BinOp::Mod,

            Token::BitwiseAnd => BinOp::BitwiseAnd,
            Token::BitwiseOr => BinOp::BitwiseOr,
            Token::BitwiseXor => BinOp::BitwiseXor,

            _ => return None,
        };

        op.should_parse_in_prec(prec).then_some(op)
    }
}
