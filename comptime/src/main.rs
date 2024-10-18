//! Aim: Like Zig, but with type checked comptime.
//!
//! Means: first order everything. First order references (like Hylo), first order comptime (like
//! Zig), first order functions.

mod ast;
mod env;
mod interp;
mod parse;
mod pretty_print;
mod runtime_error;
mod show_error;
mod type_check;
mod type_error;

use ast::Prog;
use interp::run_prog;
use parse::make_prog_parser;
use parser_ll1::CompiledParser;
use pretty_print::pretty_print;
use show_error::show_error;
use std::fs;
use std::io;
use std::path::PathBuf;
use type_check::type_check;

/// Experimental Programming Language
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandLineArgs {
    /// The source file to run. If not provided, start a REPL.
    path: Option<PathBuf>,
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

fn run(parser: &impl CompiledParser<Prog>, source: &str) {
    let prog = match parser.parse("stdin", source) {
        Ok(prog) => prog,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    println!("{}", pretty_print(&prog, 80, true));

    if let Err(type_err) = type_check(&prog) {
        println!("{}", show_error(type_err, source));
        return;
    }

    match run_prog(&prog) {
        Err(runtime_err) => println!("{}", show_error(runtime_err, source)),
        Ok(value) => println!("{}", value),
    }
}

fn repl(parser: &impl CompiledParser<Prog>) {
    let mut input_buffer = String::new();
    loop {
        let source = prompt(&mut input_buffer).unwrap();
        if source.is_empty() {
            break;
        }
        run(parser, source);
    }
    println!("Goodbye!");
}

fn main() {
    let parser = match make_prog_parser() {
        Ok(parser) => parser,
        Err(err) => panic!("{}", err),
    };

    let args = <CommandLineArgs as clap::Parser>::parse();

    if let Some(path) = args.path {
        let source = fs::read_to_string(path).unwrap();
        run(&parser, &source);
    } else {
        repl(&parser);
    }
}
