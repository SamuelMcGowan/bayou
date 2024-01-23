use std::collections::HashMap;
use std::io;
use std::ops::Range;

use termcolor::{Color, ColorSpec, WriteColor};

use super::sources::{Source, Sources};
use super::{Diagnostic, DiagnosticKind, Snippet};

impl<S: Sources> Diagnostic<S> {
    pub fn write_to_stream_old(
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

    pub fn write_to_stream(
        &self,
        sources: &S,
        config: &Config,
        stream: &mut impl WriteColor,
    ) -> io::Result<()> {
        // split snippets up by source
        let mut file_snippets = HashMap::new();
        for snippet in &self.snippets {
            // get the source entry, or make a new one
            let (source, file_snippets) =
                file_snippets.entry(snippet.source_id).or_insert_with(|| {
                    let source = sources
                        .get_source(snippet.source_id)
                        .expect("source missing");

                    (source, vec![])
                });

            // calculate line range
            let first_line = source
                .byte_to_line_index(snippet.span.start)
                .expect("span start out of bounds");
            let last_line = source
                .byte_to_line_index(snippet.span.end)
                .expect("span end out of bounds")
                + 1;
            let line_range = first_line..last_line;

            file_snippets.push(SnippetProcessed {
                bytes: snippet.span.clone(),
                lines: line_range,
                message: &snippet.label,
            });
        }

        for (source, snippets) in file_snippets.into_values() {
            writeln!(stream, "in {}", source.name_str())?;

            let groups = get_overlapping_ranges(snippets, |snippet| snippet.lines.clone());
            for (snippets, group_line_range) in groups {
                // split snippets into multiline/inline snippets
                let mut multiline_snippets = vec![];
                let mut inline_snippets = vec![];

                for snippet in snippets {
                    if snippet.lines.len() <= 1 {
                        let line = snippet.lines.start;

                        let line_start = source.line_to_byte(line).expect("line out of bounds");
                        let offset = snippet.bytes.start - line_start;

                        inline_snippets.push(InlineSnippet {
                            line,
                            offset, // TODO: proper unicode width
                            width: snippet.bytes.len(),
                            message: snippet.message,
                        });
                    } else {
                        multiline_snippets.push(MultilineSnippet {
                            lines: snippet.lines,
                            message: snippet.message,
                        });
                    }
                }

                // draw snippets
                for line in group_line_range {
                    macro_rules! draw_multiline_snippets {
                        () => {{
                            for snippet in &multiline_snippets {
                                if snippet.lines.contains(&line) {
                                    write!(stream, "| ")?;
                                } else {
                                    write!(stream, "  ")?;
                                }
                            }
                        }};
                    }

                    draw_multiline_snippets!();

                    // draw line
                    let line_range = source.line_range(line).expect("line out of bounds");
                    let line_str = &source.source_str()[line_range].trim_end();
                    writeln!(stream, "{line_str}")?;

                    if !inline_snippets.iter().any(|s| s.line == line) {
                        continue;
                    }

                    draw_multiline_snippets!();

                    // draw inline snippets
                    for snippet in &inline_snippets {
                        if snippet.line != line {
                            continue;
                        }

                        for _ in 0..(multiline_snippets.len() * 2 + snippet.offset) {
                            write!(stream, " ")?;
                        }
                        for _ in 0..snippet.width {
                            write!(stream, "^")?;
                        }
                        writeln!(stream, "{}", snippet.message)?;
                    }
                }
            }
        }

        Ok(())
    }
}

struct SnippetGroup<'a> {
    lines: Range<usize>,

    side_snippets: Vec<MultilineSnippet<'a>>,
    inline_snippets: Vec<InlineSnippet<'a>>,
}

struct MultilineSnippet<'a> {
    lines: Range<usize>,
    message: &'a str,
}

struct InlineSnippet<'a> {
    line: usize,

    offset: usize,
    width: usize,

    message: &'a str,
}

struct SnippetProcessed<'a> {
    bytes: Range<usize>,
    lines: Range<usize>,
    message: &'a str,
}

fn get_overlapping_ranges<T, F: Fn(&T) -> Range<usize>>(
    mut ranges: Vec<T>,
    get_range: F,
) -> Vec<(Vec<T>, Range<usize>)> {
    /*
    - sort ranges by starts
    - go through ranges either adding the next range to the current group, or
      starting a new group
    */

    ranges.sort_by_key(|item| get_range(item).start);

    let mut groups = vec![];

    let mut group = vec![];
    let mut group_start = 0;
    let mut group_end = 0;

    for item in ranges {
        let range = get_range(&item);

        if range.start > group_end && !group.is_empty() {
            groups.push((std::mem::take(&mut group), group_start..group_end));
            group_start = range.start;
        }

        group_end = group_end.max(range.end);
        group.push(item);
    }

    if !group.is_empty() {
        groups.push((group, group_start..group_end));
    }

    groups
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
    use crate::diagnostics::render::get_overlapping_ranges;
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

    #[test]
    fn example_from_ariadne() {
        const SOURCE: &str = "def five = match () in {\n    () => 5,\n    () => \"5\",\n}";

        let diagnostic = Diagnostic::error()
            .with_message("Incompatible types")
            .with_snippet(Snippet::new("This is of type `Nat`", 0, 32..33))
            .with_snippet(Snippet::new("This is of type `Str`", 0, 42..45))
            .with_snippet(Snippet::new(
                "The values are outputs of this `match` expression",
                0,
                11..48,
            ));

        let s = diagnostic_to_string(diagnostic, vec![Cached::new(("sample.tao", SOURCE))]);

        println!("{s}");
    }

    #[test]
    #[allow(clippy::single_range_in_vec_init)]
    fn overlapping_ranges() {
        let ranges = vec![0..1, 0..10, 1..2, 5..7, 11..12];
        let overlapping_ranges = get_overlapping_ranges(ranges, |r| r.clone());

        assert_eq!(
            &overlapping_ranges,
            &[
                (vec![0..1, 0..10, 1..2, 5..7], 0..10),
                (vec![11..12], 11..12)
            ]
        );
    }
}
