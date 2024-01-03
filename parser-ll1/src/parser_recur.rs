use crate::initial_set::InitialSet;
use crate::lexer::LexemeIter;
use crate::{GrammarError, ParseResult, Parser};
use std::cell::{Cell, OnceCell};
use std::rc::{Rc, Weak};

/*========================================*/
/*          Parser: Recursion             */
/*========================================*/

struct RecurP<O: Clone> {
    name: String,
    parser: OnceCell<Box<dyn Parser<Output = O>>>,
    initial_set: Cell<Option<InitialSet>>,
}

impl<O: Clone> Clone for RecurP<O> {
    fn clone(&self) -> RecurP<O> {
        RecurP {
            name: self.name.clone(),
            parser: self.parser.clone(),
            initial_set: Cell::new(None),
        }
    }
}

impl<O: Clone> Parser for RecurP<O> {
    type Output = O;

    fn name(&self) -> String {
        self.name.clone()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        if let Some(initial_set) = self.initial_set.take() {
            // We're currently in a recursive case of validate() (see `else` branch).
            // Use the initial_set we set for ourselves.
            Ok(initial_set)
        } else {
            // Compute our initial set with a recursive depth limited to 2.
            // This is guaranteed to be the same as the limit as the depth goes to infinity.
            let initial_set_0 = InitialSet::new_void(&self.name);
            self.initial_set.set(Some(initial_set_0));
            let initial_set_1 = self.validate()?;
            self.initial_set.set(Some(initial_set_1));
            let initial_set_2 = self.validate()?;
            self.initial_set.set(None);
            Ok(initial_set_2)
        }
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<O> {
        self.parser.get().unwrap().parse(stream)
    }
}

pub struct Recursive<O: Clone>(Rc<RecurP<O>>);

impl<O: Clone> Recursive<O> {
    pub fn new(name: &str) -> Recursive<O> {
        Recursive(Rc::new(RecurP {
            name: name.to_owned(),
            parser: OnceCell::new(),
            initial_set: Cell::new(None),
        }))
    }

    pub fn refn(&self) -> impl Parser<Output = O> + Clone {
        RecurPWeak(Rc::downgrade(&self.0))
    }

    pub fn define(
        self,
        parser: impl Parser<Output = O> + Clone + 'static,
    ) -> impl Parser<Output = O> + Clone {
        match self.0.parser.set(Box::new(parser)) {
            Ok(()) => (),
            Err(_) => panic!("Bug in recur: failed to set OnceCell"),
        }
        RecurPStrong(self.0)
    }
}

/* ========== Recur: Weak ========== */

#[derive(Clone)]
struct RecurPWeak<O: Clone>(Weak<RecurP<O>>);

impl<O: Clone> Parser for RecurPWeak<O> {
    type Output = O;

    fn name(&self) -> String {
        self.0.upgrade().unwrap().name()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.0.upgrade().unwrap().validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<O> {
        self.0.upgrade().unwrap().parse(stream)
    }
}

/* ========== Recur: Strong ========== */

#[derive(Clone)]
struct RecurPStrong<O: Clone>(Rc<RecurP<O>>);

impl<O: Clone> Parser for RecurPStrong<O> {
    type Output = O;

    fn name(&self) -> String {
        self.0.as_ref().name()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.0.as_ref().validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<O> {
        self.0.as_ref().parse(stream)
    }
}
