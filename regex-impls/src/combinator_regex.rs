/// A trait for Regex combinators. The key to combinators is a shared interface.
///
/// This interface allows for `O(NM)` regex parsing. It took me a few attempts to find it. My first
/// attempts would have served as parser combinator interfaces, which as far as I could tell do
/// _not_ allow for `O(NM)` regex parsing.
///
/// Users only need call the `Regex.is_match(&str)` method.
///
/// # Spec
///
/// A `Regex` needs to maintain state as it runs. You _can_ think of this `State` as the _set_ of
/// NFA states, but you don't _have_ to. It just has to obey this spec:
///
/// **Definition.** At any time, this state is "tracking" a set of strings:
///
/// - The state constructed by `Regex::init_state()` tracks an empty set of strings.
/// - The `start()` method adds the empty string to the tracking set.
/// - The `advance(char)` method appends the char to each string in the tracking set.
///
/// **Requirement.** The `accepts()` method returns true iff the `Regex` accepts any of the strings
/// in its tracking set.
pub trait Regex: Clone {
    /// Reset to the initial, _empty_ state. In NFA terms, this is an empty set of states.
    fn initialize(&mut self);
    /// Track an empty string.
    fn start(&mut self);
    /// Append `ch` to every string being tracked.
    fn advance(&mut self, ch: char);
    /// Does the regex match any of the tracked strings?
    fn accepts(&self) -> bool;
    /// Is it true that both (i) accepts() is false, and (ii) accepts() will remain false for any
    /// possible sequence of `advance`s? This is used for a short-circuiting optimization.
    fn is_dead(&self) -> bool;

    /// Does the input match this regex? Note that this is not looking for an occurrence of the
    /// Regex pattern _somewhere_ in the input; it's specifically checking that the _entire input_
    /// matches the regex.
    fn is_match(&mut self, input: &str) -> bool {
        self.initialize();
        self.start();
        for ch in input.chars() {
            self.advance(ch);
            if self.is_dead() {
                return false;
            }
        }
        self.accepts()
    }
}

/*******************/
/* Char Predicates */
/*******************/

trait Predicate: Copy {
    fn matches(&self, ch: char) -> bool;
}

#[derive(Clone, Copy)]
struct SingleChar<P: Predicate> {
    predicate: P,
    state: SimpleState,
}

impl<P: Predicate> SingleChar<P> {
    fn new(predicate: P) -> SingleChar<P> {
        SingleChar {
            predicate,
            state: SimpleState::Neither,
        }
    }
}

impl<P: Predicate> Regex for SingleChar<P> {
    fn initialize(&mut self) {
        self.state = SimpleState::Neither;
    }

    fn start(&mut self) {
        use SimpleState::*;

        self.state = match self.state {
            Neither | Start => Start,
            Both | End => Both,
        }
    }

    fn advance(&mut self, ch: char) {
        use SimpleState::*;

        if self.predicate.matches(ch) {
            self.state = match self.state {
                Neither | End => Neither,
                Both | Start => End,
            };
        } else {
            self.state = Neither;
        }
    }

    fn accepts(&self) -> bool {
        use SimpleState::*;

        match self.state {
            End | Both => true,
            Start | Neither => false,
        }
    }

