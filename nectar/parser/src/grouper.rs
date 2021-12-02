use std::collections::HashMap;

use source::{SourceFile, Span};
use cache::{CacheEntry, CacheId};
use error::*;
use item::{Lexeme, GroupItem};
use grammar;
use grammar::{Operator, Grammar};
use lexer::{Lexer};

// TODO:
// * terminal or not
// * Trie instead of List


/****************************** PUBLIC ******************************/


// Grouping //

pub fn group<'s, 'g>(src: &'s SourceFile, input: Lexer<'s, 'g>, grammar: &'g Grammar)
                     -> Result<GroupItem<'s, 'g>, ParseError<'s, 'g>> {
    let mut grouper = Grouper::new(grammar);
    try!(grouper.start(Lexeme::lexeme_start(src, grammar)));
    for lexeme in input {
        let lexeme = try!(lexeme);
        try!(grouper.consume(lexeme));
    }
    try!(grouper.end(Lexeme::lexeme_end(src, grammar)));
    Ok(grouper.pop().0.close())
}

pub type Token = CacheEntry<grammar::Token>;


/****************************** PRIVATE ******************************/


type Group<'s, 'g> = Vec<GroupItem<'s, 'g>>;

struct Multi<'s, 'g> {
    op:         &'g Operator,
    span:       Span<'s>, // Total span. Must keep up to date.
    groups:     Vec<(Span<'s>, Group<'s, 'g>)>,
    expected:   Vec<&'g Token>
}

impl<'s, 'g> Multi<'s, 'g> {
    fn new(span: Span<'s>, op: &'g Operator, mut parts: Vec<&'g Token>) -> Multi<'s, 'g> {
        parts.reverse();
        Multi{
            span:       span,
            op:         op,
            groups:     vec!(),
            expected:   parts
        }
    }

    fn expectation(&self) -> Option<&'g Token> {
        match self.expected.last() {
            None => None,
            Some(token) => Some(token)
        }
    }

    fn next_part(&mut self, group: Group<'s, 'g>, span: Span<'s>) {
        self.span = self.span + span;
        self.groups.push((span, group));
        self.expected.pop();
    }

    fn complete(&self) -> bool {
        self.expectation().is_none()
    }

    fn close(self) -> GroupItem<'s, 'g> {
        GroupItem::Multeme(self.span, self.op, self.groups)
    }
}

pub struct Grouper<'s, 'g> {
    op_table: HashMap<CacheId<grammar::Token>, (&'g Operator, Vec<&'g Token>)>,
    stack: Vec<(Multi<'s, 'g>, Group<'s, 'g>)>
}

impl<'s, 'g> Grouper<'s, 'g> {
    fn new(grammar: &'g Grammar) -> Grouper<'s, 'g> {
        Grouper{
            op_table: grammar.op_table(),
            stack:    vec!()
        }
    }

    fn pop(&mut self) -> (Multi<'s, 'g>, Group<'s, 'g>) {
        match self.stack.pop() {
            None => panic!("Grouper::pop - Unexpectedly empty stack"),
            Some((multi, group)) => (multi, group)
        }
    }

    fn push_item(&mut self, item: GroupItem<'s, 'g>) {
        match self.stack.last_mut() {
            None => panic!("Grouper: Unexpected empty stack"),
            Some(&mut (_, ref mut group)) => group.push(item)
        }
    }

