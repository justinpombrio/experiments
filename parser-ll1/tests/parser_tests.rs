use parser_ll1::{choice, empty, seq2, seq3, seq4, Grammar, GrammarError, ParseError, Parser};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::mem;

fn make_parser(
    description: &str,
) -> Result<impl Fn(&str) -> Result<String, ParseError>, GrammarError> {
    let mut grammar = Grammar::new(" +")?;
    let mut stack: Vec<Parser<String>> = Vec::new();
    let mut word = String::new();
    let mut chars = description.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            ' ' => continue,
            '.' => stack.push(empty().map(|()| ".".to_owned())),
            '/' => {
                let word = mem::take(&mut word);
                let parser = grammar
                    .regex("regex", &word)?
                    .span(|span| span.substr.to_owned());
                stack.push(parser);
            }
            '"' => {
                let word = mem::take(&mut word);
                let parser = grammar.string(&word)?;
                let parser = parser.map(move |()| word.clone());
                stack.push(parser);
            }
            '?' => {
                let parser = stack.pop().unwrap();
                let parser = parser.opt()?.map(|opt| match opt {
                    None => ".".to_owned(),
                    Some(s) => s,
                });
                stack.push(parser);
            }
            '*' => {
                let parser = stack.pop().unwrap();
                let parser = parser.many()?.map(|vec| format!("({})", vec.join(" ")));
                stack.push(parser);
            }
            '$' => {
                let parser = stack.pop().unwrap();
                let parser = parser.complete();
                stack.push(parser);
            }
            ',' => {
                let parser_2 = stack.pop().unwrap();
                let parser_1 = stack.pop().unwrap();
                let parser = parser_1
                    .sep(parser_2)?
                    .map(|vec| format!("({})", vec.join(" ")));
                stack.push(parser);
            }
            '&' => match chars.peek() {
                Some('2') => {
                    chars.next();
                    let parser_2 = stack.pop().unwrap();
                    let parser_1 = stack.pop().unwrap();
                    let parser = seq2(parser_1, parser_2)?.map(|(a, b)| format!("({} {})", a, b));
                    stack.push(parser);
                }
                Some('3') => {
                    chars.next();
                    let parser_3 = stack.pop().unwrap();
                    let parser_2 = stack.pop().unwrap();
                    let parser_1 = stack.pop().unwrap();
                    let parser = seq3(parser_1, parser_2, parser_3)?
                        .map(|(a, b, c)| format!("({} {} {})", a, b, c));
                    stack.push(parser);
                }
                Some('4') => {
                    chars.next();
                    let parser_4 = stack.pop().unwrap();
                    let parser_3 = stack.pop().unwrap();
                    let parser_2 = stack.pop().unwrap();
                    let parser_1 = stack.pop().unwrap();
                    let parser = seq4(parser_1, parser_2, parser_3, parser_4)?
                        .map(|(a, b, c, d)| format!("({} {} {} {})", a, b, c, d));
                    stack.push(parser);
                }
                _ => panic!("Bad count after '&' in parser test case"),
            },
            '|' => match chars.peek() {
                Some('2') => {
                    chars.next();
                    let parser_2 = stack.pop().unwrap();
                    let parser_1 = stack.pop().unwrap();
                    let parser = choice("|", [parser_1, parser_2])?;
                    stack.push(parser);
                }
                Some('3') => {
                    chars.next();
                    let parser_3 = stack.pop().unwrap();
                    let parser_2 = stack.pop().unwrap();
                    let parser_1 = stack.pop().unwrap();
                    let parser = choice("|", [parser_1, parser_2, parser_3])?;
                    stack.push(parser);
                }
                Some('4') => {
                    chars.next();
                    let parser_4 = stack.pop().unwrap();
                    let parser_3 = stack.pop().unwrap();
                    let parser_2 = stack.pop().unwrap();
                    let parser_1 = stack.pop().unwrap();
                    let parser = choice("|", [parser_1, parser_2, parser_3, parser_4])?;
                    stack.push(parser);
                }
                _ => panic!("Bad count after '|' in parser test case"),
            },
            _ => word.push(ch),
        }
    }
    assert_eq!(stack.len(), 1, "Bad parser test case");
    let parser = stack.into_iter().next().unwrap();
    Ok(grammar.make_parse_fn(parser))
}

fn assert_parse(parser_description: &str, input: &str, expected: Result<String, String>) {
    let parse = match make_parser(parser_description) {
        Ok(parser) => parser,
        Err(err) => {
            assert_eq!(Err(err.to_string()), expected);
            return;
        }
    };
    assert_eq!(parse(input).map_err(|e| format!("{}", e)), expected);
}

#[test]
fn test_parser() {
    let mut parser = String::new();
    let mut input = String::new();

    let file = File::open("tests/parser_test_cases.txt").unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        if line == "" || line.starts_with("#") {
            continue;
        } else if let Some(parser_str) = line.strip_prefix("PARSER ") {
            parser = parser_str.trim().to_owned();
        } else if let Some(input_str) = line.strip_prefix("INPUT ") {
            input = input_str.trim().to_owned();
        } else if let Some(expect_str) = line.strip_prefix("EXPECT ") {
            let expect_str = expect_str.trim();
            if let Some(ok_str) = expect_str.strip_prefix("ok ") {
                assert_parse(&parser, &input, Ok(ok_str.trim().to_owned()));
            } else if let Some(err_str) = expect_str.strip_prefix("err ") {
                assert_parse(&parser, &input, Err(err_str.trim().to_owned()));
            } else {
                panic!("Bad test case input (expected `ok` or `err`): {}", line);
            }
        } else {
            panic!("Bad test case input: {}", line);
        }
    }
}
