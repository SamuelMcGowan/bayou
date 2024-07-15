use bayou_diagnostic::sources::Cached;
use bayou_diagnostic::{Config, Diagnostic, Snippet};
use termcolor::{ColorChoice, StandardStream};

fn main() {
    let input = "fn (x, y!) => [5 + 4)]\n\n\n\n\n\n\n\n\naaa";

    let sources = vec![Cached::new(("repl".to_owned(), input.to_owned()))];

    let diagnostics = vec![
        Diagnostic::error()
            .with_id("E03")
            .with_message(
                "Unexpected `!` while parsing pattern, expected one of `:`, `,`, `::`, `)`",
            )
            .with_snippet(Snippet::primary("Unexpected `!`", 0, 8..9)),
        Diagnostic::error()
            .with_id("E03")
            .with_message("Unclosed delimiter `[` while parsing expression, expected `]`")
            .with_snippet(Snippet::primary("Delimiter `[` is never closed", 0, 14..15))
            .with_snippet(Snippet::secondary(
                "Must be closed before this `)`",
                0,
                20..21,
            ))
            .with_snippet(Snippet::secondary("Also look at this", 0, 0..34))
            .with_note("Nice weather today")
            .with_suggestion("Try fixing the problem"),
    ];

    let config = Config::default();
    let mut stream = StandardStream::stderr(ColorChoice::Auto);
    for diagnostic in diagnostics {
        diagnostic
            .write_to_stream(&sources, &config, &mut stream)
            .unwrap();
    }
}
