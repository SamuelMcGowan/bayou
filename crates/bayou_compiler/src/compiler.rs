use bayou_diagnostic::sources::{Cached, Source};

use crate::diagnostics::Diagnostics;
use crate::frontend::parser::Parser;
use crate::frontend::resolver::Resolver;
use crate::ir::Interner;
use crate::symbols::Symbols;

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
        let source_id = self.sources.len();
        self.sources.push(Cached::new((name.into(), source.into())));
        let source = self.sources.last().unwrap();

        let parser = Parser::new(source.source_str(), source_id);
        let (mut ast, mut context) = parser.parse();

        if context.diagnostics.had_errors() {
            return context.diagnostics;
        }

        let resolver = Resolver::new(&mut context);
        resolver.run(&mut ast);

        if context.diagnostics.had_errors() {
            return context.diagnostics;
        }

        context.diagnostics
    }
}

pub struct ModuleContext {
    pub source_id: usize,

    pub symbols: Symbols,
    pub interner: Interner,

    pub diagnostics: Diagnostics,
}
