use super::{ParseError, Parser};
use crate::ir::ast::Module;
use crate::ir::Interner;
use crate::lexer::Lexer;
use crate::utils::assert_yaml_snapshot_with_source;

fn parse(source: &str) -> (Module, Vec<ParseError>) {
    let mut interner = Interner::new();

    let lexer = Lexer::new(source, &mut interner);
    let (tokens, lexer_errors) = lexer.lex();

    assert!(lexer_errors.is_empty(), "lexer errors in parser tests");

    let parser = Parser::new(tokens);
    parser.parse()
}

macro_rules! assert_parse {
    ($source:expr) => {{
        let source = $source;
        assert_yaml_snapshot_with_source!(source => parse(source));
    }};
}

#[test]
fn return_integer() {
    assert_parse!("func main() -> i64 { return 0; }");
}

#[test]
fn missing_paren() {
    assert_parse!("func main( { return 0; }");
}

#[test]
fn no_return_value() {
    assert_parse!("func main() { return; }");
}

#[test]
fn no_brace() {
    assert_parse!("func main() -> i64 { return 0;");
}

#[test]
fn no_semicolon() {
    assert_parse!("func main() -> i64 { return 0 }");
}

#[test]
fn no_semicolon_or_return_value() {
    assert_parse!("func main() -> i64 { return }");
}

#[test]
fn no_space() {
    assert_parse!("func main() -> i64 { return0; }");
}

#[test]
fn wrong_case() {
    assert_parse!("func main() -> i64 { RETURN 0; }");
}
