use std::fmt;

pub struct IndentWriter<'a> {
    fmt: fmt::Formatter<'a>,
    indent: usize,
}

impl<'a> fmt::Write for IndentWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut lines = s.lines();
        if let Some(first_line) = lines.next() {
            self.fmt.write_str(first_line)?;
        }
        for line in lines {
            for _ in 0..self.indent {
                self.fmt.write_str(" ")?;
            }
            self.fmt.write_str(line)?;
        }
        Ok(())
    }
}
