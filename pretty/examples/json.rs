use pretty::{pretty_print, NotationBuilder, NotationRef};

#[derive(Clone, Copy)]
struct JsonBuilder<'a>(&'a NotationBuilder<'a>);

impl<'a> JsonBuilder<'a> {
    fn json_to_notation(self, json: serde_json::Value) -> NotationRef<'a> {
        use serde_json::Value::{Array, Bool, Null, Number, Object, String};

        match json {
            Null => self.json_null(),
            Bool(b) => self.json_bool(b),
            Number(n) => self.json_number(n),
            String(s) => self.json_string(&s),
            Array(elems) => {
                self.json_array(elems.into_iter().map(|elem| self.json_to_notation(elem)))
            }
            Object(entries) => self.json_object(
                entries
                    .into_iter()
                    .map(|(key, val)| (key, self.json_to_notation(val))),
            ),
        }
    }

    fn json_null(self) -> NotationRef<'a> {
        self.0.txt("false")
    }

    fn json_bool(self, b: bool) -> NotationRef<'a> {
        if b {
            self.0.txt("true")
        } else {
            self.0.txt("false")
        }
    }

    fn json_string(self, s: &str) -> NotationRef<'a> {
        // TODO: escape sequences
        self.0.txt(format!("\"{}\"", s))
    }

    fn json_number(self, n: impl ToString) -> NotationRef<'a> {
        self.0.txt(n)
    }

    fn json_array(self, elems: impl IntoIterator<Item = NotationRef<'a>>) -> NotationRef<'a> {
        let elems = elems.into_iter().collect::<Vec<_>>();
        self.surrounded("[", &elems, "]")
    }

    fn json_object_entry(self, key: String, value: NotationRef<'a>) -> NotationRef<'a> {
        self.0
            .concat([self.json_string(&key), self.0.txt(": "), value])
    }

    fn json_object(
        self,
        entries: impl IntoIterator<Item = (String, NotationRef<'a>)>,
    ) -> NotationRef<'a> {
        let entries = entries
            .into_iter()
            .map(|(key, val)| self.json_object_entry(key, val))
            .collect::<Vec<_>>();
        self.surrounded("{", &entries, "}")
    }

    fn comma_sep_single_line(self, elems: &[NotationRef<'a>]) -> NotationRef<'a> {
        let mut list = self.0.flat(elems[0]);
        for elem in &elems[1..] {
            list = self.0.concat([list, self.0.txt(", "), self.0.flat(elem)]);
        }
        list
    }

    fn comma_sep_multi_line(self, elems: &[NotationRef<'a>]) -> NotationRef<'a> {
        let mut list = elems[0];
        for elem in &elems[1..] {
            list = self.0.concat([list, self.0.txt(", "), self.0.nl(), elem]);
        }
        list
    }

    fn surrounded(self, open: &str, elems: &[NotationRef<'a>], closed: &str) -> NotationRef<'a> {
        if elems.is_empty() {
            return self.0.txt(format!("{}{}", open, closed));
        }

        let single_line = self.0.concat([
            self.0.txt(open),
            self.comma_sep_single_line(elems),
            self.0.txt(closed),
        ]);
        let multi_line = self.0.concat([
            self.0.txt(open),
            self.0.indent(
                4,
                self.0
                    .concat([self.0.nl(), self.comma_sep_multi_line(elems)]),
            ),
            self.0.nl(),
            self.0.txt(closed),
        ]);
        self.0.choice(single_line, multi_line)
    }
}

fn main() {
    use std::env;
    use std::fs::File;
    use std::io::BufReader;
    use std::time::Instant;

    // Get the filename to parse from the command line args
    let env_args = env::args().collect::<Vec<_>>();
    if env_args.len() != 2 {
        panic!("Usage: cargo run --release --example json FILENAME.json");
    }
    let filename = &env_args[1];

    // Parse the file into json using serde
    let start = Instant::now();
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let json = serde_json::from_reader(reader).unwrap();
    let ms_to_parse = start.elapsed().as_millis();

    // Convert it to a Notation
    let start = Instant::now();
    let notation_builder = NotationBuilder::new();
    let json_builder = JsonBuilder(&notation_builder);
    let notation = json_builder.json_to_notation(json);
    let ms_to_construct = start.elapsed().as_millis();

    // Pretty print the Notation
    let start = Instant::now();
    let output = pretty_print(notation, 120);
    let ms_to_pretty_print = start.elapsed().as_millis();

    // Print to terminal
    let start = Instant::now();
    println!("{}", output);
    let ms_to_output = start.elapsed().as_millis();

    // Print timing info to stderr
    eprintln!("Time to parse file as Json:    {} ms", ms_to_parse);
    eprintln!("Time to construct Notation:    {} ms", ms_to_construct);
    eprintln!("Time to pretty print Notation: {} ms", ms_to_pretty_print);
    eprintln!("Time to pretty to terminal:    {} ms", ms_to_output);
}
