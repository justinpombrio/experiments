mod ll1;
mod nomp;

use std::env;
use std::fmt;
use std::io;

trait JsonParser {
    fn new() -> Self
    where
        Self: Sized;
    fn name(&self) -> &'static str;
    fn parse_json(&self, input: &str) -> Json;
}

#[derive(Debug, Clone)]
enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn write(&self, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
        use Json::*;

        const INDENT: usize = 2;

        match self {
            Null => write!(f, "null"),
            Boolean(false) => write!(f, "false"),
            Boolean(true) => write!(f, "true"),
            Number(n) => write!(f, "{}", n),
            String(s) => write!(f, "\"{}\"", s),
            Array(elems) => {
                writeln!(f, "[")?;
                for (i, elem) in elems.iter().enumerate() {
                    write!(f, "{:indent$}", "", indent = INDENT * (indent + 1))?;
                    elem.write(f, indent + 1)?;
                    if i + 1 != elems.len() {
                        writeln!(f, ",")?;
                    } else {
                        writeln!(f)?;
                    }
                }
                write!(f, "{:indent$}", "", indent = INDENT * indent)?;
                write!(f, "]")
            }
            Object(entries) => {
                writeln!(f, "{{")?;
                for (i, entry) in entries.iter().enumerate() {
                    write!(f, "{:indent$}", "", indent = INDENT * (indent + 1))?;
                    write!(f, "\"{}\": ", entry.0,)?;
                    entry.1.write(f, indent + 1)?;
                    if i + 1 != entries.len() {
                        writeln!(f, ",")?;
                    } else {
                        writeln!(f)?;
                    }
                }
                write!(f, "{:indent$}", "", indent = INDENT * indent)?;
                write!(f, "}}")
            }
        }
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write(f, 0)?;
        write!(f, "\n")
    }
}

fn main() {
    use ll1::LL1Parser;
    use nomp::NomParser;
    use std::time::Instant;

    // Construct parsers
    let now = Instant::now();
    let parsers: Vec<Box<dyn JsonParser>> =
        vec![Box::new(LL1Parser::new()), Box::new(NomParser::new())];
    println!(
        "Time to construct all parsers: {} μs",
        now.elapsed().as_micros()
    );
    let parser_names = parsers
        .iter()
        .map(|parser| parser.name())
        .collect::<Vec<_>>();

    // Find parser from command line args
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!(
            "Must pass one argument saying which parser library to use: {}",
            parser_names.join(", ")
        );
    }
    let parser = match parser_names.iter().position(|n| n == &args[1]) {
        Some(index) => &parsers[index],
        None => panic!(
            "Parser library argument must be one of: {}",
            parser_names.join(", ")
        ),
    };

    // Read stdin
    let now = Instant::now();
    let input = io::read_to_string(io::stdin()).unwrap();
    println!("Time to read stdin: {} μs", now.elapsed().as_micros());

    // Parse json
    let now = Instant::now();
    let json = parser.parse_json(&input);
    println!("Time to parse json: {} μs", now.elapsed().as_micros());

    // Format json
    let now = Instant::now();
    let printed_json = format!("{}", json);
    println!("Time to print json: {} μs", now.elapsed().as_micros());

    // Check if the input was well formatted
    if input == printed_json {
        println!("Well formatted");
    } else {
        let zipped_lines = input.lines().zip(printed_json.lines());
        for (r, (input_line, printed_line)) in zipped_lines.enumerate() {
            if input_line != printed_line {
                println!("Poorly formatted on line {}:", r);
                println!("{}", input_line);
                println!("versus:");
                println!("{}", printed_line);
                break;
            }
        }
    }
}
