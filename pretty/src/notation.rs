use typed_arena::Arena;

pub type NotationRef<'a> = &'a Notation<'a>;

#[derive(Debug)]
pub enum Notation<'a> {
    Newline,
    Text(String, u32),
    Flat(NotationRef<'a>),
    Indent(u32, NotationRef<'a>),
    Concat(Vec<NotationRef<'a>>),
    Choice(NotationRef<'a>, NotationRef<'a>),
}

pub struct NotationBuilder<'a>(Arena<Notation<'a>>);

impl<'a> NotationBuilder<'a> {
    pub fn new() -> NotationBuilder<'a> {
        NotationBuilder(Arena::new())
    }

    /// Display a newline
    pub fn nl(&'a self) -> NotationRef<'a> {
        self.0.alloc(Notation::Newline)
    }

    /// Display text exactly as-is. The text should not contain a newline!
    pub fn txt(&'a self, text: impl ToString) -> NotationRef<'a> {
        let string = text.to_string();
        let width = unicode_width::UnicodeWidthStr::width(&string as &str) as u32;
        self.0.alloc(Notation::Text(string, width))
    }
    /// Use the leftmost option of every choice in the contained Notation.
    /// If the contained Notation follows the recommendation of not putting
    /// newlines in the left-most options of choices, then this `flat` will
    /// be displayed all on one line.
    pub fn flat(&'a self, notation: NotationRef<'a>) -> NotationRef<'a> {
        self.0.alloc(Notation::Flat(notation))
    }

    /// Increase the indentation level of the contained notation by the given width. The
    /// indentation level determines the number of spaces put after `Newline`s. (It therefore
    /// doesn't affect the first line of a notation.)
    pub fn indent(&'a self, indent: u32, notation: NotationRef<'a>) -> NotationRef<'a> {
        self.0.alloc(Notation::Indent(indent, notation))
    }

    /// Display the notations in order, with no spacing in between.
    pub fn concat(
        &'a self,
        notations: impl IntoIterator<Item = NotationRef<'a>>,
    ) -> NotationRef<'a> {
        let notations = notations.into_iter().collect::<Vec<_>>();
        self.0.alloc(Notation::Concat(notations))
    }

    /// If inside a `flat`, _or_ the first line of the first notation fits within
    /// the required width, then display the first notation. Otherwise, display
    /// the second notation.
    pub fn choice(&'a self, first: NotationRef<'a>, second: NotationRef<'a>) -> NotationRef<'a> {
        self.0.alloc(Notation::Choice(first, second))
    }
}
