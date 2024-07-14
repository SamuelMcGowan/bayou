#[cfg(test)]
mod tests;

use std::str::Chars;

use bayou_interner::Interner;
use bayou_session::diagnostics::prelude::*;
use bayou_utils::peek::Peek;

use crate::token::*;

#[derive(serde::Serialize)]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub span: Span,
}

#[derive(serde::Serialize, thiserror::Error, Debug)]
pub enum LexerErrorKind {
    #[error("unexpected character {0:?}")]
    UnexpectedChar(char),

    #[error("integer overflow")]
    IntegerOverflow,

    #[error("digit {digit:?} is invalid for base {base}")]
    IntegerDigitWrongBase { base: u32, digit: char },
}

impl IntoDiagnostic<SourceId> for LexerError {
    fn into_diagnostic(self, source_id: SourceId) -> Diagnostic {
        Diagnostic::error()
            .with_message("syntax error")
            .with_snippet(Snippet::primary(
                self.kind.to_string(),
                source_id,
                self.span,
            ))
    }
}

pub type LexerResult<T> = Result<T, LexerErrorKind>;

pub struct Lexer<'sess> {
    interner: &'sess Interner,
    errors: Vec<LexerError>,

    all: &'sess str,
    chars: Chars<'sess>,

    token_start: usize,
}

impl<'sess> Lexer<'sess> {
    pub fn new(source: &'sess str, interner: &'sess Interner) -> Self {
        Self {
            interner,
            errors: vec![],

            all: source,
            chars: source.chars(),

            token_start: 0,
        }
    }

    pub fn lex(mut self) -> (TokenIter, Vec<LexerError>) {
        let mut tokens = vec![];
        while let Some(token) = self.lex_token() {
            tokens.push(token);
        }

        let iter = TokenIter {
            tokens: tokens.into_iter(),
            prev_span: Span::empty(0),
            eof_span: Span::empty(self.chars.as_str().len()),
        };

        (iter, self.errors)
    }

    fn lex_token(&mut self) -> Option<Token> {
        loop {
            macro_rules! try_lex {
                ($e:expr) => {{
                    match $e {
                        Ok(token) => token,
                        Err(err) => {
                            self.report_error(err);
                            continue;
                        }
                    }
                }};
            }

            self.token_start = self.byte_pos();

            let kind = match self.chars.next()? {
                // comment
                '/' if self.chars.eat('/') => {
                    while !matches!(self.chars.next(), Some('\n') | None) {}
                    continue;
                }

                ch if ch.is_ascii_whitespace() => continue,

                '{' => TokenKind::LBrace,
                '}' => TokenKind::RBrace,
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,

                '.' => TokenKind::Dot,
                ':' => TokenKind::Colon,
                ',' => TokenKind::Comma,
                ';' => TokenKind::Semicolon,
                '!' => TokenKind::Bang, // will break `!=`
                '-' if self.chars.eat('>') => TokenKind::Arrow,

                '+' => TokenKind::Add,
                '-' => TokenKind::Sub,
                '*' => TokenKind::Mul,
                '/' => TokenKind::Div,
                '%' => TokenKind::Mod,

                '=' => TokenKind::Assign,

                '&' => TokenKind::BitwiseAnd,
                '|' => TokenKind::BitwiseOr,
                '^' => TokenKind::BitwiseXor,
                '~' => TokenKind::BitwiseInvert,

                '0' if self.chars.eat('x') => try_lex!(self.lex_integer(0, 16)),
                '0' if self.chars.eat('o') => try_lex!(self.lex_integer(0, 8)),
                '0' if self.chars.eat('b') => try_lex!(self.lex_integer(0, 2)),

                ch @ '0'..='9' => try_lex!(self.lex_integer(ch as i64 - 48, 10)),

                ch if is_ident_start(ch) => self.lex_alpha(),

                ch => {
                    self.report_error(LexerErrorKind::UnexpectedChar(ch));
                    continue;
                }
            };

            let token = Token {
                kind,
                span: Span::new(self.token_start, self.byte_pos()),
            };

            return Some(token);
        }
    }

    fn lex_integer(&mut self, start: i64, base: u32) -> LexerResult<TokenKind> {
        let mut n = Some(start);

        while let Some(ch @ ('0'..='9' | 'a'..='f' | 'A'..='F' | '_')) = self.chars.peek() {
            self.chars.next();

            if ch == '_' {
                continue;
            }

            let digit = ch
                .to_digit(base)
                .ok_or(LexerErrorKind::IntegerDigitWrongBase { base, digit: ch })?;

            n = n.and_then(|n| n.checked_mul(base as i64));
            n = n.and_then(|n| n.checked_add(digit as i64));
        }

        n.map(TokenKind::Integer)
            .ok_or(LexerErrorKind::IntegerOverflow)
    }

    fn lex_alpha(&mut self) -> TokenKind {
        while matches!(self.chars.peek(), Some(ch) if is_ident(ch)) {
            self.chars.next();
        }

        let s = &self.all[self.token_start..self.byte_pos()];

        match s {
            "submodule" => TokenKind::Keyword(Keyword::Submodule),
            "func" => TokenKind::Keyword(Keyword::Func),
            "return" => TokenKind::Keyword(Keyword::Return),
            "let" => TokenKind::Keyword(Keyword::Let),
            "if" => TokenKind::Keyword(Keyword::If),
            "then" => TokenKind::Keyword(Keyword::Then),
            "else" => TokenKind::Keyword(Keyword::Else),
            "i64" => TokenKind::Keyword(Keyword::I64),
            "bool" => TokenKind::Keyword(Keyword::Bool),
            "void" => TokenKind::Keyword(Keyword::Void),
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ => {
                let interned = self.interner.intern(s);
                TokenKind::Identifier(interned)
            }
        }
    }

    fn byte_pos(&self) -> usize {
        self.all.len() - self.chars.as_str().len()
    }

    fn report_error(&mut self, kind: LexerErrorKind) {
        let span = Span::new(self.token_start, self.byte_pos());
        self.errors.push(LexerError { kind, span });
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

pub struct TokenIter {
    tokens: std::vec::IntoIter<Token>,
    prev_span: Span,
    eof_span: Span,
}

impl TokenIter {
    pub fn prev_span(&self) -> Span {
        self.prev_span
    }

    pub fn peek_span(&self) -> Span {
        self.peek().map(|t| t.span).unwrap_or(self.eof_span)
    }

    pub fn eof_span(&self) -> Span {
        self.eof_span
    }
}

impl Iterator for TokenIter {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.tokens.next()?;
        self.prev_span = token.span;
        Some(token)
    }
}

impl Peek for TokenIter {
    fn peek(&self) -> Option<Self::Item> {
        self.tokens.as_slice().first().copied()
    }
}
