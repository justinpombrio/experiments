use crate::initial_set::InitialSet;
use crate::lexer::LexemeIter;
use crate::{GrammarError, ParseResult, Parser};
use std::cell::{Cell, OnceCell};
use std::rc::{Rc, Weak};

/*========================================*/
/*          Parser: Recursion             */
/*========================================*/

struct RecurP<T: Clone> {
    name: String,
    parser: OnceCell<Box<dyn Parser<T>>>,
    initial_set: Cell<Option<InitialSet>>,
}

impl<T: Clone> Clone for RecurP<T> {
    fn clone(&self) -> RecurP<T> {
        RecurP {
            name: self.name.clone(),
            parser: self.parser.clone(),
            initial_set: Cell::new(None),
        }
    }
}

impl<T: Clone> Parser<T> for RecurP<T> {
    fn name(&self, is_empty: Option<bool>) -> String {
        if is_empty == Some(true) {
            format!("empty {}", self.name.clone())
        } else {
            self.name.clone()
        }
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        if let Some(initial_set) = self.initial_set.take() {
            // We're currently in a recursive case of validate() (see `else` branch).
            // Use the initial_set we set for ourselves.
            Ok(initial_set)
        } else {
            // Compute our initial set with a recursive depth limited to 2.
            // This is guaranteed to be the same as the limit as the depth goes to infinity.
            let initial_set_0 = InitialSet::new_void();
            self.initial_set.set(Some(initial_set_0));
            let initial_set_1 = self.validate()?;
            self.initial_set.set(Some(initial_set_1));
            let initial_set_2 = self.validate()?;
            self.initial_set.set(None);
            Ok(initial_set_2)
        }
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<T> {
        self.parser.get().unwrap().parse(stream)
    }
}

pub struct Recursive<T: Clone>(Rc<RecurP<T>>);

impl<T: Clone> Recursive<T> {
    pub fn new(name: &str) -> Recursive<T> {
        Recursive(Rc::new(RecurP {
            name: name.to_owned(),
            parser: OnceCell::new(),
            initial_set: Cell::new(None),
        }))
    }

    pub fn refn(&self) -> impl Parser<T> + Clone {
        RecurPWeak {
            name: self.0.name.clone(),
            weak: Rc::downgrade(&self.0),
        }
    }

    pub fn define(self, parser: impl Parser<T> + Clone + 'static) -> impl Parser<T> + Clone {
        match self.0.parser.set(Box::new(parser)) {
            Ok(()) => (),
            Err(_) => panic!("Bug in recur: failed to set OnceCell"),
        }
        RecurPStrong(self.0)
    }
}

/* ========== Recur: Weak ========== */

#[derive(Clone)]
struct RecurPWeak<T: Clone> {
    name: String,
    weak: Weak<RecurP<T>>,
}

impl<T: Clone> RecurPWeak<T> {
    fn unwrap<R>(&self, cb: impl FnOnce(&RecurP<T>) -> R) -> R {
        match self.weak.upgrade() {
            None => panic!(
                "Recursive: you must call 'define()' before using recursive parser '{}'",
                self.name
            ),
            Some(rc) => cb(rc.as_ref()),
        }
    }
}

impl<T: Clone> Parser<T> for RecurPWeak<T> {
    fn name(&self, is_empty: Option<bool>) -> String {
        self.unwrap(|p| p.name(is_empty))
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.unwrap(|p| p.validate())
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<T> {
        self.unwrap(|p| p.parse(stream))
    }
}

/* ========== Recur: Strong ========== */

#[derive(Clone)]
struct RecurPStrong<T: Clone>(Rc<RecurP<T>>);

impl<T: Clone> Parser<T> for RecurPStrong<T> {
    fn name(&self, is_empty: Option<bool>) -> String {
        self.0.as_ref().name(is_empty)
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.0.as_ref().validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<T> {
        self.0.as_ref().parse(stream)
    }
}
