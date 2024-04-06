use super::{Lexer, LexerError};
use crate::ir::token::Token;
use crate::ir::Interner;
use crate::utils::assert_yaml_snapshot_with_source;

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
    assert_lex!("\nfunc\nmain\n(\n)\n->\nint\n{\nreturn\n0\n;\n}");
}

#[test]
fn no_newlines() {
    assert_lex!("func main()->int{return 0;}");
}

#[test]
fn spaces() {
    assert_lex!("  func  main  (  )  ->  int  {  return  0  ;  }");
}
