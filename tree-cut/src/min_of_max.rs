use crate::binary_search::reverse_binary_search;
use crate::tree::{Tree, Weight};

impl Tree {
    pub fn min_max_weight(&mut self, max_num_cuts: u32) -> Weight {
        let max_weight = reverse_binary_search(0, self.total_weight, |max_weight| {
            let mut num_cuts = 0;
            let ok = fewest_cuts(self, max_weight, &mut num_cuts).is_some();
            ok && num_cuts <= max_num_cuts
        });
        fewest_cuts(self, max_weight, &mut 0); // make the cuts
        max_weight
    }
}

fn fewest_cuts(tree: &mut Tree, max_weight: Weight, num_cuts: &mut u32) -> Option<Weight> {
    let mut child_partitions = Vec::new();
    for (i, child) in tree.children.iter_mut().enumerate() {
        let child_weight = fewest_cuts(child, max_weight, num_cuts)?;
        child_partitions.push((i, child_weight));
    }
    child_partitions.sort_by_key(|(_, weight)| *weight);

    let mut weight = tree.weight;
    for (i, child_weight) in child_partitions {
        if child_weight + weight <= max_weight {
            tree.children[i].is_cut = false;
            weight += child_weight;
        } else {
            tree.children[i].is_cut = true;
            *num_cuts += 1;
        }
    }
    if weight > max_weight {
        None
    } else {
        Some(weight)
    }
}

#[test]
fn test_min_max_weight() {
    use crate::oracle::oracle_min_max_weight;

    for mut tree in Tree::all_up_to_weight(9) {
        for max_num_cuts in 1..7 {
            let expected = oracle_min_max_weight(&tree, max_num_cuts);
            let actual = tree.min_max_weight(max_num_cuts);
            if actual != expected {
                println!("{}with {} cuts", tree, max_num_cuts);
            }
            assert_eq!(actual, expected);
            assert!(tree.num_cuts() <= max_num_cuts);
            assert!(tree.max_region_weight() == actual);
        }
    }
}
