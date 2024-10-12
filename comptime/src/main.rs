//! Aim: Like Zig, but with type checked comptime.
//!
//! Means: first order everything. First order references (like Hylo), first order comptime (like
//! Zig), first order functions.

mod ast;
mod env;
mod interp;
mod parser;
mod rt_error;

use interp::run;
use parser::make_prog_parser;
use parser_ll1::CompiledParser;
use std::io;

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
        let input = prompt(&mut input_buffer).unwrap();
        if input.is_empty() {
            break;
        }

        match parser.parse("stdin", input) {
            Err(err) => println!("{}", err),
            Ok(prog) => {
                println!("{:?}", prog);
                match run(prog) {
                    Err(err) => println!("{}", err),
                    Ok(value) => println!("{}", value),
                }
            }
        }
    }

    println!("Goodbye!")
}
