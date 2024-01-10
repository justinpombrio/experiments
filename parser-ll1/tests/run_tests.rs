use parser_ll1::{choice, empty, tuple, CompiledParser, Grammar, GrammarError, Parser};
use std::fmt;

type LineNum = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    Ok,
    Err,
}

#[derive(Debug, Clone)]
enum Header {
    Parser,
    Input,
    Expect(Status, LineNum),
}

#[derive(Debug, Clone)]
struct Section {
    header: Header,
    contents: String,
}

#[derive(Debug, Clone)]
struct TestCases {
    sections: Vec<Section>,
}

impl TestCases {
    fn num_tests(&self) -> usize {
        let mut count = 0;
        for section in &self.sections {
            if matches!(section.header, Header::Expect(_, _)) {
                count += 1;
            }
        }
        count
    }
}

fn make_test_case_parser() -> Result<impl CompiledParser<TestCases>, GrammarError> {
    let mut g = Grammar::with_whitespace(r#"([ \t\n]|#[^\n]*\n)+"#).unwrap();

    let status_p = choice(
        "Status",
        (
            g.string("Ok")?.constant(Status::Ok),
            g.string("Err")?.constant(Status::Err),
        ),
    );
    let header_p = choice(
        "Header",
        (
            g.string("Parser")?.constant(Header::Parser),
            g.string("Input")?.constant(Header::Input),
            status_p
                .preceded(g.string("Expect")?)
                .map_span(|span, status| Header::Expect(status, span.start.line)),
        ),
    );
    let line_p = g.regex("Line", r#">( [^\n]*)?"#)?.span(|span| {
        if span.substr.len() >= 2 {
            format!("{}", &span.substr[2..])
        } else {
            String::new()
        }
    });
    let contents_p = line_p.many1().map(|lines| lines.join("\n"));
    let section_p = tuple("Section", (header_p, contents_p))
        .map(|(header, contents)| Section { header, contents });
    let sections_p = section_p.many0().map(|sections| TestCases { sections });

    g.compile_parser(sections_p)
}

#[derive(Debug, Clone)]
struct CustomError(String);

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for CustomError {}

fn parse_parser(parser_description: &str) -> Result<impl CompiledParser<String>, GrammarError> {
    type SExprParser = Box<dyn Parser<String>>;

    let mut grammar = Grammar::new();
    let mut stack: Vec<SExprParser> = Vec::new();

    for word in parser_description.split_whitespace() {
        if word.ends_with('/') {
            let mut parts = word.split('/');
            let name = parts.next().unwrap();
            let regex = parts.next().unwrap();
            assert_eq!(parts.next(), Some(""));
            assert_eq!(parts.next(), None);
            let parser = grammar
                .regex(name, regex)?
                .span(|span| span.substr.to_owned());
            stack.push(Box::new(parser));
            continue;
        } else if word.ends_with('"') {
            assert_eq!(&word[0..1], "\"");
            let string = &word[1..word.len() - 1];
            let parser = grammar.string(string)?.constant(string.to_owned());
            stack.push(Box::new(parser));
            continue;
        }
        match word {
            "empty" => {
                let parser = empty().constant("()".to_owned());
                stack.push(Box::new(parser));
            }

            // Mapping
            "constant" => {
                let parser_1 = stack.pop().unwrap();
                let parser = parser_1.constant("constant".to_owned());
                stack.push(Box::new(parser));
            }
            "map" => {
                let parser_1 = stack.pop().unwrap();
                let parser = parser_1.map(|s| format!("(map {})", s));
                stack.push(Box::new(parser));
            }
            "try_map" => {
                let parser_1 = stack.pop().unwrap();
                let parser = parser_1.try_map(|s| {
                    if s.contains("ok") {
                        Ok(format!("(ok {})", s))
                    } else {
                        Err(CustomError(format!("oops something went wrong: {}", s)))
                    }
                });
                stack.push(Box::new(parser));
            }
            "span" => {
                let parser_1 = stack.pop().unwrap();
                let parser = parser_1
                    .span(|span| format!("(span {} {}-{})", span.substr, span.start, span.end));
                stack.push(Box::new(parser));
            }
            "try_span" => {
                let parser_1 = stack.pop().unwrap();
                let parser = parser_1.try_span(|span| {
                    if span.substr.contains("ok") {
                        Ok(format!("(ok {} {}-{})", span.substr, span.start, span.end))
                    } else {
                        Err(CustomError(format!(
                            "oops something went wrong: {} {}-{}",
                            span.substr, span.start, span.end
                        )))
                    }
                });
                stack.push(Box::new(parser));
            }
            "map_span" => {
                let parser_1 = stack.pop().unwrap();
                let parser = parser_1.map_span(|span, s| {
                    format!(
                        "(map_span {} {}-{} {})",
                        span.substr, span.start, span.end, s
                    )
                });
                stack.push(Box::new(parser));
            }
            "try_map_span" => {
                let parser_1 = stack.pop().unwrap();
                let parser = parser_1.try_map_span(|span, s| {
                    if span.substr.contains("ok") {
                        Ok(format!(
                            "(ok {} {}-{} {})",
                            span.substr, span.start, span.end, s
                        ))
                    } else {
                        Err(CustomError(format!(
                            "oops something went wrong: {} {}-{} {}",
                            span.substr, span.start, span.end, s
                        )))
                    }
                });
                stack.push(Box::new(parser));
            }
            "opt" => {
                let parser_1 = stack.pop().unwrap();
                let parser = parser_1.opt().map(|opt| match opt {
                    None => ".".to_owned(),
                    Some(s) => s,
                });
                stack.push(Box::new(parser));
            }
            _ => panic!("Bad test case parser description: {} not recognized", word),
        }
    }
    assert_eq!(stack.len(), 1, "Bad parser test case");
    let parser = stack.into_iter().next().unwrap();
    grammar.compile_parser(parser)
}

