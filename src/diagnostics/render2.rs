use std::collections::HashMap;
use std::io;
use std::ops::Range;

use termcolor::WriteColor;
use unicode_width::UnicodeWidthStr;

use super::sources::{Cached, Source, Sources};
use super::{Config, Diagnostic};

const TAB: &str = "    ";

impl<S: Sources> Diagnostic<S> {
    pub fn write_to_stream(
        &self,
        sources: &S,
        config: &Config,
        stream: &mut impl WriteColor,
    ) -> io::Result<()> {
        DiagnosticWriter {
            diagnostic: self,
            sources,
            stream,
            config,
        }
        .draw_all()
    }
}

struct DiagnosticWriter<'stream, 'a, W: WriteColor, S: Sources> {
    diagnostic: &'a Diagnostic<S>,
    sources: &'a S,

    stream: &'stream mut W,
    config: &'a Config,
}

impl<'a, W: WriteColor, S: Sources> DiagnosticWriter<'_, 'a, W, S> {
    fn draw_all(mut self) -> io::Result<()> {
        let source_datas = self.snippets_by_source();

        for source_data in source_datas.into_values() {
            let groups = get_overlapping_groups(source_data.snippets, |s| s.lines.clone());
            for (snippets, lines) in groups {
                self.draw_group(source_data.source, &snippets, lines)?;
            }
        }

        Ok(())
    }

    fn draw_group(
        &mut self,
        source: &Cached<S::Source>,
        snippets: &[SnippetData],
        lines: Range<usize>,
    ) -> io::Result<()> {
        let mut multiline_snippets = vec![];
        let mut inline_snippets = vec![];

        for snippet in snippets.iter().cloned() {
            if snippet.lines.len() > 1 {
                multiline_snippets.push(snippet);
            } else {
                inline_snippets.push(snippet);
            }
        }

        for line in lines {
            self.draw_multiline_snippets(&multiline_snippets, line, true)?;

            let line_str = source
                .line_str(line)
                .expect("line out of bounds")
                .replace('\t', TAB);

            writeln!(self.stream, "{line_str}")?;

            let line_start = source.line_to_byte(line).expect("line out of bounds");
            for snippet in &inline_snippets {
                if snippet.lines.start != line {
                    continue;
                }

                self.draw_multiline_snippets(&multiline_snippets, line, false)?;

                let before_snippet = &source.source_str()[line_start..snippet.bytes.start];
                let offset = str_width(before_snippet);

                for _ in 0..offset {
                    write!(self.stream, " ")?;
                }

                for _ in 0..snippet.bytes.len() {
                    write!(self.stream, "{}", self.config.underline)?;
                }

                writeln!(
                    self.stream,
                    "{}{}{}",
                    self.config.underline_trace, self.config.underline_trace, snippet.label
                )?;
            }
        }

        Ok(())
    }

    fn draw_multiline_snippets(
        &mut self,
        multiline_snippets: &[SnippetData],
        line: usize,
        source_line: bool,
    ) -> io::Result<()> {
        for snippet in multiline_snippets {
            let ch = if source_line {
                if line < snippet.lines.start {
                    ' '
                } else if line == snippet.lines.start {
                    self.config.gutter_top
                } else if line + 1 == snippet.lines.end {
                    self.config.gutter_bottom
                } else if line < snippet.lines.end {
                    self.config.gutter_main
                } else {
                    self.config.gutter_trace
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if line < snippet.lines.start {
                    ' '
                } else if line + 1 >= snippet.lines.end {
                    self.config.gutter_trace
                } else {
                    self.config.gutter_main
                }
            };

            write!(self.stream, "{ch} ")?;
        }

        Ok(())
    }

    fn snippets_by_source(&self) -> HashMap<S::SourceId, SourceData<'a, S>> {
        let mut source_datas = HashMap::new();

        for snippet in &self.diagnostic.snippets {
            let source_data = source_datas
                .entry(snippet.source_id)
                .or_insert_with(|| SourceData {
                    source: self
                        .sources
                        .get_source(snippet.source_id)
                        .expect("source missing"),
                    snippets: vec![],
                });

            let start_line = source_data
                .source
                .byte_to_line_index(snippet.span.start)
                .expect("span start out of bounds");
            let end_line = source_data
                .source
                .byte_to_line_index(snippet.span.end)
                .expect("span end out of bounds")
                + 1;
            let lines = start_line..end_line;

            source_data.snippets.push(SnippetData {
                label: &snippet.label,
                bytes: snippet.span.clone(),
                lines,
            });
        }

        source_datas
    }
}

struct SourceData<'a, S: Sources> {
    source: &'a Cached<S::Source>,
    snippets: Vec<SnippetData<'a>>,
}

#[derive(Clone)]
struct SnippetData<'a> {
    label: &'a str,

    bytes: Range<usize>,
    lines: Range<usize>,
}

fn get_overlapping_groups<T, F: Fn(&T) -> Range<usize>>(
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

fn str_width(s: &str) -> usize {
    let num_tabs = s.chars().filter(|&ch| ch == '\t').count();
    s.width() + num_tabs * TAB.len()
}

#[cfg(test)]
mod tests {
    use termcolor::NoColor;

    use super::get_overlapping_groups;
    use crate::diagnostics::sources::{Cached, Sources};
    use crate::diagnostics::{Config, Diagnostic, Snippet};

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
    #[allow(clippy::single_range_in_vec_init)]
    fn overlapping_ranges() {
        let ranges = vec![0..1, 0..10, 1..2, 5..7, 11..12];
        let overlapping_ranges = get_overlapping_groups(ranges, |r| r.clone());

        assert_eq!(
            &overlapping_ranges,
            &[
                (vec![0..1, 0..10, 1..2, 5..7], 0..10),
                (vec![11..12], 11..12)
            ]
        );
    }

    #[test]
    fn example_from_ariadne() {
        const SOURCE: &str = "def five = match () in {\n\t() => 5,\n\t() => \"5\",\n}";

        let diagnostic = Diagnostic::error()
            .with_message("Incompatible types")
            .with_snippet(Snippet::new("This is of type `Nat`", 0, 32..33))
            .with_snippet(Snippet::new("This is of type `Str`", 0, 42..45))
            .with_snippet(Snippet::new(
                "The values are outputs of this `match` expression",
                0,
                11..48,
            ))
            .with_snippet(Snippet::new("hehe", 0, 32..45));

        let s = diagnostic_to_string(diagnostic, vec![Cached::new(("sample.tao", SOURCE))]);

        println!("{s}");
    }
}