    fn expectation(&self) -> &'g Token {
        for &(ref multi, _) in self.stack.iter().rev() {
            if let Some(token) = multi.expectation() {
                return token
            }
        }
        panic!("Grouper::expectation - Unexpected lack of expectation")
    }

    fn initial(&self, lex: Lexeme<'s, 'g>) -> Option<(&'g Operator, Vec<&'g Token>)> {
        match self.op_table.get(&lex.token.index) {
            None => None,
            Some(&(ref op, ref parts)) => Some((op, parts.clone()))
        }
    }

    fn push_multi(&mut self, multi: Multi<'s, 'g>) {
        if multi.complete() && !self.stack.is_empty() {
            let item = multi.close();
            self.push_item(item);
        } else {
            self.stack.push((multi, vec!()));
        }
    }

    fn begin_op(&mut self, lex: Lexeme<'s, 'g>, op: &'g Operator, parts: Vec<&'g Token>) {
        self.push_multi(Multi::new(lex.span, op, parts));
    }
    
    fn continue_op(&mut self, lex: Lexeme<'s, 'g>) {
        let (mut multi, group) = self.pop();
        multi.next_part(group, lex.span);
        self.push_multi(multi);
    }

    fn start(&mut self, lex: Lexeme<'s, 'g>) -> Result<(), ParseError<'s, 'g>> {
        let (op, parts) = self.initial(lex)
            .expect("Grouper::start - Could not find START lexeme");
        self.begin_op(lex, op, parts);
        Ok(())
    }

    fn end(&mut self, lex: Lexeme<'s, 'g>) -> Result<(), ParseError<'s, 'g>> {
        self.consume(lex)
    }

    fn consume(&mut self, lex: Lexeme<'s, 'g>) -> Result<(), ParseError<'s, 'g>> {
        if !lex.token.is_part() {
            // This lexeme not participating in grouping. Push it and carry on.
            self.push_item(GroupItem::from(lex));
            return Ok(())
        }
        // It's an operator part.
        let expectation = self.expectation();
        if lex.token == expectation {
            // An expected next part of an operator. Advance the operator group.
            self.continue_op(lex);
            Ok(())
        } else if let Some((op, parts)) = self.initial(lex) {
            // This lexeme starts a new operator.
            self.begin_op(lex, op, parts);
            Ok(())
        } else {
            // Neither expected by an operator, nor can start an operator.
            // This lexeme does not belong here.
            Err(wrong_part_error(lex, expectation))
        }
    }
}


/****************************** TESTS ******************************/

#[cfg(test)]
mod test {

    use grammar::*;
    use lexer::lex;
    use grouper::group;
    use source::SourceFile;

    #[test]
    fn test_grouping() {
        let tokens = vec!(
            identifier("Id", "^[a-z][_a-zA-Z0-9]*"),
            identifier("Var", "^\\$[_a-zA-Z0-9-]+"),
            identifier("TypeId", "^[A-Z][_a-zA-Z0-9]*"),
            constant("True", "true"),
            constant("False", "false"),
            literal("Num", "^[0-9]+"),
            whitespace("Space", "^[ \t\n]+"),
            literal("Str", "^\"[^\"]*\""));

        let ops = vec!(
            op("Plus", vec!(punctuation("+"))),
            op("DoublePlus", vec!(punctuation("++"))),
            op("Comma", vec!(punctuation(","))),
            op("Begin", vec!(keyword("begin"), keyword("end"))),
            op("If", vec!(keyword("if"), keyword("then"), keyword("else"))));

        let grammar = make_grammar("TestLanguage", tokens, ops);

        fn test_grouping(source_code: &str, expected_output: &str, grammar: &Grammar) {
            let source: SourceFile = SourceFile::new(
                "TEST_FILE".to_string(),
                source_code.to_string());
            let lexed = lex(&source, grammar.token_table());
            let result = match group(&source, lexed, &grammar) {
                Err(err) => panic!("Testing: Failed to group!\n{}", err),
                Ok(result) => result
            };
            let found_output = &format!("{}", result);
            println!("\nSource:\n    {}", source_code);
            println!("Expected:\n    {}", expected_output);
            println!("Found:\n    {}", found_output);
            assert_eq!(expected_output, found_output);
        }

        let test = |source, output| {
            test_grouping(source, output, &grammar)
        };

        test("", "(Program ())");
        test("if then else",
             "(Program ((If () ())))");
        test("A B C",
             "(Program (A B C))");
        test("A if B C then D if E then F else G else H",
             "(Program (A (If (B C) (D (If (E) (F)) G)) H))");
        test("begin begin end end",
             "(Program ((Begin ((Begin ())))))");
        test("begin end begin end",
             "(Program ((Begin ()) (Begin ())))");
        test("+",
             "(Program ((Plus)))");
        test("A ++ B",
             "(Program (A (DoublePlus) B))");
        test("begin if A ++ B then begin C,D end else if E then F else begin G+H end end",
             "(Program ((Begin ((If (A (DoublePlus) B) ((Begin (C (Comma) D)))) (If (E) (F)) (Begin (G (Plus) H))))))");
    }
}
