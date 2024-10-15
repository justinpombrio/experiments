use crate::ast::Loc;
use colored::Colorize;
use std::fmt;
use std::fmt::Write;

pub trait PrettyError {
    fn kind(&self) -> &'static str;
    fn loc(&self) -> Option<Loc>;
    fn short_message(&self) -> String;
    fn long_message(&self) -> String;
}

pub fn display_error(error: impl PrettyError, source: &str) -> String {
    let mut buffer = String::new();
    match display_error_impl(&mut buffer, error, source) {
        Ok(()) => buffer,
        Err(fmt_err) => format!("Failed to display error message: {fmt_err}"),
    }
}

fn display_error_impl(
    mut buffer: impl Write,
    error: impl PrettyError,
    source: &str,
) -> fmt::Result {
    writeln!(
        &mut buffer,
        "{}{} {}",
        error.kind().red().bold(),
        ":".bold(),
        error.long_message().bold()
    )?;

    if let Some((start, end)) = error.loc() {
        let line_num = format!("{}", start.line + 1);
        let margin_width = line_num.len();
        let line_contents = match source.lines().nth(start.line as usize) {
            Some(line) => line.to_owned(),
            None => panic!("bug: invalid line number: {}", start.line),
        };
        let num_carets = if start.line == end.line {
            (end.utf8_col - start.utf8_col).max(1) as usize
        } else {
            line_contents.chars().count() - start.utf8_col as usize
        };

        writeln!(
            &mut buffer,
            "{:indent$}{} stdin:{}:{}",
            "",
            "-->".blue().bold(),
            start.line + 1,
            start.utf8_col + 1,
            indent = margin_width,
        )?;
        writeln!(
            &mut buffer,
            "{:indent$}{}",
            "",
            "|".blue().bold(),
            indent = margin_width + 1
        )?;
        writeln!(
            &mut buffer,
            "{} {}{}",
            line_num.blue().bold(),
            "|".blue().bold(),
            line_contents,
        )?;
        writeln!(
            &mut buffer,
            "{:indent$}{}{:start$}{} {}",
            "",
            "|".blue().bold(),
            "",
            &"^".repeat(num_carets).red().bold(),
            error.short_message().red().bold(),
            start = start.utf8_col as usize,
            indent = margin_width + 1
        )?;
        write!(
            &mut buffer,
            "{:indent$}{}",
            "",
            "|".blue().bold(),
            indent = margin_width + 1
        )?;
    }

    Ok(())
}
