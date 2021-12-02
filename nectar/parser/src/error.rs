use std::fmt;

use source::Span;
use item::Lexeme;
use token::Token;
use operator::Operator;

use self::ParseErrorInfo::*;



pub struct ParseError<'s, 'g> {
    span: Span<'s>,
    error: ParseErrorInfo<'s, 'g>
}
impl<'s, 'g> fmt::Display for ParseError<'s, 'g> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\nAt {}.", self.error, self.span.preview(20)) 
   }
}

pub fn lexing_error<'s, 'g>(span: Span<'s>) -> ParseError<'s, 'g> {
    ParseError{
        span: span,
        error: LexingError
    }
}

pub fn wrong_part_error<'s, 'g>(lex: Lexeme<'s, 'g>, token: &'g Token) -> ParseError<'s, 'g> {
    ParseError{
        span: lex.span,
        error: WrongPartError(lex, token)
    }
}

pub fn ambiguous_assoc_error<'s, 'g>(left_lex: Lexeme<'s, 'g>, right_lex: Lexeme<'s, 'g>)
                                     -> ParseError<'s, 'g> {
    ParseError{
        span: left_lex.span + right_lex.span,
        error: AmbiguousAssocError(left_lex, right_lex)
    }
}



enum ParseErrorInfo<'s, 'g> {
    LexingError,
    WrongPartError(Lexeme<'s, 'g>, &'g Token),
    AmbiguousAssocError(Lexeme<'s, 'g>, Lexeme<'s, 'g>)
}

impl<'s, 'g> fmt::Display for ParseErrorInfo<'s, 'g> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &LexingError =>
                write!(f, "Could not recognize the next token."),
            &WrongPartError(ref lex, ref tok) =>
                write!(f, "Was expecting to find `{}`, but found `{}` instead.",
                       tok, lex),
            &AmbiguousAssocError(ref left_lex, ref right_lex) =>
                write!(f, "Couldn't resolve the precedence between `{}` and `{}`.",
                       left_lex, right_lex)
        }
    }
}
