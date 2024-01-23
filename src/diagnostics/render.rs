use std::io;

use termcolor::{Color, ColorSpec, WriteColor};

use super::sources::{Source, Sources};
use super::{Diagnostic, DiagnosticKind};

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
            stream.set_color(&config.emphasis)?;

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

            let (line, col) = file
                .byte_to_line_col(snippet.span.start)
                .expect("line out of range");

            stream.set_color(&config.subtle)?;
            write!(stream, "  in {}:{line}:{col} - ", file.name_str())?;
            stream.reset()?;
            writeln!(stream, "{}\n", snippet.label)?;
        }

        Ok(())
    }
}

pub struct Config {
    pub error_color: ColorSpec,
    pub warning_color: ColorSpec,

    pub emphasis: ColorSpec,
    pub subtle: ColorSpec,
}

impl Default for Config {
    fn default() -> Self {
        let mut error_color = ColorSpec::new();
        error_color.set_fg(Some(Color::Red));
        error_color.set_bold(true);

        let mut warning_color = ColorSpec::new();
        warning_color.set_fg(Some(Color::Yellow));
        warning_color.set_bold(true);

        let mut subtle = ColorSpec::new();
        subtle.set_italic(true);

        let mut emphasis = ColorSpec::new();
        emphasis.set_bold(true);

        Self {
            error_color,
            warning_color,
            emphasis,
            subtle,
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_yaml_snapshot;
    use termcolor::NoColor;

    use super::Config;
    use crate::diagnostics::sources::{Cached, Sources};
    use crate::diagnostics::{Diagnostic, Snippet};

    #[must_use]
    fn diagnostic_to_string<S: Sources>(diagnostic: Diagnostic<S>, sources: S) -> String {
        let config = Config::default();
        let mut stream = NoColor::new(vec![]);

        diagnostic
            .write_to_stream(&sources, &config, &mut stream)
            .unwrap();

        let bytes = stream.into_inner();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    #[test]
    fn simple_diagnostic() {
        let s = diagnostic_to_string(
            Diagnostic::error()
                .with_message("oops")
                .with_id("E01")
                .with_snippet(Snippet::new("this is a label", 0, 13..13)),
            vec![Cached::new(("my_file", "some contents"))],
        );
        assert_yaml_snapshot!(s);
    }
}
