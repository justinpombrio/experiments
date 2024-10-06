//! Aim: Like Zig, but with type checked comptime.
//!
//! Means: first order everything. First order references (like Hylo), first order comptime (like
//! Zig), first order functions.

mod expr;
mod interp;
mod parser;
mod rt_error;
mod value;

use interp::interp_expr;
use parser::make_expr_parser;
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
    let parser = match make_expr_parser() {
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
            Ok(expr) => match interp_expr(&expr) {
                Err(err) => println!("{}", err),
                Ok(value) => println!("{}", value),
            },
        }
    }

    println!("Goodbye!")
}
