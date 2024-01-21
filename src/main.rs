use clap::Parser;
use cli::{Cli, Command};

mod cli;

#[derive(thiserror::Error, Debug)]
enum CompilerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

type CompilerResult<T> = Result<T, CompilerError>;

fn main() -> CompilerResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Build { file } => {
            println!("building file {}", file.display());
            let s = std::fs::read_to_string(file)?;
            println!("s: {s:?}");
        }

        #[cfg(feature = "test_suite")]
        Command::RunTestSuite { stage } => {
            test_suite::run_test_suite(stage)?;
        }
    }

    Ok(())
}

#[cfg(feature = "test_suite")]
mod test_suite {
    use std::fs::{self, DirEntry};
    use std::path::PathBuf;

    use crate::CompilerResult;

    fn get_stage_path(stage: usize) -> PathBuf {
        let mut path = PathBuf::new();

        path.push("test_suite");
        path.push(format!("stage_{stage}"));

        path
    }

    pub fn run_test_suite(stage: usize) -> CompilerResult<()> {
        println!("RUNNING TEST SUITE:");

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

        println!(
            "\n{} of {} tests passed, {} failed",
            output.passed,
            output.total,
            output.total - output.passed,
        );

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
            let s = fs::read_to_string(&path)?;

            print!(" - compiling {}... ", path.display());
            let result = run_source(s);

            let passed = if expect_error {
                if result.is_err() {
                    println!("ok");
                    true
                } else {
                    println!("err (unexpectedly compiled)");
                    false
                }
            } else {
                match result {
                    Ok(_) => {
                        println!("ok");
                        true
                    }
                    Err(err) => {
                        println!("err (failed to compile)");
                        println!("  {err}");
                        false
                    }
                }
            };

            Ok(TestOutput::new(if passed { 1 } else { 0 }, 1))
        }
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

    fn run_source(_s: String) -> CompilerResult<()> {
        Ok(())
    }
}
