#[cfg(test)]
mod tests;

mod expr;
mod lexer;

use bayou_diagnostic::span::Span;

use self::lexer::{Lexer, LexerError, Peek};
use crate::ir::ast::*;
use crate::ir::token::{Keyword, Token, TokenKind};
use crate::ir::Interner;

pub enum ParseError {
    Expected { expected: String, span: Span },
    Lexer(LexerError),
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'sess> {
    errors: Vec<ParseError>,
    lexer: Lexer<'sess>,
}

impl<'sess> Parser<'sess> {
    pub fn new(source: &'sess str) -> Self {
        Self {
            errors: vec![],
            lexer: Lexer::new(source),
        }
    }

    pub fn parse(mut self) -> (Module, Interner, Vec<ParseError>) {
        let module = self.parse_module();

        let (interner, lexer_errors) = self.lexer.finish();

        let mut errors = vec![];
        errors.extend(lexer_errors.into_iter().map(ParseError::Lexer));
        errors.extend(self.errors);

        (module, interner, errors)
    }

    fn parse_module(&mut self) -> Module {
        let item = self.parse_or_recover(
            |parser| parser.parse_func_decl().map(Item::FuncDecl),
            |_| Item::ParseError,
        );
        Module { item }
    }

    fn parse_func_decl(&mut self) -> ParseResult<FuncDecl> {
        self.expect(TokenKind::Keyword(Keyword::Int))?;

        let name = self.parse_ident()?;

        self.expect(TokenKind::LParen)?;
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::LBrace)?;

        let statement = self.parse_statement_or_recover();

        self.expect(TokenKind::RBrace)?;

        Ok(FuncDecl { name, statement })
    }

    fn parse_statement_or_recover(&mut self) -> Stmt {
        self.parse_or_recover(Self::parse_statement, |parser| {
            parser.seek_and_consume(TokenKind::Semicolon);
            Stmt::ParseError
        })
    }

    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        self.expect(TokenKind::Keyword(Keyword::Return))?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Return(expr))
    }

    fn parse_ident(&mut self) -> ParseResult<Ident> {
        match self.lexer.next() {
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
        recover: impl FnOnce(&mut Self) -> T,
    ) -> T {
        parse(self).unwrap_or_else(|err| {
            self.report(err);
            recover(self)
        })
    }

    fn expect(&mut self, kind: TokenKind) -> ParseResult<Token> {
        match self.lexer.peek() {
            Some(t) if t.kind == kind => {
                self.lexer.next();
                Ok(t)
            }

            other => Err(self.error_expected_kind(kind, other)),
        }
    }

    fn seek_and_consume(&mut self, kind: TokenKind) {
        if self.seek(kind) {
            self.lexer.next();
        }
    }

    fn seek(&mut self, kind: TokenKind) -> bool {
        // could just use `paren_depth_stack` but this is clearer
        let mut brace_depth = 0;

        let mut paren_depth = 0;
        let mut paren_depth_stack = vec![];

        loop {
            match self.lexer.peek() {
                Some(token) if token.kind == kind => {
                    return true;
                }

                Some(token) if token.kind == TokenKind::LBrace => {
                    self.lexer.next();

                    brace_depth += 1;

                    paren_depth_stack.push(paren_depth);
                    paren_depth = 0;
                }

                Some(token) if token.kind == TokenKind::RBrace => {
                    if brace_depth == 0 {
                        return false;
                    } else {
                        self.lexer.next();

                        brace_depth -= 1;

                        paren_depth = paren_depth_stack.pop().unwrap_or(0);
                    }
                }

                Some(token) if token.kind == TokenKind::LParen => {
                    self.lexer.next();
                    paren_depth += 1;
                }

                Some(token) if token.kind == TokenKind::RParen => {
                    if paren_depth == 0 {
                        return false;
                    } else {
                        self.lexer.next();
                        paren_depth -= 1;
                    }
                }

                Some(_) => {
                    self.lexer.next();
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
            Some(token) => ParseError::Expected {
                expected: expected.into(),
                span: token.span,
            },
            None => ParseError::Expected {
                expected: expected.into(),
                span: self.lexer.eof_span(),
            },
        }
    }
}
