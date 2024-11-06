use comptime::{Language, RunResult, ShowError};
use std::fs;
use std::mem;

const TESTS_PATH: &str = "tests/test_cases.trd";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseState {
    Initial,
    ReadingSource,
    ReadingOutput,
}

#[test]
fn run_tests() {
    use ParseState::{Initial, ReadingOutput, ReadingSource};

    let input = fs::read_to_string(TESTS_PATH).unwrap();

    let mut parse_state = ParseState::Initial;
    let mut test_cases = Vec::new();
    let mut source = String::new();
    let mut output = String::new();
    for line in input.lines() {
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        match (parse_state, line) {
            (Initial, "TEST") => parse_state = ReadingSource,
            (Initial, _) => panic!("Test cases: expected 'TEST'"),
            (ReadingSource, "EXPECT") => parse_state = ReadingOutput,
            (ReadingOutput, "TEST") => {
                test_cases.push((mem::take(&mut source), mem::take(&mut output)));
                parse_state = ReadingSource;
            }
            (ReadingSource, _) => source += line,
            (ReadingOutput, _) => output += line,
        }
    }
    if parse_state != Initial {
        test_cases.push((mem::take(&mut source), mem::take(&mut output)));
    }

    let mut language = Language::new();
    for (source, expected_output) in test_cases {
        let actual_output = format!("    {}", run(&mut language, &source));
        if expected_output != actual_output {
            println!("TEST");
            println!("{}", source);
            println!("EXPECT");
            println!("{}", expected_output);
            println!("ACTUAL");
            for line in actual_output.lines() {
                println!("    {}", line);
            }
            assert_eq!(expected_output, actual_output);
        }
    }
}

fn run(language: &mut Language, source: &str) -> String {
    use RunResult::{ParseError, RuntimeError, Success, TypeError};

    match language.run(source) {
        ParseError(err) => format!("{}", err),
        TypeError(_, err) => brief_error_message(err),
        RuntimeError(_, err) => brief_error_message(err),
        Success(_, value) => format!("{}", value),
    }
}

fn brief_error_message(error: impl ShowError) -> String {
    format!("{}: {}", error.kind(), error.long_message())
}
