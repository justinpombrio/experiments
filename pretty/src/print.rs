use crate::notation::{Notation, NotationInner};

pub fn pretty_print(notation: Notation, printing_width: u32) {
    let mut printer = PrettyPrinter::new(notation, printing_width);
    printer.print();
}

struct PrettyPrinter {
    width: u32,
    col: u32,
    chunks: Vec<Chunk>,
}

#[derive(Debug, Clone)]
struct Chunk {
    notation: Notation,
    indent: u32,
    flat: bool,
}

impl Chunk {
    fn with_notation(self: &Chunk, notation: Notation) -> Chunk {
        Chunk {
            notation,
            indent: self.indent,
            flat: self.flat,
        }
    }

    fn indented(self: &Chunk, indent: u32, notation: Notation) -> Chunk {
        Chunk {
            notation,
            indent: self.indent + indent,
            flat: self.flat,
        }
    }

    fn flat(self: &Chunk, notation: Notation) -> Chunk {
        Chunk {
            notation,
            indent: self.indent,
            flat: true,
        }
    }
}

impl PrettyPrinter {
    fn new(notation: Notation, width: u32) -> PrettyPrinter {
        let chunk = Chunk {
            notation,
            indent: 0,
            flat: false,
        };
        PrettyPrinter {
            width,
            col: 0,
            chunks: vec![chunk],
        }
    }

    fn print(&mut self) {
        use NotationInner::*;

        while let Some(chunk) = self.chunks.pop() {
            match chunk.notation.0.as_ref() {
                Newline => {
                    print!("\n{:spaces$}", "", spaces = chunk.indent as usize);
                    self.col = chunk.indent;
                }
                Text(text, width) => {
                    print!("{}", text);
                    self.col += width;
                }
                Flat(x) => self.chunks.push(chunk.flat(x.clone())),
                Indent(i, x) => self.chunks.push(chunk.indented(*i, x.clone())),
                Concat(x, y) => {
                    self.chunks.push(chunk.with_notation(y.clone()));
                    self.chunks.push(chunk.with_notation(x.clone()));
                }
                Choice(x, y) => {
                    if chunk.flat || self.fits(chunk.with_notation(x.clone())) {
                        self.chunks.push(chunk.with_notation(x.clone()));
                    } else {
                        self.chunks.push(chunk.with_notation(y.clone()));
                    }
                }
            }
        }
    }

    fn fits(&self, chunk: Chunk) -> bool {
        use NotationInner::*;

        let mut remaining = self.width.saturating_sub(self.col);
        let mut stack = vec![chunk];
        let mut chunks = &self.chunks as &[Chunk];

        loop {
            let chunk = match stack.pop() {
                Some(chunk) => chunk,
                None => match chunks.split_last() {
                    None => return true,
                    Some((chunk, more_chunks)) => {
                        chunks = more_chunks;
                        chunk.clone()
                    }
                },
            };

            match chunk.notation.0.as_ref() {
                Newline => return true,
                Text(_text, text_width) => {
                    if *text_width <= remaining {
                        remaining -= *text_width;
                    } else {
                        return false;
                    }
                }
                Flat(x) => stack.push(chunk.flat(x.clone())),
                Indent(i, x) => stack.push(chunk.indented(*i, x.clone())),
                Concat(x, y) => {
                    stack.push(chunk.with_notation(y.clone()));
                    stack.push(chunk.with_notation(x.clone()));
                }
                Choice(x, y) => {
                    if chunk.flat {
                        stack.push(chunk.with_notation(x.clone()));
                    } else {
                        // Relies on the rule that for every choice `x | y`,
                        // the first line of `y` is no longer than the first line of `x`.
                        stack.push(chunk.with_notation(y.clone()));
                    }
                }
            }
        }
    }
}
