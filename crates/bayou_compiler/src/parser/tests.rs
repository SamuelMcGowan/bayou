use target_lexicon::Triple;

use crate::compiler::{PackageCompilation, Session};

fn test_compiles(source: &str, should_compile: bool) {
    let mut session = Session::new(vec![]);

    let compiled = PackageCompilation::parse(&mut session, "tests", source)
        .and_then(|pkg| pkg.compile(&mut session, &Triple::host()))
        .is_ok();

    match (compiled, should_compile) {
        (false, true) => panic!(
            "failed to compile: {source:?}, diagnostics: {:?}",
            session.diagnostics
        ),
        (true, false) => panic!("unexpectedly compiled: {source:?}"),
        _ => {}
    }
}

#[test]
fn multi_digit() {
    test_compiles("func main() -> int { return 100; }", true);
}

#[test]
fn newlines() {
    test_compiles("\nfunc\nmain\n(\n)\n->\nint\n{\nreturn\n0\n;\n}", true);
}

#[test]
fn no_newlines() {
    test_compiles("func main()->int{return 0;}", true);
}

#[test]
fn spaces() {
    test_compiles("  func  main  (  )  ->  int  {  return  0  ;  }", true);
}

#[test]
fn return_0() {
    test_compiles("func main() -> int { return 0; }", true);
}

#[test]
fn return_2() {
    test_compiles("func main() -> int { return 2; }", true);
}

#[test]
fn missing_paren() {
    test_compiles("func main( { return 0; }", false);
}

#[test]
fn missing_retval() {
    test_compiles("func main() { return; }", false);
}

#[test]
fn no_brace() {
    test_compiles("func main() -> int { return 0;", false);
}

#[test]
fn no_semicolon() {
    test_compiles("func main() -> int { return 0 }", false);
}

#[test]
fn no_space() {
    test_compiles("func main() -> int { return0; }", false);
}

#[test]
fn wrong_case() {
    test_compiles("func main() -> int { RETURN 0; }", false);
}
