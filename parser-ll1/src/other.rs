use crate::lexer::{LexemeIter, Token};
use crate::vec_map::VecMap;
use std::iter::Peekable;

// COMBINATOR | ERROR      | PARSE
//  token T   | expected T | take T else None
//  A seq B   | A.error    | A ?N !E :B ?N !E :(A, B)
// l: A or B  | expected l | A ? Ok : B

type TokenStream<'l, 's> = Peekable<LexemeIter<'l, 's>>;

struct ParseError; // TODO: fill

trait Parser {
    type Output;

    fn initial_empty(&self) -> bool;

    fn initial_tokens(&self) -> VecMap<()>;

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError>;
}

struct Empty;

impl Parser for Empty {
    type Output = ();

    fn initial_empty(&self) -> bool {
        true
    }

    fn initial_tokens(&self) -> VecMap<()> {
        VecMap::new()
    }

    fn parse(&self, stream: &mut TokenStream) -> Result<(), ParseError> {
        Ok(())
    }
}

struct Seq<P0: Parser, P1: Parser>(P0, P1);

impl<P0: Parser, P1: Parser> Parser for Seq<P0, P1> {
    type Output = (P0::Output, P1::Output);

    fn initial_empty(&self) -> bool {
        self.0.initial_empty() && self.1.initial_empty()
    }

    fn initial_tokens(&self) -> VecMap<()> {
        let mut tokens = self.0.initial_tokens();
        if self.0.initial_empty() {
            tokens.extend(self.1.initial_tokens());
        }
        tokens
    }

    fn parse(&self, stream: &mut TokenStream) -> Result<(P0::Output, P1::Output), ParseError> {
        let result_0 = self.0.parse(stream)?;
        let result_1 = self.1.parse(stream)?;
        Ok((result_0, result_1))
    }
}

/*
struct Choice<P0: Parser, P1: Parser<Output = P0::Output>> {
    name: String,
    parsers: (P0, P1),
    initial_sets: (VecMap<()>, VecMap<()>),
}

impl<P0: Parser, P1: Parser<Output = P0::Output>> Parser for Choice<P0, P1> {
    type Output = P0::Output;

    fn initial_empty(&self) -> bool {
        self.parsers.0.initial_empty() || self.parsers.1.initial_empty()
    }

    fn initial_tokens(&self) -> VecMap<()> {
        let mut tokens = self.parsers.0.initial_tokens();
        tokens.extend(self.parsers.1.initial_tokens());
        tokens
    }

    fn parse(&self, stream: &mut TokenStream) -> Result<P0::Output, ParseError> {
        let result_0 = self.0.parse(stream)?;
        let result_1 = self.1.parse(stream)?;
        Ok((result_0, result_1))
    }
}
*/
