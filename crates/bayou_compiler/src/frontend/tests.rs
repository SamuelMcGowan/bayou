use crate::compile;
use crate::diagnostic::DiagnosticOutput;
use crate::session::Session;

fn test_compiles(source: &str, should_compile: bool) {
    let session = Session::new(DiagnosticOutput::owned());
    let result = compile(source, &session);

    match (result, should_compile) {
        (Err(_), true) => panic!("failed to compile: {source:?}"),
        (Ok(_), false) => panic!("unexpectedly compiled: {source:?}"),
        _ => {}
    }
}

#[test]
fn multi_digit() {
    test_compiles("int main() { return 100; }", true);
}

#[test]
fn newlines() {
    test_compiles("\nint\nmain\n(\n)\n{\nreturn\n0\n;\n}", true);
}

#[test]
fn no_newlines() {
    test_compiles("int main(){return 0;}", true);
}

#[test]
fn spaces() {
    test_compiles("   int   main    (  )  {   return  0 ; }", true);
}

#[test]
fn return_0() {
    test_compiles("int main() { return 0; }", true);
}

#[test]
fn return_2() {
    test_compiles("int main() { return 2; }", true);
}

#[test]
fn missing_paren() {
    test_compiles("int main( { return 0; }", false);
}

#[test]
fn missing_retval() {
    test_compiles("int main() { return; }", false);
}

#[test]
fn no_brace() {
    test_compiles("int main() { return 0;", false);
}

#[test]
fn no_semicolon() {
    test_compiles("int main() { return 0 }", false);
}

#[test]
fn no_space() {
    test_compiles("int main() { return0; }", false);
}

#[test]
fn wrong_case() {
    test_compiles("int main() { RETURN 0; }", false);
}
