use bayou_diagnostic::sources::{Cached, Source};

use crate::diagnostic::Diagnostics;
use crate::frontend::parser::Parser;
use crate::frontend::resolver::Resolver;
use crate::{CompilerError, CompilerResult};

#[derive(Default)]
pub struct Compiler {
    pub sources: Vec<Cached<(String, String)>>,
}

impl Compiler {
    pub fn compile(
        &mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> (CompilerResult<String>, Diagnostics) {
        self.sources.push(Cached::new((name.into(), source.into())));
        let source = self.sources.last().unwrap();

        let mut parser = Parser::new(source.source_str());
        let mut ast = parser.parse_module();

        let (interner, parser_diagnostics) = parser.finish();
        if parser_diagnostics.had_errors() {
            return (Err(CompilerError::HadErrors), parser_diagnostics);
        }

        let resolver = Resolver::new(&interner);
        let resolver_diagnostics = resolver.run(&mut ast);
        if resolver_diagnostics.had_errors() {
            return (Err(CompilerError::HadErrors), resolver_diagnostics);
        }

        (Ok("".into()), Diagnostics::default())
    }
}