    fn is_dead(&self) -> bool {
        self.state == SimpleState::Neither
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SimpleState {
    Start,
    End,
    Both,
    Neither,
}

/***********************/
/* Single Char Regexes */
/***********************/

#[derive(Clone, Copy)]
struct Dot;

impl Predicate for Dot {
    fn matches(&self, _ch: char) -> bool {
        true
    }
}

#[derive(Clone, Copy)]
struct Char(char);

impl Predicate for Char {
    fn matches(&self, ch: char) -> bool {
        self.0 == ch
    }
}

#[derive(Clone, Copy)]
struct CharRange(char, char);

impl Predicate for CharRange {
    fn matches(&self, ch: char) -> bool {
        self.0 <= ch && ch <= self.1
    }
}

/*********/
/* Empty */
/*********/

#[derive(Clone, Copy)]
struct Empty {
    empty: bool,
}

impl Empty {
    fn new() -> Empty {
        Empty { empty: false }
    }
}

impl Regex for Empty {
    fn initialize(&mut self) {
        self.empty = false;
    }

    fn start(&mut self) {
        self.empty = true;
    }

    fn advance(&mut self, _: char) {
        self.empty = false;
    }

    fn accepts(&self) -> bool {
        self.empty
    }

    fn is_dead(&self) -> bool {
        !self.empty
    }
}

/********/
/* Star */
/********/

#[derive(Clone)]
struct Star<P: Regex> {
    init: bool,
    state: P,
}

impl<P: Regex> Star<P> {
    fn new(regex: P) -> Star<P> {
        Star {
            init: false,
            state: regex,
        }
    }
}

impl<P: Regex> Regex for Star<P> {
    fn initialize(&mut self) {
        self.init = false;
        self.state.initialize();
    }

    fn start(&mut self) {
        self.init = true;
        self.state.start();
    }

    fn advance(&mut self, ch: char) {
        self.init = false;
        self.state.advance(ch);
        if self.state.accepts() {
            self.init = true;
            self.state.start();
        }
    }

    fn accepts(&self) -> bool {
        self.init || self.state.accepts()
    }

    fn is_dead(&self) -> bool {
        !self.init && self.state.is_dead()
    }
}

/*********/
/* Maybe */
/*********/

#[derive(Clone)]
struct Maybe<P: Regex> {
    init: bool,
    state: P,
}

impl<P: Regex> Maybe<P> {
    fn new(regex: P) -> Maybe<P> {
        Maybe {
            init: false,
            state: regex,
        }
    }
}

impl<P: Regex> Regex for Maybe<P> {
    fn initialize(&mut self) {
        self.init = false;
        self.state.initialize();
    }

    fn start(&mut self) {
        self.init = true;
        self.state.start();
    }

    fn advance(&mut self, ch: char) {
        self.init = false;
        self.state.advance(ch);
    }

    fn accepts(&self) -> bool {
        self.init || self.state.accepts()
    }

    fn is_dead(&self) -> bool {
        !self.init && self.state.is_dead()
    }
}

/*******/
/* Alt */
/*******/

#[derive(Clone)]
struct Alt<P: Regex, Q: Regex>(P, Q);

impl<P: Regex, Q: Regex> Regex for Alt<P, Q> {
    fn initialize(&mut self) {
        self.0.initialize();
        self.1.initialize();
    }

    fn start(&mut self) {
        self.0.start();
        self.1.start();
    }

    fn advance(&mut self, ch: char) {
        self.0.advance(ch);
        self.1.advance(ch);
    }

    fn accepts(&self) -> bool {
        self.0.accepts() || self.1.accepts()
    }

    fn is_dead(&self) -> bool {
        self.0.is_dead() && self.1.is_dead()
    }
}

/*******/
/* Seq */
/*******/

#[derive(Clone)]
struct Seq<P: Regex, Q: Regex>(P, Q);

impl<P: Regex, Q: Regex> Regex for Seq<P, Q> {
    fn initialize(&mut self) {
        self.0.initialize();
        self.1.initialize();
    }

    fn start(&mut self) {
        self.0.start();
        if self.0.accepts() {
            self.1.start();
        }
    }

    fn advance(&mut self, ch: char) {
        self.1.advance(ch);
        self.0.advance(ch);
        if self.0.accepts() {
            self.1.start();
        }
    }

    fn accepts(&self) -> bool {
        self.1.accepts()
    }

    fn is_dead(&self) -> bool {
        self.0.is_dead() && self.1.is_dead()
    }
}

pub mod combinators {
    use super::*;

    pub fn empty() -> impl Regex {
        Empty::new()
    }

    pub fn dot() -> impl Regex {
        SingleChar::new(Dot)
    }

    pub fn achar(ch: char) -> impl Regex {
        SingleChar::new(Char(ch))
    }

    pub fn char_range(min_ch: char, max_ch: char) -> impl Regex {
        SingleChar::new(CharRange(min_ch as char, max_ch as char))
    }

    pub fn seq(first: impl Regex, second: impl Regex) -> impl Regex {
        Seq(first, second)
    }

    pub fn alt(left: impl Regex, right: impl Regex) -> impl Regex {
        Alt(left, right)
    }

    pub fn star(regex: impl Regex) -> impl Regex {
        Star::new(regex)
    }

    pub fn maybe(regex: impl Regex) -> impl Regex {
        Maybe::new(regex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    // 100 chars in total (50 each)
    const ANUM: &str = "31415926535897932384626.4338327950288419716939937";
    const NOTANUM: &str = "31415926535897932384626.4338327.95028841971693993";

    #[test]
    fn tests() {
        use combinators::*;

        let mut zero = achar('0');
        assert!(!zero.is_match(""));
        assert!(zero.is_match("0"));
        assert!(!zero.is_match("1"));
        assert!(!zero.is_match("00"));
        assert!(!zero.is_match("01"));
        assert!(!zero.is_match("10"));

        let mut digit = char_range('0', '1');
        assert!(!digit.is_match(""));
        assert!(digit.is_match("0"));
        assert!(digit.is_match("1"));
        assert!(!digit.is_match("2"));
        assert!(!digit.is_match("01"));
        assert!(!digit.is_match("00"));

        let mut zeroes = star(achar('0'));
        assert!(zeroes.is_match(""));
        assert!(zeroes.is_match("0"));
        assert!(zeroes.is_match("00"));
        assert!(!zeroes.is_match("1"));
        assert!(!zeroes.is_match("01"));
        assert!(!zeroes.is_match("0010"));

        let mut oh_one = seq(achar('0'), achar('1'));
        assert!(oh_one.is_match("01"));

        let mut integer = alt(achar('0'), seq(achar('1'), star(char_range('0', '1'))));
        assert!(integer.is_match("0"));
        assert!(!integer.is_match("2"));
        assert!(integer.is_match("10"));
        assert!(!integer.is_match("01"));
        assert!(integer.is_match("1101001"));
        assert!(!integer.is_match("0101001"));
        assert!(!integer.is_match("1101021"));
    }

    // ~6ns / char
    #[bench]
    fn this_crate(bencher: &mut Bencher) {
        use combinators::*;

        let integer = alt(
            achar('0'),
            seq(char_range('1', '9'), star(char_range('0', '9'))),
        );
        let tail = seq(achar('.'), star(char_range('0', '9')));
        let mut decimal = seq(integer, maybe(tail));

        bencher.iter(|| {
            assert!(decimal.is_match(ANUM));
            assert!(!decimal.is_match(NOTANUM));
        });
    }

    // Burnt Sushi's Regexes.
    // It's 3 times faster on this example.
    #[bench]
    fn regex_crate(bencher: &mut Bencher) {
        use regex::Regex;
        let number = Regex::new("^(0|[1-9][0-9]*)(\\.[0-9]*)?$").unwrap();
        bencher.iter(|| {
            assert!(number.is_match(ANUM));
            assert!(!number.is_match(NOTANUM));
        })
    }
}
