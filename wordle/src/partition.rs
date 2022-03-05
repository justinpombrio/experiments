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

    // Could also use an entropy score, but that just seems worse.
    // Entropy says the partition {4, 1, 1, 1, 1} is equally good
    // as {2, 2, 2, 2}, but the former has a good chance of hitting
    // the 4 and doing poorly.
    /// Size of the largest subset. Smaller is better.
    pub fn worst_case_score(&self) -> usize {
        let mut score = 0;
        for (_, wordset) in &self.0 {
            score = score.max(wordset.len());
        }
        score
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
    use Color::{Green as G, White as W, Yellow as Y};

    let mut partition = Partition::new();
    assert_eq!(partition.worst_case_score(), 0);

    partition.insert(Word::from_str("00000"), [W, W, W, W, W]);
    assert_eq!(partition.worst_case_score(), 1);

    partition.insert(Word::from_str("22222"), [W, W, W, W, W]);
    assert_eq!(partition.worst_case_score(), 2);

    partition.insert(Word::from_str("44444"), [W, G, W, W, W]);
    assert_eq!(partition.worst_case_score(), 2);

    partition.insert(Word::from_str("33333"), [W, W, W, W, W]);
    assert_eq!(partition.worst_case_score(), 3);

    partition.insert(Word::from_str("55555"), [W, W, W, W, W]);
    partition.insert(Word::from_str("11111"), [W, G, W, W, W]);
    partition.insert(Word::from_str("66666"), [W, W, W, W, W]);
    partition.insert(Word::from_str("77777"), [W, G, W, G, W]);
    partition.insert(Word::from_str("88888"), [W, G, W, G, W]);
    partition.insert(Word::from_str("99999"), [W, W, W, W, W]);
    assert_eq!(partition.worst_case_score(), 6);
}
