mod ast;
// mod comptime;
mod eval_error;
mod memory;
mod parse;
mod pretty_print;
mod runtime;
mod show_error;
mod type_check;
mod type_error;

use parse::make_prog_parser;
use parser_ll1::CompiledParser;
use runtime::run_prog;
use std::default::Default;
use type_check::type_check;

pub use ast::Prog;
pub use eval_error::EvalError;
pub use memory::Value;
pub use parser_ll1::ParseError;
pub use pretty_print::pretty_print;
pub use show_error::show_error;
pub use show_error::ShowError;
pub use type_error::TypeError;

pub struct Language {
    parser: Box<dyn CompiledParser<Prog>>,
}

impl Default for Language {
    fn default() -> Language {
        Language::new()
    }
}

pub enum RunResult {
    ParseError(ParseError),
    TypeError(Prog, TypeError),
    RuntimeError(Prog, EvalError),
    Success(Prog, Value),
}

pub enum FmtResult {
    ParseError(ParseError),
    Success(String),
}

impl Language {
    pub fn new() -> Language {
        let parser = match make_prog_parser() {
            Ok(parser) => parser,
            Err(err) => panic!("{}", err),
        };
        Language {
            parser: Box::new(parser),
        }
    }

    pub fn run(&mut self, source: &str) -> RunResult {
        let mut prog = match self.parser.parse("stdin", source) {
            Ok(prog) => prog,
            Err(err) => {
                return RunResult::ParseError(err);
            }
        };

        if let Err(type_err) = type_check(&mut prog) {
            return RunResult::TypeError(prog, type_err);
        }

        match run_prog(&prog) {
            Err(runtime_err) => RunResult::RuntimeError(prog, runtime_err),
            Ok(value) => RunResult::Success(prog, value),
        }
    }

    pub fn fmt(&mut self, source: &str, width: u16) -> FmtResult {
        let prog = match self.parser.parse("stdin", source) {
            Ok(prog) => prog,
            Err(err) => {
                return FmtResult::ParseError(err);
            }
        };
        FmtResult::Success(pretty_print(&prog, width, false))
    }
}
