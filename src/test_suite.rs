use std::fs::{self, DirEntry};
use std::path::PathBuf;

use owo_colors::OwoColorize;

use crate::{compile, CompilerResult};

pub fn run_test_suite(stage: usize) -> CompilerResult<()> {
    println!("{}", "running tests...".blue().bold());

    let mut output = TestOutput::new(0, 0);

    for stage in 1..=stage {
        let path = get_stage_path(stage);

        let path_valid = path.join("valid");
        let path_invalid = path.join("invalid");

        for entry in fs::read_dir(path_valid)? {
            output.append(run_tests_in_entry(entry?, false)?);
        }

        for entry in fs::read_dir(path_invalid)? {
            output.append(run_tests_in_entry(entry?, true)?);
        }
    }

    print!("\n{} of {} tests passed - ", output.passed, output.total);

    if output.passed == output.total {
        println!("{}", "all ok".green());
    } else {
        println!(
            "{}",
            format_args!("{} tests failed", output.total - output.passed)
                .red()
                .bold()
        );
    }

    Ok(())
}

fn run_tests_in_entry(entry: DirEntry, expect_error: bool) -> CompilerResult<TestOutput> {
    let metadata = entry.metadata()?;
    let path = entry.path();

    if metadata.is_dir() {
        let mut output = TestOutput::new(0, 0);

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            output.append(run_tests_in_entry(entry, expect_error)?);
        }

        Ok(output)
    } else {
        let source = fs::read_to_string(&path)?;

        let result = compile(&source);

        print!(" - {} - ", path.display());

        let passed = if expect_error {
            if result.is_err() {
                println!("{}", "ok".green().bold());
                true
            } else {
                println!("{}", "err (unexpectedly compiled)".red().bold());
                false
            }
        } else {
            match result {
                Ok(_) => {
                    println!("{}", "ok".green().bold());
                    true
                }
                Err(err) => {
                    println!("{}", "err (failed to compile)".red().bold());
                    println!("  {err}");
                    false
                }
            }
        };

        Ok(TestOutput::new(if passed { 1 } else { 0 }, 1))
    }
}

fn get_stage_path(stage: usize) -> PathBuf {
    let mut path = PathBuf::new();

    path.push("test_suite");
    path.push(format!("stage_{stage}"));

    path
}

#[derive(Clone, Copy)]
struct TestOutput {
    passed: usize,
    total: usize,
}

impl TestOutput {
    fn new(passed: usize, total: usize) -> Self {
        Self { passed, total }
    }

    fn append(&mut self, other: Self) {
        self.passed += other.passed;
        self.total += other.total;
    }
}
