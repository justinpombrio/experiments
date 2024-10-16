//! Aim: Like Zig, but with type checked comptime.
//!
//! Means: first order everything. First order references (like Hylo), first order comptime (like
//! Zig), first order functions.

mod ast;
mod env;
mod interp;
mod parse;
mod runtime_error;
mod show_error;
mod type_check;
mod type_error;

use interp::run;
use parse::make_prog_parser;
use parser_ll1::CompiledParser;
use show_error::show_error;
use std::io;
use type_check::type_check;

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

fn main() {
    let parser = match make_prog_parser() {
        Ok(parser) => parser,
        Err(err) => panic!("{}", err),
    };

    let mut input_buffer = String::new();
    loop {
        let source = prompt(&mut input_buffer).unwrap();
        if source.is_empty() {
            break;
        }

        let prog = match parser.parse("stdin", source) {
            Ok(prog) => prog,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };

        if let Err(type_err) = type_check(&prog) {
            println!("{}", show_error(type_err, source));
            continue;
        }

        match run(source, &prog) {
            Err(runtime_err) => println!("{}", show_error(runtime_err, source)),
            Ok(value) => println!("{}", value),
        }
    }

    println!("Goodbye!")
}
