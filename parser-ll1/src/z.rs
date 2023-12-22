// TODO: temporary
#![allow(unused)]

mod lexer;

pub enum GrammarError {
    Dummy,
}

pub enum ParseError {
    Dummy,
}

pub enum Pattern {
    String(String),
}

pub trait Parser {
    type Output;

    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Output), ParseError>;

    fn initial(&self) -> (bool, Vec<Pattern>);

    fn map<B>(self, func: impl Fn(Self::Output) -> B) -> impl Parser<Output = B>
    where
        Self: Sized,
    {
        Map { parser: self, func }
    }

    fn seq<P: Parser>(
        self,
        parser: P,
    ) -> Result<impl Parser<Output = (Self::Output, P::Output)>, GrammarError>
    where
        Self: Sized,
    {
        Ok(Seq {
            parser_1: self,
            parser_2: parser,
        })
    }
}

struct Produce<A: Clone>(A);

impl<A: Clone> Parser for Produce<A> {
    type Output = A;

    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, A), ParseError> {
        Ok((input, self.0.clone()))
    }

    fn initial(&self) -> (bool, Vec<Pattern>) {
        (true, Vec::new())
    }
}

struct Map<O, P: Parser, F: Fn(P::Output) -> O> {
    parser: P,
    func: F,
}

impl<O, P: Parser, F: Fn(P::Output) -> O> Parser for Map<O, P, F> {
    type Output = O;

    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, O), ParseError> {
        let (output, result) = self.parser.parse(input)?;
        Ok((output, (self.func)(result)))
    }

    fn initial(&self) -> (bool, Vec<Pattern>) {
        self.parser.initial()
    }
}

struct Seq<P: Parser, Q: Parser> {
    parser_1: P,
    parser_2: Q,
}

impl<P: Parser, Q: Parser> Parser for Seq<P, Q> {
    type Output = (P::Output, Q::Output);

    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, (P::Output, Q::Output)), ParseError> {
        let (output_1, result_1) = self.parser_1.parse(input)?;
        let (output_2, result_2) = self.parser_2.parse(output_1)?;
        Ok((output_2, (result_1, result_2)))
    }

    fn initial(&self) -> (bool, Vec<Pattern>) {
        panic!("NYI")
    }
}

/*
struct Choice<A, P: Parser<A>, Q: Parser<A>> {
    parser_1: P,
    parser_2: Q,
    phantom: PhantomData<A>,
}
*/
