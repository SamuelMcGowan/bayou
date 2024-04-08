#[cfg(test)]
mod tests;

mod expr;

use bayou_common::peek::Peek;
use bayou_diagnostic::span::Span;
use bayou_ir::{Ident, Spanned, Type};

use crate::ast::*;
use crate::lexer::TokenIter;
use crate::token::{Keyword, Token, TokenKind};

#[derive(serde::Serialize)]
pub struct ParseError {
    pub expected: String,
    pub span: Span,
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    tokens: TokenIter,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: TokenIter) -> Self {
        Self {
            tokens,
            errors: vec![],
        }
    }

    pub fn parse(mut self) -> (Module, Vec<ParseError>) {
        let module = self.parse_module();
        (module, self.errors)
    }

    fn parse_module(&mut self) -> Module {
        let mut items = vec![];

        while !self.tokens.at_end() {
            match self.tokens.next() {
                Some(t) if t.kind == TokenKind::Keyword(Keyword::Func) => {
                    let item = self.parse_or_recover(
                        |parser| parser.parse_func_decl().map(Item::FuncDecl),
                        |_, _| Item::ParseError,
                    );
                    items.push(item);
                }

                other => {
                    self.report(self.error_expected("an item", other));
                    self.seek(TokenKind::Keyword(Keyword::Func));
                }
            }
        }

        Module { items }
    }

    fn parse_func_decl(&mut self) -> ParseResult<FuncDecl> {
        let name = self.parse_ident()?;

        self.expect_or_recover(TokenKind::LParen);
        self.expect_or_recover(TokenKind::RParen); // TODO: change when parsing parameters?

        let ret_ty = if self.eat_kind(TokenKind::Arrow) {
            self.parse_or_recover(Self::parse_type, |parser, _| {
                parser.seek(TokenKind::LBrace);
                Type::Void
            })
        } else {
            Type::Void
        };

        self.expect(TokenKind::LBrace)?;

        let mut statements = vec![];
        while self
            .tokens
            .peek()
            .is_some_and(|t| t.kind != TokenKind::RBrace)
        {
            let statement = self.parse_statement_or_recover();
            statements.push(statement);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(FuncDecl {
            name,
            ret_ty,
            statements,
        })
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        match self.tokens.peek() {
            Some(t) if t.kind == TokenKind::Keyword(Keyword::I64) => {
                self.tokens.next();
                Ok(Type::I64)
            }
            Some(t) if t.kind == TokenKind::Keyword(Keyword::Void) => {
                self.tokens.next();
                Ok(Type::Void)
            }
            Some(t) if t.kind == TokenKind::Bang => {
                self.tokens.next();
                Ok(Type::Never)
            }

            other => Err(self.error_expected("a type", other)),
        }
    }

    fn parse_statement_or_recover(&mut self) -> Stmt {
        self.parse_or_recover(Self::parse_statement, |parser, _| {
            parser.seek_and_consume(TokenKind::Semicolon);
            Stmt::ParseError
        })
    }

    // should always advance at least one token (unless at end)
    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        match self.tokens.next() {
            Some(token) if token.kind == TokenKind::Keyword(Keyword::Return) => {
                let expr = if self.eat_kind(TokenKind::Semicolon) {
                    Expr::new(
                        ExprKind::Void,
                        // `return` span
                        token.span,
                    )
                } else {
                    match self.tokens.peek() {
                        // handle case of `return }`
                        Some(brace_token) if brace_token.kind == TokenKind::RBrace => {
                            self.report(
                                self.error_expected_kind(TokenKind::Semicolon, Some(brace_token)),
                            );

                            Expr::new(
                                ExprKind::Void,
                                // `return` span
                                token.span,
                            )
                        }

                        _ => {
                            let expr = self.parse_expr()?;

                            self.expect_or_recover(TokenKind::Semicolon);
                            expr
                        }
                    }
                };

                Ok(Stmt::Return(expr))
            }

            Some(token) if token.kind == TokenKind::Keyword(Keyword::Let) => {
                let ident = self.parse_ident()?;
                self.expect(TokenKind::Assign)?;
                let expr = self.parse_expr()?;
                self.expect_or_recover(TokenKind::Semicolon);
                Ok(Stmt::Assign { ident, expr })
            }

            other => Err(self.error_expected("a statement", other)),
        }
    }

    fn parse_ident(&mut self) -> ParseResult<Ident> {
        match self.tokens.next() {
            Some(Token {
                kind: TokenKind::Identifier(ident),
                span,
            }) => Ok(Ident { ident, span }),
            other => Err(self.error_expected("an identifier", other)),
        }
    }

    fn parse_or_recover<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> ParseResult<T>,
        recover: impl FnOnce(&mut Self, Span) -> T,
    ) -> T {
        let result = self.parse_spanned(parse);
        match result.node {
            Ok(node) => node,
            Err(err) => {
                self.report(err);
                recover(self, result.span)
            }
        }
    }

    fn parse_spanned<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> Spanned<T> {
        let span_start = self.tokens.peek_span();
        let node = f(self);
        let span_end = self.tokens.prev_span();

        let span = Span::new(span_start.start, span_end.end.max(span_start.start));
        Spanned::new(node, span)
    }

    fn expect(&mut self, kind: TokenKind) -> ParseResult<Token> {
        match self.tokens.peek() {
            Some(t) if t.kind == kind => {
                self.tokens.next();
                Ok(t)
            }

            other => Err(self.error_expected_kind(kind, other)),
        }
    }

    fn expect_or_recover(&mut self, kind: TokenKind) {
        if let Err(error) = self.expect(kind) {
            self.report(error);
        }
    }

    fn eat_kind(&mut self, kind: TokenKind) -> bool {
        match self.tokens.peek() {
            Some(t) if t.kind == kind => {
                self.tokens.next();
                true
            }
            _ => false,
        }
    }

    fn seek_and_consume(&mut self, kind: TokenKind) {
        if self.seek(kind) {
            self.tokens.next();
        }
    }

    fn seek(&mut self, kind: TokenKind) -> bool {
        // could just use `paren_depth_stack` but this is clearer
        let mut brace_depth = 0;

        let mut paren_depth = 0;
        let mut paren_depth_stack = vec![];

        loop {
            match self.tokens.peek() {
                Some(token) if token.kind == kind => {
                    return true;
                }

                Some(token) if token.kind == TokenKind::LBrace => {
                    self.tokens.next();

                    brace_depth += 1;

                    paren_depth_stack.push(paren_depth);
                    paren_depth = 0;
                }

                Some(token) if token.kind == TokenKind::RBrace => {
                    if brace_depth == 0 {
                        return false;
                    } else {
                        self.tokens.next();

                        brace_depth -= 1;

                        paren_depth = paren_depth_stack.pop().unwrap_or(0);
                    }
                }

                Some(token) if token.kind == TokenKind::LParen => {
                    self.tokens.next();
                    paren_depth += 1;
                }

                Some(token) if token.kind == TokenKind::RParen => {
                    if paren_depth == 0 {
                        return false;
                    } else {
                        self.tokens.next();
                        paren_depth -= 1;
                    }
                }

                Some(_) => {
                    self.tokens.next();
                }

                None => {
                    return false;
                }
            }
        }
    }

    fn report(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    fn error_expected_kind(&self, kind: TokenKind, found: Option<Token>) -> ParseError {
        self.error_expected(kind.token_name(), found)
    }

    fn error_expected(&self, expected: impl Into<String>, found: Option<Token>) -> ParseError {
        match found {
            Some(token) => ParseError {
                expected: expected.into(),
                span: token.span,
            },
            None => ParseError {
                expected: expected.into(),
                span: self.tokens.eof_span(),
            },
        }
    }
}
