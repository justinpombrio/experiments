use super::{Constraint, YesNoMaybe};
use bitvec::{bitvec, vec::BitVec};
use std::fmt::Debug;
use std::fs;
use std::hash::Hash;
use std::path::Path;

/// The constraint that `{X1, ..., Xn}` is a word from a list of allowed words. Or more generally,
/// that that sequence is present in a list of allowed sequences.
#[derive(Debug, Clone)]
pub struct Seq<T: Debug + Hash + Eq + Ord + Clone + Sized + 'static> {
    #[allow(unused)]
    seq_len: usize,
    allowed_seqs: Vec<Vec<T>>,
}

impl Seq<char> {
    /// Allowed sequences are the lowercase words of the given length from the file at `path`.
    pub fn word_list_file(
        path: impl AsRef<Path>,
        word_len: usize,
    ) -> Result<Seq<char>, std::io::Error> {
        let word_list = fs::read_to_string(path)?;
        let allowed_words = word_list
            .lines()
            .map(|s| s.trim())
            .filter(|s| &s.to_lowercase() == s)
            .filter(|s| s.chars().count() == word_len)
            .map(|s| s.chars().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        for word in &allowed_words {
            assert_eq!(word.len(), word_len);
        }
        Ok(Seq {
            seq_len: word_len,
            allowed_seqs: allowed_words,
        })
    }
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + 'static> Seq<T> {
    /// Constraint that the sequence is one of the sequences listed in `allowed_seqs`.
    pub fn new(seq_len: usize, allowed_seqs: impl IntoIterator<Item = Vec<T>>) -> Seq<T> {
        let allowed_seqs = allowed_seqs.into_iter().collect::<Vec<_>>();
        for seq in &allowed_seqs {
            assert_eq!(seq.len(), seq_len);
        }
        Seq {
            seq_len,
            allowed_seqs,
        }
    }
}

/// Represents a set of sequences.
#[derive(Debug, Clone)]
pub struct SeqSet {
    /// `set[i]` iff `allowed_seqs[i]` is in the set.
    set: BitVec,
    /// The number of sequences in the set, or `None` if the set contains at least one sequence
    /// that's not in `allowed_seqs`.
    cardinality: Option<u128>,
}

impl<T: Debug + Hash + Eq + Ord + Clone + Sized + 'static> Constraint<T> for Seq<T> {
    // Represents a set of possible words. Set[i] iff self.allowed_seqs[i] is in the set.
    type Set = SeqSet;

    const NAME: &'static str = "Seq";

    fn singleton(&self, index: usize, elem: T) -> Self::Set {
        let mut set: BitVec = bitvec![0; self.allowed_seqs.len()];
        for (i, seq) in self.allowed_seqs.iter().enumerate() {
            if seq[index] == elem {
                set.set(i, true);
            }
        }
        let cardinality = if set.count_ones() > 0 { Some(1) } else { None };
        SeqSet { set, cardinality }
    }

    fn and(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        let set = a.set & b.set;
        let cardinality = match (a.cardinality, b.cardinality) {
            (None, None) | (None, Some(_)) | (Some(_), None) => None,
            (Some(ca), Some(cb)) => Some(ca * cb),
        };
        SeqSet { set, cardinality }
    }

    fn or(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        let a_ones_count = a.set.count_ones();
        let set = a.set | b.set;
        // Brittle! We're going to assume that `b` is `and`s of `singleton`s. This implies that `|a
        // U b| = |a|` or `|a U b| = |a| + |b|`.
        if let Some(c) = b.cardinality {
            assert_eq!(c, 1);
        }
        let cardinality = match (a.cardinality, b.cardinality) {
            (None, None) | (None, Some(_)) | (Some(_), None) => None,
            (Some(ca), Some(cb)) => {
                if set.count_ones() == a_ones_count {
                    Some(ca)
                } else {
                    Some(ca + cb)
                }
            }
        };
        SeqSet { set, cardinality }
    }

    fn check(&self, set: Self::Set) -> YesNoMaybe {
        use YesNoMaybe::{Maybe, No, Yes};

        if set.set.any() {
            if set.cardinality == Some(set.set.count_ones() as u128) {
                Yes
            } else {
                Maybe
            }
        } else {
            No
        }
    }
}

#[test]
fn test_seq() {
    use YesNoMaybe::{Maybe, No, Yes};

    let s = Seq::word_list_file("/usr/share/dict/words", 3).unwrap();

    // Three words of the form `s_x`: `s{a,e,i,o}x`.
    assert_eq!(
        s.and(s.singleton(0, 's'), s.singleton(2, 'x'))
            .set
            .count_ones(),
        4
    );

    assert_eq!(
        s.check(s.and(
            s.and(s.singleton(1, 'o'), s.singleton(2, 'o')),
            s.singleton(0, 't')
        )),
        Yes
    );

    assert_eq!(
        s.check(s.and(
            s.and(s.singleton(1, 'o'), s.singleton(2, 'o')),
            s.or(s.singleton(0, 't'), s.singleton(0, 'b'))
        )),
        Yes
    );

    assert_eq!(
        s.check(s.and(
            s.and(
                s.singleton(1, 'o'),
                s.or(s.singleton(2, 'o'), s.singleton(2, 'x'))
            ),
            s.or(s.singleton(0, 't'), s.singleton(0, 'b'))
        )),
        Maybe
    );

    assert_eq!(
        s.check(s.and(
            s.singleton(0, 'x'),
            s.and(s.singleton(1, 'a'), s.singleton(2, 'c'))
        )),
        No
    );

    assert_eq!(
        s.check(s.and(
            s.or(s.singleton(0, 't'), s.singleton(0, 'n')),
            s.and(
                s.or(s.singleton(1, 't'), s.singleton(1, 'n')),
                s.or(s.singleton(2, 't'), s.singleton(2, 'n'))
            )
        )),
        No
    );

    assert_eq!(
        s.check(s.and(
            s.and(s.singleton(0, 't'), s.singleton(1, 't')),
            s.singleton(2, 't')
        )),
        No
    );
}
