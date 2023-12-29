use crate::lexer::LexemeIter;
use crate::{GrammarError, Parse, ParseError, ParseFn, Parser};
use dyn_clone::clone_box;

/*========================================*/
/*          Parser: Seq3                  */
/*========================================*/

struct Seq3P<T0, T1, T2>(ParseFn<T0>, ParseFn<T1>, ParseFn<T2>);

impl<T0: Clone, T1: Clone, T2: Clone> Clone for Seq3P<T0, T1, T2> {
    fn clone(&self) -> Self {
        Seq3P(
            clone_box(self.0.as_ref()),
            clone_box(self.1.as_ref()),
            clone_box(self.2.as_ref()),
        )
    }
}

impl<T0: Clone, T1: Clone, T2: Clone> Parse<(T0, T1, T2)> for Seq3P<T0, T1, T2> {
    fn parse(&self, stream: &mut LexemeIter) -> Result<(T0, T1, T2), ParseError> {
        let result_0 = self.0.parse(stream)?;
        let result_1 = self.1.parse(stream)?;
        let result_2 = self.2.parse(stream)?;
        Ok((result_0, result_1, result_2))
    }
}

pub fn seq3<T0: Clone + 'static, T1: Clone + 'static, T2: Clone + 'static>(
    parser_0: Parser<T0>,
    parser_1: Parser<T1>,
    parser_2: Parser<T2>,
) -> Result<Parser<(T0, T1, T2)>, GrammarError> {
    let mut initial_set = parser_0.initial_set;
    initial_set.seq(parser_1.initial_set)?;
    initial_set.seq(parser_2.initial_set)?;
    Ok(Parser {
        initial_set,
        parse_fn: Box::new(Seq3P(
            parser_0.parse_fn,
            parser_1.parse_fn,
            parser_2.parse_fn,
        )),
    })
}

/*========================================*/
/*          Parser: Seq4                  */
/*========================================*/

struct Seq4P<T0, T1, T2, T3>(ParseFn<T0>, ParseFn<T1>, ParseFn<T2>, ParseFn<T3>);

impl<T0: Clone, T1: Clone, T2: Clone, T3: Clone> Clone for Seq4P<T0, T1, T2, T3> {
    fn clone(&self) -> Self {
        Seq4P(
            clone_box(self.0.as_ref()),
            clone_box(self.1.as_ref()),
            clone_box(self.2.as_ref()),
            clone_box(self.3.as_ref()),
        )
    }
}

impl<T0: Clone, T1: Clone, T2: Clone, T3: Clone> Parse<(T0, T1, T2, T3)> for Seq4P<T0, T1, T2, T3> {
    fn parse(&self, stream: &mut LexemeIter) -> Result<(T0, T1, T2, T3), ParseError> {
        let result_0 = self.0.parse(stream)?;
        let result_1 = self.1.parse(stream)?;
        let result_2 = self.2.parse(stream)?;
        let result_3 = self.3.parse(stream)?;
        Ok((result_0, result_1, result_2, result_3))
    }
}

pub fn seq4<T0: Clone + 'static, T1: Clone + 'static, T2: Clone + 'static, T3: Clone + 'static>(
    parser_0: Parser<T0>,
    parser_1: Parser<T1>,
    parser_2: Parser<T2>,
    parser_3: Parser<T3>,
) -> Result<Parser<(T0, T1, T2, T3)>, GrammarError> {
    let mut initial_set = parser_0.initial_set;
    initial_set.seq(parser_1.initial_set)?;
    initial_set.seq(parser_2.initial_set)?;
    initial_set.seq(parser_3.initial_set)?;
    Ok(Parser {
        initial_set,
        parse_fn: Box::new(Seq4P(
            parser_0.parse_fn,
            parser_1.parse_fn,
            parser_2.parse_fn,
            parser_3.parse_fn,
        )),
    })
}
