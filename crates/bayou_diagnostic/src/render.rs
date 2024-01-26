use std::collections::HashMap;
use std::io;

use termcolor::{ColorSpec, WriteColor};
use unicode_width::UnicodeWidthStr;

use super::sources::{Cached, Source, Sources};
// use super::span2::Span;
use super::{Config, Diagnostic, DiagnosticKind, SnippetKind};
use crate::span::Span;

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
        self.draw_header()?;

        let source_datas = self.snippets_by_source();

        if source_datas.is_empty() {
            writeln!(self.stream)?;
        }

        for source_data in source_datas.into_values() {
            let groups = get_overlapping_groups(source_data.snippets, |s| s.lines);
            for (snippets, mut lines) in groups {
                lines.start = lines.start.saturating_sub(self.config.context_size);
                lines.end =
                    (lines.end + self.config.context_size).min(source_data.source.num_lines());

                self.draw_group(source_data.source, &snippets, lines)?;
            }
        }

        Ok(())
    }

    fn draw_header(&mut self) -> io::Result<()> {
        self.stream.set_color(self.get_primary_color())?;

        if let Some(id) = &self.diagnostic.id {
            write!(self.stream, "[{id}] ")?;
        }

        let kind_str = self.get_kind_str();
        write!(self.stream, "{kind_str}:")?;

        self.stream.reset()?;

        if let Some(message) = &self.diagnostic.message {
            writeln!(self.stream, " {message}")?;
        }

        Ok(())
    }

    fn draw_group(
        &mut self,
        source: &Cached<S::Source>,
        snippets: &[SnippetData],
        lines: Span,
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

        let line_num_width = 1 + (lines.end.saturating_sub(1).max(1)).ilog10() as usize;

        // all groups have at least one snippet
        let (line_num, col_num) = source
            .byte_to_line_col(snippets[0].bytes.start)
            .expect("position out of bounds");

        self.stream.set_color(&self.config.subtle)?;
        write!(self.stream, "In {}:{line_num}:{col_num}", source.name_str())?;

        if let Some(path) = source.path() {
            write!(self.stream, " ({}:{line_num}:{col_num})", path.display())?;
        }

        writeln!(self.stream)?;
        self.stream.reset()?;

        for line in lines {
            self.draw_gutter(Some(line), line_num_width)?;
            self.draw_multilines(&multiline_snippets, line, true)?;

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

                self.draw_gutter(None, line_num_width)?;
                self.draw_multilines(&multiline_snippets, line, false)?;

                let before_snippet = &source.source_str()[line_start..snippet.bytes.start];
                let offset = str_width(before_snippet);

                self.stream
                    .set_color(self.get_snippet_color(snippet.kind))?;

                write!(self.stream, "{:<offset$}", "")?;

                for _ in 0..snippet.bytes.len().max(1) {
                    write!(self.stream, "{}", self.config.underline)?;
                }

                writeln!(
                    self.stream,
                    "{}{}",
                    self.config.underline_after, snippet.label
                )?;

                self.stream.reset()?;
            }
        }

        self.draw_multiline_labels(&multiline_snippets, line_num_width)?;

        writeln!(self.stream)?;

        Ok(())
    }

    fn draw_gutter(&mut self, line: Option<usize>, line_num_width: usize) -> io::Result<()> {
        self.stream.set_color(&self.config.subtle)?;

        if let Some(line) = line {
            write!(self.stream, "{line:>width$}", width = line_num_width)?;
        } else {
            write!(self.stream, "{:>width$}", "", width = line_num_width)?;
        }

        write!(self.stream, " {} ", self.config.gutter)?;

        self.stream.reset()?;

        Ok(())
    }

    fn draw_multilines(
        &mut self,
        multiline_snippets: &[SnippetData],
        line: usize,
        source_line: bool,
    ) -> io::Result<()> {
        for snippet in multiline_snippets {
            self.stream
                .set_color(self.get_snippet_color(snippet.kind))?;

            let ch = if source_line {
                if line < snippet.lines.start {
                    self.config.multiline_empty
                } else if line == snippet.lines.start {
                    self.config.multiline_top
                } else if line + 1 == snippet.lines.end {
                    self.config.multiline_bottom
                } else {
                    self.config.multiline_main
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if line < snippet.lines.start {
                    self.config.multiline_empty
                } else {
                    self.config.multiline_main
                }
            };

            write!(self.stream, "{ch} ")?;
        }

        self.stream.reset()?;

        Ok(())
    }

    fn draw_multilines_simple(&mut self, multiline_snippets: &[SnippetData]) -> io::Result<()> {
        for snippet in multiline_snippets {
            self.stream
                .set_color(self.get_snippet_color(snippet.kind))?;

            write!(self.stream, "{} ", self.config.multiline_main)?;
        }

        Ok(())
    }

    fn draw_multiline_labels(
        &mut self,
        mut multiline_snippets: &[SnippetData],
        line_num_width: usize,
    ) -> io::Result<()> {
        if multiline_snippets.is_empty() {
            return Ok(());
        }

        // blank line
        self.draw_gutter(None, line_num_width)?;
        self.draw_multilines_simple(multiline_snippets)?;
        writeln!(self.stream)?;

        while let [prevs @ .., snippet] = multiline_snippets {
            self.draw_gutter(None, line_num_width)?;
            self.draw_multilines_simple(prevs)?;

            self.stream
                .set_color(self.get_snippet_color(snippet.kind))?;

            write!(
                self.stream,
                "{} {}",
                self.config.multiline_very_bottom, snippet.label
            )?;

            writeln!(self.stream)?;

            multiline_snippets = prevs;
        }

        self.stream.reset()?;

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

            source_data.snippets.push(SnippetData {
                label: &snippet.label,
                kind: snippet.kind,

                bytes: snippet.span,
                lines: Span::new(start_line, end_line),
            });
        }

        source_datas
    }

    // TODO: make this a method on `DiagnosticKind`.
    fn get_kind_str(&self) -> &'static str {
        match self.diagnostic.kind {
            DiagnosticKind::Warning => "Warning",
            DiagnosticKind::Error => "Error",
        }
    }

    fn get_primary_color(&self) -> &'a ColorSpec {
        match self.diagnostic.kind {
            DiagnosticKind::Warning => &self.config.warning_color,
            DiagnosticKind::Error => &self.config.error_color,
        }
    }

    fn get_secondary_color(&self) -> &'a ColorSpec {
        &self.config.emphasis
    }

    fn get_snippet_color(&self, kind: SnippetKind) -> &'a ColorSpec {
        match kind {
            SnippetKind::Primary => self.get_primary_color(),
            SnippetKind::Secondary => self.get_secondary_color(),
        }
    }
}

