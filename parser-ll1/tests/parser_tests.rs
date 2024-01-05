use parser_ll1::{choice, empty, tuple, Grammar, GrammarError, ParseError, Parser};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::mem;

/// Convert textual parser description to parser that produces s-expression string:
///
/// ```text
/// .         -- match nothing
/// foobar"   -- match string foobar
/// [0-9]*/   -- match regex [0-9]*
/// X ?       -- optional X
/// X *       -- many X
/// X $       -- X then EOF
/// X Y ,     -- X sep by Y
/// X Y &2    -- X then Y
/// X Y |2    -- X or Y
/// X Y Z &3  -- X then Y then Z
/// X Y Z |3  -- X or Y or Z
/// ```

fn make_parser(
    description: &str,
) -> Result<impl Fn(&str, &str) -> Result<String, ParseError>, GrammarError> {
    let mut grammar = Grammar::with_whitespace(" +")?;
    let mut stack: Vec<Box<dyn Parser<String>>> = Vec::new();
    let mut word = String::new();
    let mut chars = description.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            ' ' => continue,
            '.' => stack.push(Box::new(empty().map(|()| ".".to_owned()))),
            '/' => {
                let word = mem::take(&mut word);
                let parser = grammar
                    .regex(&word, &word)?
                    .span(|span| span.substr.to_owned());
                stack.push(Box::new(parser));
            }
            '"' => {
                let word = mem::take(&mut word);
                let parser = grammar.string(&word)?;
                let parser = parser.map(move |()| word.clone());
                stack.push(Box::new(parser));
            }
            '?' => {
                let parser = stack.pop().unwrap();
                let parser = parser.opt().map(|opt| match opt {
                    None => ".".to_owned(),
                    Some(s) => s,
                });
                stack.push(Box::new(parser));
            }
            '*' => {
                let parser = stack.pop().unwrap();
                let parser = parser.many0().map(|vec| format!("({})", vec.join(" ")));
                stack.push(Box::new(parser));
            }
            '$' => {
                let parser = stack.pop().unwrap();
                let parser = parser.complete();
                stack.push(Box::new(parser));
            }
            ',' => {
                let parser_2 = stack.pop().unwrap();
                let parser_1 = stack.pop().unwrap();
                let parser = parser_1
                    .many_sep0(parser_2)
                    .map(|vec| format!("({})", vec.join(" ")));
                stack.push(Box::new(parser));
            }
            '&' => match chars.peek() {
                Some('2') => {
                    chars.next();
                    let parser_2 = stack.pop().unwrap();
                    let parser_1 = stack.pop().unwrap();
                    let parser = tuple((parser_1, parser_2)).map(|(a, b)| format!("({} {})", a, b));
                    stack.push(Box::new(parser));
                }
                Some('3') => {
                    chars.next();
                    let parser_3 = stack.pop().unwrap();
                    let parser_2 = stack.pop().unwrap();
                    let parser_1 = stack.pop().unwrap();
                    let parser = tuple((parser_1, parser_2, parser_3))
                        .map(|(a, b, c)| format!("({} {} {})", a, b, c));
                    stack.push(Box::new(parser));
                }
                _ => panic!("Bad count after '&' in parser test case"),
            },
            '|' => match chars.peek() {
                Some('2') => {
                    chars.next();
                    let parser_2 = stack.pop().unwrap();
                    let parser_1 = stack.pop().unwrap();
                    let parser = choice("|", (parser_1, parser_2));
                    stack.push(Box::new(parser));
                }
                Some('3') => {
                    chars.next();
                    let parser_3 = stack.pop().unwrap();
                    let parser_2 = stack.pop().unwrap();
                    let parser_1 = stack.pop().unwrap();
                    let parser = choice("|", (parser_1, parser_2, parser_3));
                    stack.push(Box::new(parser));
                }
                _ => panic!("Bad count after '|' in parser test case"),
            },
            _ => word.push(ch),
        }
    }
    assert_eq!(stack.len(), 1, "Bad parser test case");
    let parser = stack.into_iter().next().unwrap();
    grammar.make_parse_fn(parser)
}

fn assert_parse(
    line_num: usize,
    parser_description: &str,
    input: &str,
    expected: Result<String, String>,
) {
    let mut expected = match expected {
        Ok(result) => format!("ok {}", result),
        Err(err) => format!("err {}", err),
    };
    // Compare only the first line
    expected = expected.lines().next().unwrap().to_owned();

    let mut actual = match make_parser(parser_description) {
        Ok(parse) => match parse("test case", input) {
            Ok(result) => format!("ok {}", result),
            Err(err) => format!("err {}", err),
        },
        Err(err) => format!("err {}", err),
    };
    // Compare only the first line
    actual = actual.lines().next().unwrap().to_owned();

    if actual != expected {
        panic!(
            "Parser test case failure, line {}:\nPARSER {}\nINPUT {}\nEXPECT {}\nACTUAL {}",
            line_num, parser_description, input, expected, actual
        );
    }
}

#[test]
fn test_parser() {
    let mut parser = String::new();
    let mut input = String::new();

    let file = File::open("tests/parser_test_cases.txt").unwrap();
    let reader = BufReader::new(file);
    let mut line_num = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        line_num += 1;
        if line == "" || line.starts_with("#") {
            continue;
        } else if let Some(parser_str) = line.strip_prefix("PARSER ") {
            parser = parser_str.trim().to_owned();
        } else if let Some(input_str) = line.strip_prefix("INPUT ") {
            input = input_str.trim().to_owned();
        } else if let Some(expect_str) = line.strip_prefix("EXPECT ") {
            let expect_str = expect_str.trim();
            if let Some(ok_str) = expect_str.strip_prefix("ok") {
                assert_parse(line_num, &parser, &input, Ok(ok_str.trim().to_owned()));
            } else if let Some(err_str) = expect_str.strip_prefix("err") {
                assert_parse(line_num, &parser, &input, Err(err_str.trim().to_owned()));
            } else {
                panic!("Bad test case input (expected `ok` or `err`): {}", line);
            }
        } else {
            panic!("Bad test case input: {}", line);
        }
    }
}
