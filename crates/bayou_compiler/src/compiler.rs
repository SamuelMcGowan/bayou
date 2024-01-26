use bayou_diagnostic::sources::{Cached, Source};

use crate::diagnostic::Diagnostics;
use crate::frontend::parser::Parser;
use crate::frontend::resolver::Resolver;

#[derive(Default)]
pub struct Compiler {
    pub sources: Vec<Cached<(String, String)>>,
}

impl Compiler {
    pub fn compile(
        &mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> (Option<String>, Diagnostics) {
        let mut diagnostics = Diagnostics::default();

        self.sources.push(Cached::new((name.into(), source.into())));
        let source = self.sources.last().unwrap();

        let mut parser = Parser::new(source.source_str());
        let mut ast = parser.parse_module();

        let (interner, parser_diagnostics) = parser.finish();
        diagnostics.join(parser_diagnostics);

        if diagnostics.had_errors() {
            return (None, diagnostics);
        }

        let resolver = Resolver::new(&interner);
        diagnostics.join(resolver.run(&mut ast));

        if diagnostics.had_errors() {
            return (None, diagnostics);
        }

        (Some("".into()), diagnostics)
    }
}
