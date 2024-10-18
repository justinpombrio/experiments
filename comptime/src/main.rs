//! Aim: Like Zig, but with type checked comptime.
//!
//! Means: first order everything. First order references (like Hylo), first order comptime (like
//! Zig), first order functions.

use comptime::{show_error, FmtResult, Language, RunResult};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Experimental Programming Language
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandLineArgs {
    /// The source file to run. If not provided, start a REPL.
    path: Option<PathBuf>,
    /// Whether to pretty print the source file instead of running it.
    #[arg(short, long)]
    pretty: bool,
}

fn prompt(buffer: &mut String) -> Result<&str, io::Error> {
    use std::io::Write;

    // Write prompt
    print!("> ");
    io::stdout().flush()?;

    // Read line
    buffer.clear();
    io::stdin().read_line(buffer)?;
    Ok(buffer.trim())
}

fn run(language: &mut Language, source: &str) {
    use RunResult::{ParseError, RuntimeError, Success, TypeError};

    match language.run(source) {
        ParseError(err) => println!("{}", err),
        TypeError(_, err) => println!("{}", show_error(err, source)),
        RuntimeError(_, err) => println!("{}", show_error(err, source)),
        Success(_, value) => println!("{}", value),
    }
}

fn fmt(language: &mut Language, source: &str) {
    use FmtResult::{ParseError, Success};

    match language.fmt(source, 80) {
        ParseError(err) => println!("{}", err),
        Success(string) => println!("{}", string),
    }
}

fn repl(language: &mut Language) {
    let mut input_buffer = String::new();
    loop {
        let source = prompt(&mut input_buffer).unwrap();
        if source.is_empty() {
            break;
        }
        run(language, source);
    }
    println!("Goodbye!");
}

fn main() {
    let mut lang = Language::new();

    let args = <CommandLineArgs as clap::Parser>::parse();

    if let Some(path) = args.path {
        let source = fs::read_to_string(path).unwrap();
        if args.pretty {
            fmt(&mut lang, &source);
        } else {
            run(&mut lang, &source);
        }
    } else {
        repl(&mut lang);
    }
}
