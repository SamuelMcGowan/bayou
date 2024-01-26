mod expr;

use bayou_diagnostic::Diagnostic;

use super::lexer::{Lexer, Peek};
use crate::diagnostic::Diagnostics;
use crate::ir::ast::*;
use crate::ir::token::{Keyword, Token, TokenKind};
use crate::ir::{InternedStr, Interner};

pub struct ParseError {
    expected: String,
    found: Option<Token>,
}

impl ParseError {
    fn expected(expected: impl Into<String>, found: Option<Token>) -> Self {
        Self {
            expected: expected.into(),
            found,
        }
    }

    fn expected_kind(token: TokenKind, found: Option<Token>) -> Self {
        Self::expected(format!("token {token:?}"), found)
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'sess> {
    diagnostics: Diagnostics,

    lexer: Lexer<'sess>,
}

impl<'sess> Parser<'sess> {
    pub fn new(source: &'sess str) -> Self {
        Self {
            diagnostics: Diagnostics::default(),

            lexer: Lexer::new(source),
        }
    }

    pub fn finish(self) -> (Interner, Diagnostics) {
        let (interner, mut diagnostics) = self.lexer.finish();
        diagnostics.join(self.diagnostics);
        (interner, diagnostics)
    }

    pub fn parse_module(&mut self) -> Module {
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
            parser.recover_past(TokenKind::Semicolon);
            Stmt::ParseError
        })
    }

    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        self.expect(TokenKind::Keyword(Keyword::Return))?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Return(expr))
    }

    fn parse_ident(&mut self) -> ParseResult<InternedStr> {
        match self.lexer.next() {
            Some(Token {
                kind: TokenKind::Identifier(ident),
                ..
            }) => Ok(ident),
            other => Err(ParseError::expected("an integer", other)),
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
        match self.lexer.next() {
            Some(t) if t.kind == kind => Ok(t),
            other => Err(ParseError::expected_kind(kind, other)),
        }
    }

    fn recover_until(&mut self, kind: TokenKind) {
        loop {
            let Some(token) = self.lexer.peek() else {
                return;
            };

            if token.kind == kind {
                return;
            }

            self.lexer.next();
        }
    }

    fn recover_past(&mut self, kind: TokenKind) {
        loop {
            let Some(token) = self.lexer.next() else {
                return;
            };

            if token.kind == kind {
                return;
            }
        }
    }

    fn report(&mut self, error: ParseError) {
        self.diagnostics
            .report(Diagnostic::error().with_message(format!(
                "expected {}, but found {:?}",
                error.expected, error.found
            )));
    }
}
