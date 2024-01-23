use std::collections::HashMap;
use std::io;
use std::ops::Range;

use termcolor::WriteColor;

use super::sources::{Cached, Sources};
use super::{Config, Diagnostic};

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
        .run()
    }
}

struct DiagnosticWriter<'stream, 'a, W: WriteColor, S: Sources> {
    diagnostic: &'a Diagnostic<S>,
    sources: &'a S,

    stream: &'stream mut W,
    config: &'a Config,
}

impl<'a, W: WriteColor, S: Sources> DiagnosticWriter<'_, 'a, W, S> {
    fn run(mut self) -> io::Result<()> {
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
                .byte_to_line_index(snippet.span.start)
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

#[cfg(test)]
mod tests {
    use super::get_overlapping_groups;

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
}
