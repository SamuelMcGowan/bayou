use bayou_ir::{BinOp, Ident, UnOp};
use bayou_utils::peek::Peek;

use super::{ParseResult, Parser};
use crate::ast::*;
use crate::token::*;

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

fn should_parse_binop_in_prec(binop: &BinOp, in_prec: Prec) -> bool {
    let prec = binop_prec(binop);
    prec > in_prec || binop_is_r_assoc(binop) && prec == in_prec
}

fn binop_prec(binop: &BinOp) -> Prec {
    match binop {
        // BinOp::LogicalOr => Prec::LogicalOr,
        // BinOp::LogicalAnd => Prec::LogicalAnd,

        // BinOp::Equal | BinOp::NotEqual => Prec::Equality,
        // BinOp::Gt | BinOp::Lt | BinOp::GtEq | BinOp::LtEq => Prec::Comparison,
        BinOp::BitwiseAnd => Prec::BitwiseAnd,
        BinOp::BitwiseXor => Prec::BitwiseXor,
        BinOp::BitwiseOr => Prec::BitwiseOr,

        BinOp::Add | BinOp::Sub => Prec::Term,
        BinOp::Mul | BinOp::Div | BinOp::Mod => Prec::Factor,
        // BinOp::Field => Prec::Field,
        // BinOp::Call | BinOp::Index => Prec::Call,
    }
}

fn binop_is_r_assoc(_binop: &BinOp) -> bool {
    // nothing for now, but best to keep the framework in place
    false
}

impl Parser {
    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_prec(Prec::Lowest)
    }

    fn parse_prec(&mut self, prec: Prec) -> ParseResult<Expr> {
        let mut expr = self.parse_lhs()?;

        while let Some(op) = self.peek_bin_op(prec) {
            // `get_op` doesn't consume a token because
            // some (as of yet unimplemented) operations need to consume
            // the token themselves
            self.tokens.next();

            let rhs = self.parse_prec(binop_prec(&op))?;

            let span = expr.span.union(rhs.span);
            expr = Expr::new(
                ExprKind::BinOp {
                    op,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn parse_lhs(&mut self) -> ParseResult<Expr> {
        match self.tokens.peek() {
            Some(Token {
                kind: TokenKind::Integer(n),
                span,
            }) => {
                self.tokens.next();
                Ok(Expr::new(ExprKind::Constant(n), span))
            }

            Some(Token {
                kind: TokenKind::Identifier(ident),
                span,
            }) => {
                self.tokens.next();
                // TODO: rely on expression span instead of storing in ident??
                Ok(Expr::new(ExprKind::Var(Ident { ident, span }), span))
            }

            Some(t) if t.kind == TokenKind::Keyword(Keyword::Void) => {
                self.tokens.next();
                Ok(Expr::new(ExprKind::Void, t.span))
            }

            Some(t) if t.kind == TokenKind::Sub => {
                self.tokens.next();

                let expr = self.parse_prec(Prec::Unary)?;
                Ok(Expr::new(
                    ExprKind::UnOp {
                        op: UnOp::Negate,
                        expr: Box::new(expr),
                    },
                    t.span,
                ))
            }

            Some(t) if t.kind == TokenKind::BitwiseInvert => {
                self.tokens.next();

                let expr = self.parse_prec(Prec::Unary)?;
                Ok(Expr::new(
                    ExprKind::UnOp {
                        op: UnOp::BitwiseInvert,
                        expr: Box::new(expr),
                    },
                    t.span,
                ))
            }

            Some(t) if t.kind == TokenKind::LParen => {
                self.tokens.next();

                let expr = self.parse_or_recover(Self::parse_expr, |parser, span| {
                    parser.seek(TokenKind::RParen);
                    Expr::new(ExprKind::ParseError, span)
                });

                self.expect(TokenKind::RParen)?;

                Ok(expr)
            }

            other => Err(self.error_expected("an expression", other)),
        }
    }

    fn peek_bin_op(&self, prec: Prec) -> Option<BinOp> {
        let op = match self.tokens.peek().map(|t| t.kind)? {
            TokenKind::Add => BinOp::Add,
            TokenKind::Sub => BinOp::Sub,
            TokenKind::Mul => BinOp::Mul,
            TokenKind::Div => BinOp::Div,
            TokenKind::Mod => BinOp::Mod,

            TokenKind::BitwiseAnd => BinOp::BitwiseAnd,
            TokenKind::BitwiseOr => BinOp::BitwiseOr,
            TokenKind::BitwiseXor => BinOp::BitwiseXor,

            _ => return None,
        };

        should_parse_binop_in_prec(&op, prec).then_some(op)
    }
}
