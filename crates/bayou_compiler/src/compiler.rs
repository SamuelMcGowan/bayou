use bayou_diagnostic::sources::{Cached, Source};

use crate::diagnostics::Diagnostics;
use crate::frontend::parser::Parser;
use crate::frontend::resolver::Resolver;

#[derive(Default)]
pub struct Compiler {
    pub sources: Vec<Cached<(String, String)>>,
}

impl Compiler {
    pub fn parse_module(
        &mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> Diagnostics {
        let mut diagnostics = Diagnostics::default();

        let source_id = self.sources.len();
        self.sources.push(Cached::new((name.into(), source.into())));
        let source = self.sources.last().unwrap();

        let mut parser = Parser::new(source.source_str(), source_id);
        let mut ast = parser.parse_module();

        let (interner, parser_diagnostics) = parser.finish();

        diagnostics.join(parser_diagnostics);
        if diagnostics.had_errors() {
            return diagnostics;
        }

        let resolver = Resolver::new(&interner);
        let (symbols, resolver_diagnostics) = resolver.run(&mut ast);

        diagnostics.join(resolver_diagnostics);
        if diagnostics.had_errors() {
            return diagnostics;
        }

        diagnostics
    }
}
