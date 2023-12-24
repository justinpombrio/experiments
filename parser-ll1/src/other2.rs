use crate::initial_set::InitialSet;
use crate::lexer::{LexemeIter, LexerBuilder, Token};
use crate::vec_map::VecMap;
use std::iter::Peekable;

type TokenStream<'l, 's> = Peekable<LexemeIter<'l, 's>>;

struct ParseError;

pub struct GrammarError;

type ParseFn<T> = Box<dyn Fn(&mut TokenStream) -> Result<T, ParseError>>;

trait Parser<T> {
    fn copy(&self) -> Box<dyn Parser<T>>;

    fn compile(
        self,
        lexer_builder: &mut LexerBuilder,
    ) -> Result<(InitialSet, ParseFn<T>), GrammarError>;
}
