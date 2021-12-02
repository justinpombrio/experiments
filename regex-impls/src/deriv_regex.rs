use typed_arena::Arena;

#[derive(Default)]
pub struct RegexStorage<'a>(Arena<Regex<'a>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Regex<'a> {
    nullable: bool,
    contents: RegexContents<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegexContents<'a> {
    Void,
    Epsilon,
    Char(char),
    CharSet(char, char),
    Seq(&'a Regex<'a>, &'a Regex<'a>),
    Alt(&'a Regex<'a>, &'a Regex<'a>),
    Star(&'a Regex<'a>),
}

impl<'a> RegexStorage<'a> {
    pub fn new() -> RegexStorage<'a> {
        RegexStorage(Arena::new())
    }

    pub fn matches(&'a self, input: &str, regex: Regex) -> bool {
        use RegexContents::*;

        let storage = RegexStorage::new();
        let mut regex = regex;
        for c in input.chars() {
            regex = storage.deriv(c, regex);
            if regex.contents == Void {
                return false;
            }
        }
        regex.nullable
    }

    fn deriv(&'a self, c: char, regex: Regex<'a>) -> Regex<'a> {
        use RegexContents::*;

        match regex.contents {
            Void | Epsilon => self.void(),
            Char(c2) if c == c2 => self.epsilon(),
            Char(_) => self.void(),
            CharSet(min, max) if min <= c && c <= max => self.epsilon(),
            CharSet(_, _) => self.void(),
            Seq(x, y) if x.nullable => self.alt(self.seq(self.deriv(c, *x), *y), self.deriv(c, *y)),
            Seq(x, y) => self.seq(self.deriv(c, *x), *y),
            Alt(x, y) => self.alt(self.deriv(c, *x), self.deriv(c, *y)),
            Star(x) => self.seq(self.deriv(c, *x), regex),
        }
    }

    pub fn void(&self) -> Regex<'a> {
        Regex {
            nullable: false,
            contents: RegexContents::Void,
        }
    }

    pub fn epsilon(&self) -> Regex<'a> {
        Regex {
            nullable: true,
            contents: RegexContents::Epsilon,
        }
    }

    pub fn char(&self, ch: char) -> Regex<'a> {
        Regex {
            nullable: false,
            contents: RegexContents::Char(ch),
        }
    }

    pub fn char_set(&self, min: char, max: char) -> Regex<'a> {
        Regex {
            nullable: false,
            contents: RegexContents::CharSet(min, max),
        }
    }

    pub fn seq(&'a self, x: Regex<'a>, y: Regex<'a>) -> Regex<'a> {
        use RegexContents::*;

        match (x.contents, y.contents) {
            (Void, _) | (_, Void) => self.void(),
            (Epsilon, _) => y,
            (_, Epsilon) => x,
            (_, _) => {
                let x = self.0.alloc(x);
                let y = self.0.alloc(y);
                Regex {
                    nullable: x.nullable && y.nullable,
                    contents: Seq(x, y),
                }
            }
        }
    }

    pub fn alt(&'a self, x: Regex<'a>, y: Regex<'a>) -> Regex<'a> {
        use RegexContents::*;

        match (x.contents, y.contents) {
            (Void, _) => y,
            (_, Void) => x,
            (Epsilon, _) if y.nullable => y,
            (_, Epsilon) if x.nullable => x,
            (_, _) => {
                let x = self.0.alloc(x);
                let y = self.0.alloc(y);
                Regex {
                    nullable: x.nullable || y.nullable,
                    contents: Alt(x, y),
                }
            }
        }
    }

    pub fn star(&'a self, x: Regex<'a>) -> Regex<'a> {
        use RegexContents::*;

        match x.contents {
            Void | Epsilon => self.epsilon(),
            Star(_) => x,
            _ => {
                let x = self.0.alloc(x);
                Regex {
                    nullable: true,
                    contents: Star(x),
                }
            }
        }
    }
}

#[cfg(test)]
mod deriv_tests {
    use super::*;
    use test::Bencher;

    const ANUM: &str = "31415926535897932384626.4338327950288419716939937";
    const NOTANUM: &str = "31415926535897932384626.4338327.95028841971693993";

    #[bench]
    fn this_crate(bencher: &mut Bencher) {
        let storage = RegexStorage::new();
        let zero = storage.char('0');
        let nonzero = storage.char_set('1', '9');
        let dot = storage.char('.');
        let epsilon = storage.epsilon();
        let digit = storage.char_set('0', '9');
        let digits = storage.star(digit);
        let leading = storage.alt(zero, storage.seq(nonzero, digits));
        let trailing = storage.alt(epsilon, storage.seq(dot, digits));
        let number = storage.seq(leading, trailing);

        assert!(storage.matches("2", number));
        assert!(storage.matches("2.0", number));
        assert!(!storage.matches(".0", number));

        bencher.iter(|| {
            assert!(storage.matches(ANUM, number));
            assert!(!storage.matches(NOTANUM, number));
        })
    }

    // Burnt Sushi's Regexes.
    // 20 times faster on this example.
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
