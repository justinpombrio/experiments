use std::fmt;

use util::{display_sep};
use source::{SourceFile, Span, HasSpan, Spanned};
use catalog::Indexed;
use operator::Operator;
use token::Token;
use grammar::Grammar;


// Lexeme //

pub type Lexeme<'s, 'g> = Spanned<'s, Indexed<'g, Token>>;

/*
pub enum GroupItem<'s, 'g> {
    Lexeme(Span<'s>, &'g CacheEntry<Token>),
    Multeme(Span<'s>, &'g Operator, Vec<(Span<'s>, Vec<GroupItem<'s, 'g>>)>)
}

pub enum LaxItem<'s, 'g> {
    Lexeme(Span<'s>, &'g CacheEntry<Token>),
    Multeme(Span<'s>, &'g Operator, Vec<(Span<'s>, Vec<LaxItem<'s, 'g>>)>),
    Empty(Span<'s>),
    Juxt(Span<'s>)
}
*/

/*
impl<'s, 'g> From<Lexeme<'s, 'g>> for GroupItem<'s, 'g> {
    fn from(lex: Lexeme<'s, 'g>) -> GroupItem<'s, 'g> {
        GroupItem::Lexeme(lex.span, lex.token)
    }
}

impl<'s, 'g> HasSpan<'s> for GroupItem<'s, 'g> {
    fn span(&self) -> Span<'s> {
        match self {
            &GroupItem::Lexeme(span, _)     => span,
            &GroupItem::Multeme(span, _, _) => span
        }
    }
}

impl<'s, 'g> HasSpan<'s> for LaxItem<'s, 'g> {
    fn span(&self) -> Span<'s> {
        match self {
            &LaxItem::Lexeme(span, _)     => span,
            &LaxItem::Multeme(span, _, _) => span,
            &LaxItem::Empty(span)         => span,
            &LaxItem::Juxt(span)          => span
        }
    }
}
*/

/*
impl<'s, 'g> fmt::Display for GroupItem<'s, 'g> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &GroupItem::Lexeme(ref span, _) => write!(f, "{}", span),
            &GroupItem::Multeme(_, ref op, ref groups) => {
                if groups.is_empty() {
                    return write!(f, "({})", op);
                }
                try!(write!(f, "({} ", op));
                for (i, &(_, ref group)) in groups.iter().enumerate() {
                    try!(write!(f, "("));
                    try!(display_sep(f, " ", group.iter()));
                    try!(write!(f, ")"));
                    if i + 1 != groups.len() {
                        try!(write!(f, " "));
                    }
                }
                write!(f, ")")
            }
        }
    }
}

impl<'s, 'g> fmt::Display for LaxItem<'s, 'g> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.span())
    }
}
*/
