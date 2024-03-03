use std::ops::{BitAnd, BitOr};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Notation(pub(crate) Rc<NotationInner>);

#[derive(Debug, Clone)]
pub enum NotationInner {
    Newline,
    Text(String, u32),
    Flat(Notation),
    Indent(u32, Notation),
    Concat(Notation, Notation),
    Choice(Notation, Notation),
}

/// Display a newline
pub fn nl() -> Notation {
    Notation(Rc::new(NotationInner::Newline))
}

/// Display text exactly as-is. The text should not contain a newline!
pub fn txt(s: impl ToString) -> Notation {
    let string = s.to_string();
    let width = unicode_width::UnicodeWidthStr::width(&string as &str) as u32;
    Notation(Rc::new(NotationInner::Text(string, width)))
}

/// Use the leftmost option of every choice in the contained Notation.
/// If the contained Notation follows the recommendation of not putting
/// newlines in the left-most options of choices, then this `flat` will
/// be displayed all on one line.
pub fn flat(notation: Notation) -> Notation {
    Notation(Rc::new(NotationInner::Flat(notation)))
}

/// Increase the indentation level of the contained notation by the given width. The
/// indentation level determines the number of spaces put after `Newline`s. (It therefore
/// doesn't affect the first line of a notation.)
pub fn indent(indent: u32, notation: Notation) -> Notation {
    Notation(Rc::new(NotationInner::Indent(indent, notation)))
}

impl BitAnd<Notation> for Notation {
    type Output = Notation;

    /// Display both notations. The first character of the right notation immediately
    /// follows the last character of the left notation.
    fn bitand(self, other: Notation) -> Notation {
        Notation(Rc::new(NotationInner::Concat(self, other)))
    }
}

impl BitOr<Notation> for Notation {
    type Output = Notation;

    /// If inside a `flat`, _or_ the first line of the left notation fits within
    /// the required width, then display the left notation. Otherwise, display
    /// the right notation.
    fn bitor(self, other: Notation) -> Notation {
        Notation(Rc::new(NotationInner::Choice(self, other)))
    }
}
