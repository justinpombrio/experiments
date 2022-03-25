// TODO: More efficient representation?
//       Maybe a Vec<Vec<Word>> using a bijection Coloring<->index?

use crate::word::{Coloring, Word};
use std::collections::HashMap;

pub struct Partition(HashMap<Coloring, Vec<Word>>);

impl Partition {
    pub fn new() -> Partition {
        Partition(HashMap::new())
    }

    pub fn insert(&mut self, word: Word, coloring: Coloring) {
        self.0.entry(coloring).or_insert(Vec::new()).push(word);
    }

    /// Size of the largest subset. Smaller is better.
    #[allow(unused)]
    pub fn worst_case_score(&self) -> usize {
        let mut score = 0;
        for (_, wordset) in &self.0 {
            score = score.max(wordset.len());
        }
        score
    }

    /// Entropy of the subsets. More is better.
    pub fn entropy(&self) -> f64 {
        let total: f64 = self.0.iter().map(|(_, wordset)| wordset.len() as f64).sum();

        self.0
            .iter()
            .map(|(_, wordset)| wordset.len() as f64 / total)
            .map(|frac: f64| -frac * frac.log2())
            .sum()
    }

    /// The subset of solutions that have the given coloring.
    /// Returns `None` if there is no such subset.
    pub fn extract_subset(mut self, coloring: Coloring) -> Option<Vec<Word>> {
        self.0.remove(&coloring)
    }
}

#[test]
fn test_partition() {
    use crate::word::Color;
    use Color::{Green as G, White as W};

    let mut partition = Partition::new();
    assert_eq!(partition.worst_case_score(), 0);
    assert_eq!(partition.entropy(), 0.0);

    partition.insert(Word::from_str("00000"), Coloring([W, W, W, W, W]));
    assert_eq!(partition.worst_case_score(), 1);
    assert_eq!(partition.entropy(), 0.0);

    partition.insert(Word::from_str("22222"), Coloring([W, W, W, W, W]));
    assert_eq!(partition.worst_case_score(), 2);
    assert_eq!(partition.entropy(), 0.0);

    partition.insert(Word::from_str("44444"), Coloring([W, G, W, W, W]));
    assert_eq!(partition.worst_case_score(), 2);
    let expected_entropy = {
        let frac1 = 1.0 / 3.0 as f64;
        let frac2 = 2.0 / 3.0 as f64;
        -frac1 * frac1.log2() - frac2 * frac2.log2()
    };
    assert_eq!(partition.entropy(), expected_entropy);

    partition.insert(Word::from_str("33333"), Coloring([W, W, W, W, W]));
    assert_eq!(partition.worst_case_score(), 3);
    let expected_entropy = {
        let frac1 = 1.0 / 4.0 as f64;
        let frac2 = 3.0 / 4.0 as f64;
        -frac1 * frac1.log2() - frac2 * frac2.log2()
    };
    assert_eq!(partition.entropy(), expected_entropy);

    partition.insert(Word::from_str("55555"), Coloring([W, W, W, W, W]));
    partition.insert(Word::from_str("11111"), Coloring([W, G, W, W, W]));
    partition.insert(Word::from_str("66666"), Coloring([W, W, W, W, W]));
    partition.insert(Word::from_str("77777"), Coloring([W, G, W, G, W]));
    partition.insert(Word::from_str("88888"), Coloring([W, G, W, G, W]));
    partition.insert(Word::from_str("99999"), Coloring([W, W, W, W, W]));
    assert_eq!(partition.worst_case_score(), 6);
    let expected_entropy = {
        let frac1 = 0.6 as f64;
        let frac2 = 0.2 as f64;
        let frac3 = 0.2 as f64;
        -frac1 * frac1.log2() - frac2 * frac2.log2() - frac3 * frac3.log2()
    };
    assert_eq!(partition.entropy(), expected_entropy);
}
