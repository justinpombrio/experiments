//! Maximize the minimum size of cut subtrees in linear time, using the algorithm
//! from page 14 of:
//! "Max-Min Tree Partitioning" by Perl and Schach in The Weizmann Institute of Science, 1981.

use crate::tree::{Tree, Weight};

#[derive(Debug, Clone)]
struct MostCuts {
    num_cuts: u32,
    remaining_weight: Weight,
}

impl Tree {
    pub fn max_min_weight(&mut self, min_num_cuts: u32) -> Weight {
        let mut lower_bound = 0; // possible
        let mut upper_bound = self.total_weight; // impossible
        while lower_bound + 1 < upper_bound {
            let mid = lower_bound + (upper_bound - lower_bound) / 2;
            let num_cuts = most_cuts(self, mid);
            if num_cuts.is_some() && num_cuts.unwrap() >= min_num_cuts {
                lower_bound = mid;
            } else {
                upper_bound = mid;
            }
        }
        most_cuts(self, lower_bound); // make the cuts
        lower_bound
    }
}

fn most_cuts(tree: &mut Tree, min_weight: Weight) -> Option<u32> {
    if tree.total_weight < min_weight {
        None
    } else {
        let partition = most_cuts_rec(tree, min_weight, tree.total_weight);
        Some(partition.num_cuts - 1)
    }
}

fn most_cuts_rec(tree: &mut Tree, min_weight: Weight, mut uncut_weight: Weight) -> MostCuts {
    let mut weight = tree.weight;
    let mut num_cuts = 0;
    for child in &mut tree.children {
        let partition = most_cuts_rec(child, min_weight, uncut_weight);
        num_cuts += partition.num_cuts;
        weight += partition.remaining_weight;
        uncut_weight -= child.total_weight - partition.remaining_weight;
    }

    if weight >= min_weight && uncut_weight >= min_weight {
        tree.is_cut = true;
        num_cuts += 1;
        weight = 0;
    }

    MostCuts {
        num_cuts,
        remaining_weight: weight,
    }
}

#[test]
fn test_max_min_weight() {
    use crate::oracle::oracle_max_min_weight;

    for mut tree in Tree::all_up_to_weight(9) {
        for min_num_cuts in 1..9 {
            let expected = oracle_max_min_weight(&tree, min_num_cuts);
            let actual = tree.max_min_weight(min_num_cuts);
            if actual != expected {
                println!("{}with {} cuts", tree, min_num_cuts);
            }
            if min_num_cuts <= tree.size - 1 {
                assert!(tree.num_cuts() >= min_num_cuts);
            }
            assert_eq!(actual, expected);
        }
    }
}
