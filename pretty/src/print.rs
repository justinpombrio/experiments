use crate::notation::{Notation, NotationRef};

pub fn pretty_print(notation: NotationRef, printing_width: u32) -> String {
    let mut printer = PrettyPrinter::new(notation, printing_width);
    printer.print()
}

struct PrettyPrinter<'a> {
    width: u32,
    col: u32,
    chunks: Vec<Chunk<'a>>,
}

#[derive(Debug, Clone, Copy)]
struct Chunk<'a> {
    notation: NotationRef<'a>,
    indent: u32,
    flat: bool,
}

impl<'a> Chunk<'a> {
    fn with_notation(self, notation: NotationRef<'a>) -> Chunk<'a> {
        Chunk {
            notation,
            indent: self.indent,
            flat: self.flat,
        }
    }

    fn indented(self, indent: u32, notation: NotationRef<'a>) -> Chunk<'a> {
        Chunk {
            notation,
            indent: self.indent + indent,
            flat: self.flat,
        }
    }

    fn flat(self, notation: NotationRef<'a>) -> Chunk<'a> {
        Chunk {
            notation,
            indent: self.indent,
            flat: true,
        }
    }
}

impl<'a> PrettyPrinter<'a> {
    fn new(notation: NotationRef<'a>, width: u32) -> PrettyPrinter<'a> {
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

    fn print(&mut self) -> String {
        use Notation::*;

        let mut output = String::new();
        while let Some(chunk) = self.chunks.pop() {
            match chunk.notation {
                Newline => {
                    output.push('\n');
                    for _ in 0..chunk.indent {
                        output.push(' ');
                    }
                    self.col = chunk.indent;
                }
                Text(text, width) => {
                    output.push_str(text);
                    self.col += width;
                }
                Flat(x) => self.chunks.push(chunk.flat(x)),
                Indent(i, x) => self.chunks.push(chunk.indented(*i, x)),
                Concat(seq) => {
                    for notation in seq.iter().rev() {
                        self.chunks.push(chunk.with_notation(notation));
                    }
                }
                Choice(x, y) => {
                    if chunk.flat || self.fits(chunk.with_notation(x)) {
                        self.chunks.push(chunk.with_notation(x));
                    } else {
                        self.chunks.push(chunk.with_notation(y));
                    }
                }
            }
        }
        output
    }

    fn fits(&self, chunk: Chunk<'a>) -> bool {
        use Notation::*;

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
                        *chunk
                    }
                },
            };

            match chunk.notation {
                Newline => return true,
                Text(_text, text_width) => {
                    if *text_width <= remaining {
                        remaining -= *text_width;
                    } else {
                        return false;
                    }
                }
                Flat(x) => stack.push(chunk.flat(x)),
                Indent(i, x) => stack.push(chunk.indented(*i, x)),
                Concat(seq) => {
                    for notation in seq.iter().rev() {
                        stack.push(chunk.with_notation(notation));
                    }
                }
                Choice(x, y) => {
                    if chunk.flat {
                        stack.push(chunk.with_notation(x));
                    } else {
                        // Relies on the rule that for every choice `x | y`,
                        // the first line of `y` is no longer than the first line of `x`.
                        stack.push(chunk.with_notation(y));
                    }
                }
            }
        }
    }
}