struct SourceData<'a, S: Sources> {
    source: &'a Cached<S::Source>,
    snippets: Vec<SnippetData<'a>>,
}

#[derive(Clone)]
struct SnippetData<'a> {
    label: &'a str,
    kind: SnippetKind,

    bytes: Span,
    lines: Span,
}

fn get_overlapping_groups<T, F>(mut ranges: Vec<T>, get_range: F) -> Vec<(Vec<T>, Span)>
where
    F: Fn(&T) -> Span,
{
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

        if range.start > group_end {
            if !group.is_empty() {
                groups.push((
                    std::mem::take(&mut group),
                    Span::new(group_start, group_end),
                ));
            }

            group_start = range.start;
        }

        group_end = group_end.max(range.end);
        group.push(item);
    }

    if !group.is_empty() {
        groups.push((group, Span::new(group_start, group_end)));
    }

    groups
}

fn str_width(s: &str) -> usize {
    let num_tabs = s.chars().filter(|&ch| ch == '\t').count();
    s.width() + num_tabs * TAB.len()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use termcolor::Ansi;

    use super::get_overlapping_groups;
    use crate::sources::{Cached, Sources};
    use crate::span::Span;
    use crate::{Config, Diagnostic, Snippet};

    #[must_use]
    fn diagnostic_to_string<S: Sources>(diagnostic: Diagnostic<S>, sources: S) -> String {
        let config = Config::default();
        let mut stream = Ansi::new(vec![]);

        diagnostic
            .write_to_stream(&sources, &config, &mut stream)
            .unwrap();

        let bytes = stream.into_inner();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    #[test]
    #[allow(clippy::single_range_in_vec_init)]
    fn overlapping_ranges() {
        let ranges = vec![
            Span::new(0, 1),
            Span::new(0, 10),
            Span::new(1, 2),
            Span::new(5, 7),
            Span::new(11, 12),
        ];
        let overlapping_ranges = get_overlapping_groups(ranges, |&r| r);

        assert_eq!(
            &overlapping_ranges,
            &[
                (
                    vec![
                        Span::new(0, 1),
                        Span::new(0, 10),
                        Span::new(1, 2),
                        Span::new(5, 7)
                    ],
                    Span::new(0, 10)
                ),
                (vec![Span::new(11, 12)], Span::new(11, 12))
            ]
        );
    }

    #[test]
    fn example_from_ariadne() {
        const SOURCE: &str = "def five = match () in {\n\t() => 5,\n\t() => \"5\",\n}";

        let diagnostic = Diagnostic::error()
            .with_message("Incompatible types")
            .with_id("E03")
            .with_snippet(Snippet::primary("This is of type `Nat`", 0, 32..33))
            .with_snippet(Snippet::secondary("This is of type `Str`", 0, 42..45))
            .with_snippet(Snippet::secondary(
                "The values are outputs of this `match` expression",
                0,
                11..48,
            ))
            .with_snippet(Snippet::primary("hehe", 0, 32..45));

        let s = diagnostic_to_string(
            diagnostic,
            vec![Cached::new((
                "main".to_owned(),
                PathBuf::from("sample.tao"),
                SOURCE.to_owned(),
            ))],
        );

        println!("{s}");
    }
}
