use std::io;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use super::sources::{Source, Sources};
use super::{Diagnostic, DiagnosticKind, Snippet};

impl<S: Sources> Diagnostic<S> {
    pub fn write_to_stream(
        &self,
        sources: &S,
        config: &Config,
        stream: &mut impl WriteColor,
    ) -> io::Result<()> {
        let (kind_str, kind_color) = match self.kind {
            DiagnosticKind::Warning => ("Warning", &config.warning_color),
            DiagnosticKind::Error => ("Error", &config.error_color),
        };

        // top line
        {
            stream.set_color(kind_color)?;

            if let Some(id) = &self.id {
                write!(stream, "[{id}] ")?;
            }

            write!(stream, "{kind_str}:")?;
            stream.reset()?;

            if let Some(msg) = &self.message {
                write!(stream, " {msg}")?;
            }

            writeln!(stream)?;
        }

        // snippets
        for snippet in &self.snippets {
            let file = sources
                .get_source(snippet.source_id)
                .expect("source not in sources");

            stream.set_color(&config.emphasis)?;
            write!(stream, "  in {}:", file.name())?;
            stream.reset()?;
            writeln!(stream, " {}\n", snippet.label)?;
        }

        Ok(())
    }
}

pub struct Config {
    pub error_color: ColorSpec,
    pub warning_color: ColorSpec,

    pub emphasis: ColorSpec,
}

impl Default for Config {
    fn default() -> Self {
        let mut error_color = ColorSpec::new();
        error_color.set_fg(Some(Color::Red));
        error_color.set_bold(true);

        let mut warning_color = ColorSpec::new();
        warning_color.set_fg(Some(Color::Yellow));
        warning_color.set_bold(true);

        let mut emphasis = ColorSpec::new();
        emphasis.set_italic(true);

        Self {
            error_color,
            warning_color,
            emphasis,
        }
    }
}

#[test]
fn foo() {
    let diagnostic = Diagnostic::error()
        .with_message("oops")
        .with_id("E01")
        .with_snippet(Snippet::new("this is a label", 0, 0..0));

    let sources = vec![("my_file", "some contents")];

    let config = Config::default();
    let mut stream = StandardStream::stderr(ColorChoice::Auto);

    diagnostic
        .write_to_stream(&sources, &config, &mut stream)
        .unwrap();
}
