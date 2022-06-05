#[derive(Debug, Clone, Copy)]
enum Token {
    Letter,
    OpenParen,
    CloseParen,
    Plus,
    Minus,
    Asterisk,
    Question,
    Colon,
    Semi,
}

#[derive(Debug, Clone, Copy)]
enum OpToken {
    Blank,
    Juxt,
    Id,
    Open,
    Apply,
    Close,
    Plus,
    Negative,
    Minus,
    Times,
    Question,
    Colon,
    Semi,
}

fn lex(stream: &str) -> Vec<(char, Token)> {
    fn char_to_token(ch: char) -> Token {
        match ch {
            'x' => Token::Letter,
            '(' => Token::OpenParen,
            ')' => Token::CloseParen,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Asterisk,
            '?' => Token::Question,
            ':' => Token::Colon,
            ';' => Token::Semi,
            _ => panic!("unrecognized char {}", ch),
        }
    }
    stream
        .chars()
        .map(|ch| (ch, char_to_token(ch)))
        .collect::<Vec<_>>()
}

fn resolve(stream: Vec<(char, Token)>) -> Vec<(char, OpToken)> {
    fn token_to_prefixy_op(tok: Token) -> Option<OpToken> {
        match tok {
            Token::Letter => Some(OpToken::Id),
            Token::OpenParen => Some(OpToken::Open),
            Token::CloseParen => None,
            Token::Plus => None,
            Token::Minus => Some(OpToken::Negative),
            Token::Asterisk => None,
            Token::Question => None,
            Token::Colon => None,
            Token::Semi => None,
        }
    }
    fn token_to_suffixy_op(tok: Token) -> Option<OpToken> {
        match tok {
            Token::Letter => None,
            Token::OpenParen => Some(OpToken::Apply),
            Token::CloseParen => Some(OpToken::Close),
            Token::Plus => Some(OpToken::Plus),
            Token::Minus => Some(OpToken::Minus),
            Token::Asterisk => Some(OpToken::Times),
            Token::Question => Some(OpToken::Question),
            Token::Colon => Some(OpToken::Colon),
            Token::Semi => Some(OpToken::Semi),
        }
    }
    fn has_right_arg(tok: OpToken) -> bool {
        match tok {
            OpToken::Blank => false,
            OpToken::Juxt => true,
            OpToken::Id => false,
            OpToken::Open => true,
            OpToken::Apply => true,
            OpToken::Close => false,
            OpToken::Plus => true,
            OpToken::Negative => true,
            OpToken::Minus => true,
            OpToken::Times => true,
            OpToken::Question => true,
            OpToken::Colon => true,
            OpToken::Semi => false,
        }
    }
    let mut out = vec![];
    let mut expr_mode = true;
    for (ch, tok) in stream {
        if expr_mode {
            if let Some(op) = token_to_prefixy_op(tok) {
                expr_mode = has_right_arg(op);
                out.push((ch, op));
            } else {
                let op = token_to_suffixy_op(tok).unwrap();
                expr_mode = has_right_arg(op);
                out.push(('_', OpToken::Blank));
                out.push((ch, op));
            }
        } else {
            if let Some(op) = token_to_suffixy_op(tok) {
                expr_mode = has_right_arg(op);
                out.push((ch, op));
            } else {
                let op = token_to_prefixy_op(tok).unwrap();
                expr_mode = has_right_arg(op);
                out.push(('.', OpToken::Juxt));
                out.push((ch, op));
            }
        }
    }
    if expr_mode {
        out.push(('_', OpToken::Blank));
    }
    println!("resolved: {:?}", out);
    out
}

fn shunt(stream: Vec<(char, OpToken)>) -> Vec<(char, OpToken)> {
    fn prec(tok: OpToken) -> (u32, u32) {
        match tok {
            OpToken::Blank => (0, 0),
            OpToken::Juxt => (10, 10),
            OpToken::Id => (0, 0),
            OpToken::Open => (0, 1000),
            OpToken::Apply => (40, 1000),
            OpToken::Close => (1000, 0),
            OpToken::Plus => (100, 99),
            OpToken::Negative => (0, 99),
            OpToken::Minus => (100, 99),
            OpToken::Times => (60, 61),
            OpToken::Question => (50, 1000),
            OpToken::Colon => (1000, 1000),
            OpToken::Semi => (1000, 0),
        }
    }

    let mut stack = vec![];
    let mut out = vec![];
    for (ch, tok) in stream {
        loop {
            let rprec = stack.last().map(|(_, t)| prec(*t).1).unwrap_or(u32::MAX);
            let lprec = prec(tok).0;
            if rprec >= lprec {
                stack.push((ch, tok));
                break;
            } else {
                while let Some((ch, tok)) = stack.pop() {
                    let lprec = prec(tok).0;
                    out.push((ch, tok));
                    let rprec = stack.last().map(|(_, t)| prec(*t).1).unwrap_or(u32::MAX);
                    if rprec > lprec {
                        break;
                    }
                }
            }
        }
    }
    while let Some(op) = stack.pop() {
        out.push(op);
    }
    out
}

fn show(tok: OpToken) -> char {
    use OpToken::*;

    match tok {
        Blank => '_',
        Juxt => '.',
        Id => 'x',
        Open => '(',
        Apply => '[',
        Close => ')',
        Plus => '+',
        Negative => '~',
        Minus => '-',
        Times => '*',
        Question => '?',
        Colon => ':',
        Semi => ';',
    }
}

fn parse(stream: &str) -> String {
    shunt(resolve(lex(stream)))
        .into_iter()
        .map(|(_, tok)| show(tok))
        .collect()
}

#[test]
fn test() {
    assert_eq!(parse("x"), "x");

    // Blank & Juxtapose
    assert_eq!(parse("xx"), "xx.");
    assert_eq!(parse("x+"), "x_+");
    assert_eq!(parse("+x"), "_x+");

    // Assoc
    assert_eq!(parse("x+x+x"), "xx+x+");
    assert_eq!(parse("x*x*x"), "xxx**");

    // Resolution
    assert_eq!(parse("x-x-x"), "xx-x-");
    assert_eq!(parse("-x--x"), "x~x~-");

    // Links
    assert_eq!(parse("(x)"), "x)(");
    assert_eq!(parse("(x+x)"), "xx+)(");
    assert_eq!(parse("x(x)"), "xx)[");
    assert_eq!(parse("x?x:x;"), "xxx;:?");
    assert_eq!(parse("x?x:x?x:x;;"), "xxxxx;:?;:?");
    assert_eq!(parse("((x))"), "x)()(");
    assert_eq!(parse("(x((x)))"), "xx)()[)(");
}