fn run_test_case(
    filename: &str,
    line_num: LineNum,
    parser_description: &str,
    input: &str,
    expected: (Status, String),
) {
    colored::control::set_override(false);

    let actual = match parse_parser(parser_description) {
        Ok(parser) => match parser.parse(filename, input) {
            Ok(succ) => (Status::Ok, succ),
            Err(err) => (Status::Err, format!("{}", err)),
        },
        Err(err) => (Status::Err, format!("{}", err)),
    };

    if actual != expected {
        println!("Parser");
        for line in parser_description.lines() {
            println!("> {}", line);
        }
        println!("Input");
        for line in input.lines() {
            println!("> {}", line);
        }
        if input == "" {
            println!(">");
        }
        println!("Expected {}", expected.0);
        for line in expected.1.lines() {
            println!("> {}", line);
        }
        println!("Actual {}", expected.0);
        for line in actual.1.lines() {
            println!("> {}", line);
        }
        panic!("Test case failure at {}, line {}.", filename, line_num + 1);
    }
}

fn run_test_cases(filename: &str, test_cases: TestCases) {
    let mut parser = String::new();
    let mut input = String::new();
    for section in test_cases.sections {
        match section.header {
            Header::Parser => parser = section.contents,
            Header::Input => input = section.contents,
            Header::Expect(status, line_num) => {
                let expected = (status, section.contents);
                run_test_case(filename, line_num, &parser, &input, expected);
            }
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::Ok => write!(f, "Ok"),
            Status::Err => write!(f, "Err"),
        }
    }
}

#[test]
fn run_parser_tests() {
    use std::fs;

    let test_case_parser = match make_test_case_parser() {
        Ok(parser) => parser,
        Err(err) => panic!("{}", err),
    };

    for entry in fs::read_dir("tests/").unwrap() {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        if file_type.is_file() && file_name.ends_with(".tests.txt") {
            let file_contents = fs::read_to_string(entry.path()).unwrap();
            let test_cases = match test_case_parser.parse(&file_name, &file_contents) {
                Ok(test_cases) => test_cases,
                Err(err) => panic!("{}", err),
            };
            let num_tests = test_cases.num_tests();
            run_test_cases(&file_name, test_cases);
            println!("Ran {} successful test cases from {}", num_tests, file_name);
        }
    }
}
