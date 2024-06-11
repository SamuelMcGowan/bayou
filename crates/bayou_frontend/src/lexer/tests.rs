use bayou_interner::Interner;
use bayou_utils::assert_yaml_snapshot_with_source;

use super::{Lexer, LexerError};
use crate::token::Token;

fn lex(source: &str) -> (Vec<Token>, Vec<LexerError>) {
    let mut interner = Interner::new();

    let lexer = Lexer::new(source, &mut interner);
    let (tokens, lexer_errors) = lexer.lex();

    (tokens.collect(), lexer_errors)
}

macro_rules! assert_lex {
    ($source:expr) => {{
        let source = $source;
        assert_yaml_snapshot_with_source!(source => lex(source));
    }};
}

#[test]
fn integer() {
    assert_lex!("100");
}

#[test]
fn integer_overflow() {
    assert_lex!("100000000000000000000");
}

#[test]
fn newlines() {
    assert_lex!("\nfunc\nmain\n(\n)\n->\ni64\n{\nreturn\n0\n;\n}");
}

#[test]
fn no_newlines() {
    assert_lex!("func main()->i64{return 0;}");
}

#[test]
fn spaces() {
    assert_lex!("  func  main  (  )  ->  i64  {  return  0  ;  }");
}
