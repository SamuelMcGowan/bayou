#[cfg(test)]
mod tests;

mod expr;

use bayou_ir::{Ident, Type};
use bayou_session::diagnostics::prelude::*;
use bayou_session::diagnostics::span::Span;
use bayou_utils::peek::Peek;

use crate::ast::*;
use crate::lexer::TokenIter;
use crate::token::{Keyword, Token, TokenKind};

#[derive(serde::Serialize, Debug, Clone)]
pub struct ParseError {
    pub expected: String,
    pub span: Span,
}

impl IntoDiagnostic<SourceId> for ParseError {
    fn into_diagnostic(self, source_id: &SourceId) -> Diagnostic {
        Diagnostic::error()
            .with_message("syntax error")
            .with_snippet(Snippet::primary(
                format!("expected {} here", self.expected),
                *source_id,
                self.span,
            ))
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone)]
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
            match self.parse_item() {
                Ok(item) => {
                    items.push(item);
                }

                Err(err) => {
                    self.report(err);
                    self.seek(&[
                        TokenKind::Keyword(Keyword::Submodule),
                        TokenKind::Keyword(Keyword::Func),
                    ]);
                }
            }
        }

        Module { items }
    }

    fn parse_item(&mut self) -> ParseResult<Item> {
        match self.tokens.next() {
            Some(t) if t.kind == TokenKind::Keyword(Keyword::Submodule) => {
                let ident = self.parse_ident()?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Item::Submodule(ident))
            }

            Some(t) if t.kind == TokenKind::Keyword(Keyword::Func) => {
                let item = self.parse_or_recover(
                    |parser| parser.parse_func_decl().map(Item::FuncDecl),
                    |_, _| Item::ParseError,
                );
                Ok(item)
            }

            other => Err(self.error_expected("an item", other)),
        }
    }

    fn parse_func_decl(&mut self) -> ParseResult<FuncDecl> {
        let ident = self.parse_ident()?;

        self.expect_or_recover(TokenKind::LParen);
        self.expect_or_recover(TokenKind::RParen); // TODO: change when parsing parameters?

        let (ret_ty, ret_ty_span) = if self.eat_kind(TokenKind::Arrow) {
            self.parse_spanned(|parser| {
                parser.parse_or_recover(Self::parse_type, |parser, _| {
                    parser.seek(&[TokenKind::LBrace]);
                    Type::Void
                })
            })
        } else {
            (Type::Void, Span::empty(self.tokens.peek_span().start))
        };

        let block = self.parse_block()?;

        Ok(FuncDecl {
            ident,

            ret_ty,
            ret_ty_span,

            block,
        })
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        match self.tokens.peek() {
            Some(t) if t.kind == TokenKind::Keyword(Keyword::I64) => {
                self.tokens.next();
                Ok(Type::I64)
            }
            Some(t) if t.kind == TokenKind::Keyword(Keyword::Bool) => {
                self.tokens.next();
                Ok(Type::Bool)
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

    fn parse_block(&mut self) -> ParseResult<Block> {
        let ((statements, final_expr), span) = self
            .parse_spanned(|parser| -> ParseResult<_> {
                parser.expect(TokenKind::LBrace)?;

                let mut statements = vec![];
                let mut final_expr = None;

                // TODO: significantly improve recoverable parsing

                while parser
                    .tokens
                    .peek()
                    .is_some_and(|t| t.kind != TokenKind::RBrace)
                {
                    let statement = parser.parse_statement_or_recover();

                    match statement {
                        Stmt::Drop {
                            expr,
                            had_semicolon: false,
                        } if !expr.kind.stmt_semicolon_is_optional() => {
                            final_expr = Some(expr);
                            break;
                        }

                        _ => {
                            statements.push(statement);
                        }
                    }
                }

                let rbrace = parser.expect(TokenKind::RBrace)?;

                let final_expr =
                    final_expr.unwrap_or_else(|| Expr::new(ExprKind::Void, rbrace.span));

                Ok((statements, final_expr))
            })
            .transpose()?;

        Ok(Block {
            statements,
            final_expr,

            span,
        })
    }

    /// Always makes progress.
    fn parse_statement_or_recover(&mut self) -> Stmt {
        self.parse_or_recover(Self::parse_statement, |parser, _| {
            parser.seek_and_consume(&[TokenKind::Semicolon]);
            Stmt::ParseError
        })
    }

    /// Always makes progress.
    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Keyword(Keyword::Return) => {
                self.tokens.next();

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
                self.tokens.next();

                let ident = self.parse_ident()?;
                self.expect(TokenKind::Colon)?;
                let ty = self.parse_type()?;
                self.expect(TokenKind::Assign)?;
                let expr = self.parse_expr()?;
                self.expect_or_recover(TokenKind::Semicolon);

                Ok(Stmt::Assign { ident, ty, expr })
            }

            _ => {
                let expr = self.parse_expr()?;

                let had_semicolon = self.eat_kind(TokenKind::Semicolon);

                Ok(Stmt::Drop {
                    expr,
                    had_semicolon,
                })
            }
        }
    }

    fn parse_ident(&mut self) -> ParseResult<Ident> {
        match self.tokens.next() {
            Some(Token {
                kind: TokenKind::Identifier(istr),
                span,
            }) => Ok(Ident { istr, span }),

            other => Err(self.error_expected("an identifier", other)),
        }
    }

    fn parse_or_recover<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> ParseResult<T>,
        recover: impl FnOnce(&mut Self, Span) -> T,
    ) -> T {
        let (result, span) = self.parse_spanned(parse);

        match result {
            Ok(node) => node,
            Err(err) => {
                self.report(err);
                recover(self, span)
            }
        }
    }

    fn parse_spanned<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> (T, Span) {
        let span_start = self.tokens.peek_span();
        let node = f(self);
        let span_end = self.tokens.prev_span();

        let span = Span::new(span_start.start, span_end.end.max(span_start.start));
        (node, span)
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

    fn seek_and_consume(&mut self, kinds: &[TokenKind]) {
        if self.seek(kinds) {
            self.tokens.next();
        }
    }

    fn seek(&mut self, kinds: &[TokenKind]) -> bool {
        // could just use `paren_depth_stack` but this is clearer
        let mut brace_depth = 0;

        let mut paren_depth = 0;
        let mut paren_depth_stack = vec![];

        loop {
            match self.tokens.peek() {
                Some(token) if kinds.contains(&token.kind) => {
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
                    }

                    self.tokens.next();

                    brace_depth -= 1;
                    paren_depth = paren_depth_stack.pop().unwrap_or(0);
                }

                Some(token) if token.kind == TokenKind::LParen => {
                    self.tokens.next();
                    paren_depth += 1;
                }

                Some(token) if token.kind == TokenKind::RParen => {
                    if paren_depth == 0 {
                        return false;
                    }

                    self.tokens.next();
                    paren_depth -= 1;
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

pub trait Transpose {
    type Transposed;

    fn transpose(self) -> Self::Transposed;
}

impl<T, E> Transpose for (Result<T, E>, Span) {
    type Transposed = Result<(T, Span), E>;

    fn transpose(self) -> Self::Transposed {
        self.0.map(|inner| (inner, self.1))
    }
}
