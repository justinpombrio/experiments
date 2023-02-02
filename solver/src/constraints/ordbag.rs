use super::Constraint;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::Hash;

/// The constraint that `{X1, ..., Xn} = expected`
pub struct Bag<T: Debug + Hash + Eq + Ord + Clone + Sized + 'static> {
    expected: Vec<T>,
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + 'static> Bag<T> {
    pub fn new(expected: impl IntoIterator<Item = T>) -> Bag<T> {
        Bag {
            expected: expected.into_iter().collect::<Vec<_>>(),
        }
    }
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + 'static> Constraint<T> for Bag<T> {
    type Set = (Vec<T>, Vec<T>);

    const NAME: &'static str = "Bag";

    fn singleton(&self, _index: usize, elem: T) -> Self::Set {
        (vec![elem.clone()], vec![elem])
    }

    fn and(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        (append_seq(a.0, b.0), append_seq(a.1, b.1))
    }

    fn or(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        (
            MinSeqPair(SeqPair::new(a.0.into_iter(), b.0.into_iter())).collect(),
            MaxSeqPair(SeqPair::new(a.1.into_iter(), b.1.into_iter())).collect(),
        )
    }

    fn check(&self, set: Self::Set) -> bool {
        SeqPair::new(set.0.into_iter(), self.expected.clone().into_iter()).is_subset()
            && SeqPair::new(self.expected.clone().into_iter(), set.1.into_iter()).is_subset()
    }
}

fn append_seq<T: Ord>(vec_1: Vec<T>, mut vec_2: Vec<T>) -> Vec<T> {
    let mut result = vec_1;
    result.append(&mut vec_2);
    result.sort();
    result
}

struct SeqPair<T: Ord, I: Iterator<Item = T>, J: Iterator<Item = T>> {
    xs: std::iter::Peekable<I>,
    ys: std::iter::Peekable<J>,
}

impl<T: Ord, I: Iterator<Item = T>, J: Iterator<Item = T>> SeqPair<T, I, J> {
    fn new(seq_1: I, seq_2: J) -> SeqPair<T, I, J> {
        SeqPair {
            xs: seq_1.peekable(),
            ys: seq_2.peekable(),
        }
    }

    fn next(&mut self) -> Option<(T, Ordering)> {
        match (self.xs.peek(), self.ys.peek()) {
            (Some(x), Some(y)) if x == y => {
                self.xs.next();
                Some((self.ys.next().unwrap(), Ordering::Equal))
            }
            (Some(x), Some(y)) if x < y => Some((self.xs.next().unwrap(), Ordering::Less)),
            (Some(_), Some(_)) => Some((self.ys.next().unwrap(), Ordering::Greater)),
            (Some(_), None) => Some((self.xs.next().unwrap(), Ordering::Less)),
            (None, Some(_)) => Some((self.ys.next().unwrap(), Ordering::Greater)),
            (None, None) => None,
        }
    }

    fn is_subset(&mut self) -> bool {
        while let Some((_, ord)) = self.next() {
            if ord == Ordering::Less {
                return false;
            }
        }
        true
    }
}

struct MinSeqPair<T: Ord, I: Iterator<Item = T>, J: Iterator<Item = T>>(SeqPair<T, I, J>);

impl<T: Ord, I: Iterator<Item = T>, J: Iterator<Item = T>> Iterator for MinSeqPair<T, I, J> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        loop {
            match self.0.next() {
                Some((x, Ordering::Equal)) => return Some(x),
                Some((_, Ordering::Less)) => (),
                Some((_, Ordering::Greater)) => (),
                None => return None,
            }
        }
    }
}

struct MaxSeqPair<T: Ord, I: Iterator<Item = T>, J: Iterator<Item = T>>(SeqPair<T, I, J>);

impl<T: Ord, I: Iterator<Item = T>, J: Iterator<Item = T>> Iterator for MaxSeqPair<T, I, J> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if let Some((x, _)) = self.0.next() {
            Some(x)
        } else {
            None
        }
    }
}

#[test]
fn test_seq_pair() {
    let mut pair = SeqPair::new([1, 2, 2].into_iter(), [0, 2, 4].into_iter());
    assert_eq!(pair.next(), Some((0, Ordering::Greater)));
    assert_eq!(pair.next(), Some((1, Ordering::Less)));
    assert_eq!(pair.next(), Some((2, Ordering::Equal)));
    assert_eq!(pair.next(), Some((2, Ordering::Less)));
    assert_eq!(pair.next(), Some((4, Ordering::Greater)));
    assert_eq!(pair.next(), None);
    assert_eq!(pair.next(), None);
}
