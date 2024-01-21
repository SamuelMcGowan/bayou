use std::str::Chars;

use crate::ast::{Keyword, Token};
use crate::session::{Diagnostic, IntoDiagnostic, Session};

#[derive(thiserror::Error, Debug)]
pub enum LexerError {
    #[error("unexpected character {0:?}")]
    UnexpectedChar(char),

    #[error("integer overflow")]
    IntegerOverflow,

    #[error("digit {digit:?} is invalid for base {base}")]
    IntegerDigitWrongBase { base: u32, digit: char },
}

impl IntoDiagnostic for LexerError {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic {
            message: self.to_string(),
            context: "while parsing".to_string(),
        }
    }
}

pub type LexerResult<T> = Result<T, LexerError>;

pub struct Lexer<'a> {
    all: &'a str,
    chars: Chars<'a>,

    token_start: usize,

    session: Session,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            all: source,
            chars: source.chars(),

            token_start: 0,

            session: Session::default(),
        }
    }

    pub fn into_session(self) -> Session {
        self.session
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

            self.token_start = self.position();

            let token = match self.chars.next()? {
                // comment
                '/' if self.chars.eat('/') => {
                    while !matches!(self.chars.next(), Some('\n') | None) {}
                    continue;
                }

                ch if ch.is_ascii_whitespace() => continue,

                '{' => Token::OpenBrace,
                '}' => Token::CloseBrace,
                '(' => Token::OpenParen,
                ')' => Token::CloseParen,
                ';' => Token::Semicolon,

                '0' if self.chars.eat('x') => try_lex!(self.lex_integer(0, 16)),
                '0' if self.chars.eat('o') => try_lex!(self.lex_integer(0, 8)),
                '0' if self.chars.eat('b') => try_lex!(self.lex_integer(0, 2)),

                ch @ '0'..='9' => try_lex!(self.lex_integer(ch as u64 - 48, 10)),

                ch if is_ident_start(ch) => self.lex_alpha(),

                ch => {
                    self.report_error(LexerError::UnexpectedChar(ch));
                    continue;
                }
            };

            return Some(token);
        }
    }

    fn lex_integer(&mut self, start: u64, base: u32) -> LexerResult<Token> {
        let mut n = start;

        while let Some(ch @ ('0'..='9' | '_')) = self.chars.peek() {
            self.chars.next();

            if ch == '_' {
                continue;
            }

            let digit = ch
                .to_digit(base)
                .ok_or(LexerError::IntegerDigitWrongBase { base, digit: ch })?;

            n = n
                .checked_mul(base as u64)
                .ok_or(LexerError::IntegerOverflow)?;

            n = n
                .checked_add(digit as u64)
                .ok_or(LexerError::IntegerOverflow)?;
        }

        Ok(Token::Integer(n))
    }

    fn lex_alpha(&mut self) -> Token {
        while matches!(self.chars.peek(), Some(ch) if is_ident(ch)) {
            self.chars.next();
        }

        let s = &self.all[self.token_start..self.position()];

        match s {
            "int" => Token::Keyword(Keyword::Int),
            "return" => Token::Keyword(Keyword::Return),
            _ => Token::Identifier(self.session.intern(s)),
        }
    }

    fn position(&self) -> usize {
        self.all.len() - self.chars.as_str().len()
    }

    fn report_error(&mut self, error: LexerError) {
        self.session.report(error);
    }
}

trait Peek: Iterator {
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
