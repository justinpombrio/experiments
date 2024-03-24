//! Maximize the minimum size of cut subtrees in linear time, using the algorithm
//! from page 14 of:
//! "Max-Min Tree Partitioning" by Perl and Schach in The Weizmann Institute of Science, 1981.
//!
//! (This was previously a similar linear-time algorithm I discovered independently, but theirs is
//! simpler.)

use crate::binary_search::binary_search;
use crate::tree::{Tree, Weight};

impl Tree {
    pub fn max_min_weight(&mut self, min_num_cuts: u32) -> Weight {
        let min_weight = binary_search(0, self.total_weight + 1, |min_weight| {
            most_cuts(self, min_weight)
                .map(|num_cuts| num_cuts >= min_num_cuts)
                .unwrap_or(false)
        });
        most_cuts(self, min_weight); // make the cuts
        min_weight
    }
}

fn most_cuts(tree: &mut Tree, min_weight: Weight) -> Option<u32> {
    if tree.total_weight < min_weight {
        None
    } else {
        let mut uncut_weight = tree.total_weight;
        let mut num_cuts = 0;
        let partition = most_cuts_rec(tree, min_weight, &mut num_cuts, &mut uncut_weight);
        Some(num_cuts)
    }
}

fn most_cuts_rec(
    tree: &mut Tree,
    min_weight: Weight,
    num_cuts: &mut u32,
    uncut_weight: &mut Weight,
) -> Weight {
    let mut weight = tree.weight;
    for child in &mut tree.children {
        weight += most_cuts_rec(child, min_weight, num_cuts, uncut_weight);
    }

    tree.is_cut = false;
    if weight >= min_weight && *uncut_weight >= min_weight + weight {
        tree.is_cut = true;
        *num_cuts += 1;
        *uncut_weight -= weight;
        0
    } else {
        weight
    }
}

#[test]
fn test_max_min_weight() {
    use crate::oracle::oracle_max_min_weight;

    for mut tree in Tree::all_up_to_weight(9) {
        for min_num_cuts in 1..9 {
            if min_num_cuts > tree.size - 1 {
                continue;
            }
            let expected = oracle_max_min_weight(&tree, min_num_cuts);
            let actual = tree.max_min_weight(min_num_cuts);
            if actual != expected {
                println!("{}with {} cuts", tree, min_num_cuts);
            }
            assert!(tree.num_cuts() >= min_num_cuts);
            assert!(tree.min_region_weight() == actual);
            assert_eq!(actual, expected);
        }
    }
}
