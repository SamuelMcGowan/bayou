use std::str::Chars;

use bayou_diagnostic::span::Span;

use crate::ir::token::{Keyword, Token, TokenKind};
use crate::ir::Interner;

pub struct LexerError {
    pub kind: LexerErrorKind,
    pub span: Span,
}

#[derive(thiserror::Error, Debug)]
pub enum LexerErrorKind {
    #[error("unexpected character {0:?}")]
    UnexpectedChar(char),

    #[error("integer overflow")]
    IntegerOverflow,

    #[error("digit {digit:?} is invalid for base {base}")]
    IntegerDigitWrongBase { base: u32, digit: char },
}

pub type LexerResult<T> = Result<T, LexerErrorKind>;

pub struct Lexer<'sess> {
    interner: &'sess mut Interner,
    errors: Vec<LexerError>,

    all: &'sess str,
    chars: Chars<'sess>,

    token_start: usize,

    prev_span: Span,
    current: Option<Token>,
}

impl<'sess> Lexer<'sess> {
    pub fn new(source: &'sess str, interner: &'sess mut Interner) -> Self {
        let mut lexer = Self {
            interner,
            errors: vec![],

            all: source,
            chars: source.chars(),

            token_start: 0,

            prev_span: Span::new(0, 0),
            current: None,
        };
        lexer.current = lexer.lex_token();
        lexer
    }

    pub fn peek_span(&self) -> Span {
        self.peek()
            .map(|t| t.span)
            .unwrap_or_else(|| self.eof_span())
    }

    pub fn prev_span(&self) -> Span {
        self.prev_span
    }

    pub fn eof_span(&self) -> Span {
        Span::new(self.all.len(), self.all.len())
    }

    pub fn finish(self) -> Vec<LexerError> {
        self.errors
    }

    pub fn lex_token(&mut self) -> Option<Token> {
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
            "func" => TokenKind::Keyword(Keyword::Func),
            "return" => TokenKind::Keyword(Keyword::Return),
            "let" => TokenKind::Keyword(Keyword::Let),
            "int" => TokenKind::Keyword(Keyword::Int),
            "void" => TokenKind::Keyword(Keyword::Void),
            _ => {
                let interned = self.interner.get_or_intern(s);
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

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.current.take()?;
        self.prev_span = token.span;
        self.current = self.lex_token();
        Some(token)
    }
}

impl Peek for Lexer<'_> {
    fn peek(&self) -> Option<Self::Item> {
        self.current
    }
}

pub trait Peek: Iterator {
    fn peek(&self) -> Option<Self::Item>;

    fn eat<P>(&mut self, pat: P) -> bool
    where
        Self::Item: PartialEq<P>,
    {
        match self.peek() {
            Some(item) if item == pat => {
                self.next();
                true
            }
            _ => false,
        }
    }

    fn at_end(&self) -> bool {
        self.peek().is_none()
    }
}

impl Peek for Chars<'_> {
    fn peek(&self) -> Option<Self::Item> {
        self.clone().next()
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}
