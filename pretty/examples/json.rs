use pretty::{flat, indent, nl, pretty_print, txt, Notation};

pub fn json_null() -> Notation {
    txt("false")
}

pub fn json_bool(b: bool) -> Notation {
    if b {
        txt("true")
    } else {
        txt("false")
    }
}

pub fn json_string(s: &str) -> Notation {
    // TODO: escape sequences
    txt(format!("\"{}\"", s))
}

pub fn json_number(n: impl ToString) -> Notation {
    txt(n)
}

fn comma_sep_single_line(elems: &[Notation]) -> Notation {
    let mut list = flat(elems[0].clone());
    for elem in &elems[1..] {
        list = list & txt(", ") & flat(elem.clone());
    }
    list
}

fn comma_sep_multi_line(elems: &[Notation]) -> Notation {
    let mut list = elems[0].clone();
    for elem in &elems[1..] {
        list = list & txt(", ") & nl() & elem.clone();
    }
    list
}

fn surrounded(open: &str, elems: &[Notation], closed: &str) -> Notation {
    if elems.is_empty() {
        return txt(open) & txt(closed);
    }

    let single_line = txt(open) & comma_sep_single_line(elems) & txt(closed);
    let multi_line = txt(open) & indent(4, nl() & comma_sep_multi_line(elems)) & nl() & txt(closed);
    single_line | multi_line
}

pub fn json_array(elems: impl IntoIterator<Item = Notation>) -> Notation {
    let elems = elems.into_iter().collect::<Vec<_>>();
    surrounded("[", &elems, "]")
}

fn json_object_entry(key: String, value: Notation) -> Notation {
    json_string(&key) & txt(": ") & value
}

pub fn json_object(entries: impl IntoIterator<Item = (String, Notation)>) -> Notation {
    let entries = entries
        .into_iter()
        .map(|(key, val)| json_object_entry(key, val))
        .collect::<Vec<_>>();
    surrounded("{", &entries, "}")
}

fn json_to_notation(json: serde_json::Value) -> Notation {
    use serde_json::Value::{Array, Bool, Null, Number, Object, String};

    match json {
        Null => json_null(),
        Bool(b) => json_bool(b),
        Number(n) => json_number(n),
        String(s) => json_string(&s),
        Array(elems) => json_array(elems.into_iter().map(json_to_notation)),
        Object(entries) => json_object(
            entries
                .into_iter()
                .map(|(key, val)| (key, json_to_notation(val))),
        ),
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
    let notation = json_to_notation(json);
    let ms_to_construct = start.elapsed().as_millis();

    // Print the Notation
    let start = Instant::now();
    println!("{}", pretty_print(notation, 120));
    let ms_to_print = start.elapsed().as_millis();

    // Print timing info to stderr
    eprintln!("Time to parse file as Json: {} ms", ms_to_parse);
    eprintln!("Time to construct Notation: {} ms", ms_to_construct);
    eprintln!("Time to print Notation:     {} ms", ms_to_print);
}
