use crate::tree::{Tree, Weight};

#[derive(Debug, Clone)]
struct FewestCuts {
    num_cuts: u32,
    remaining_weight: Weight,
}

impl Tree {
    pub fn min_max_weight(&mut self, max_num_cuts: u32) -> Weight {
        let mut lower_bound = 0; // impossible
        let mut upper_bound = self.total_weight; // possible
        while lower_bound + 1 < upper_bound {
            let mid = lower_bound + (upper_bound - lower_bound) / 2;
            let num_cuts = fewest_cuts(self, mid).map(|partition| partition.num_cuts);
            if num_cuts.is_some() && num_cuts.unwrap() <= max_num_cuts {
                upper_bound = mid;
            } else {
                lower_bound = mid;
            }
        }
        fewest_cuts(self, upper_bound); // make the cuts
        upper_bound
    }
}

fn fewest_cuts(tree: &mut Tree, max_weight: Weight) -> Option<FewestCuts> {
    let mut child_partitions = Vec::new();
    for (i, child) in tree.children.iter_mut().enumerate() {
        let partition = fewest_cuts(child, max_weight)?;
        child_partitions.push((i, partition));
    }
    child_partitions.sort_by_key(|(_, partition)| partition.remaining_weight);

    let mut weight = tree.weight;
    let mut num_cuts = 0;
    for (i, partition) in child_partitions {
        num_cuts += partition.num_cuts;
        if partition.remaining_weight + weight <= max_weight {
            tree.children[i].is_cut = false;
            weight += partition.remaining_weight;
        } else {
            tree.children[i].is_cut = true;
            num_cuts += 1;
        }
    }
    if weight > max_weight {
        None
    } else {
        Some(FewestCuts {
            num_cuts,
            remaining_weight: weight,
        })
    }
}

#[test]
fn test_min_max_weight() {
    use crate::oracle::oracle_min_max_weight;

    for mut tree in Tree::all_up_to_weight(9) {
        for max_num_cuts in 1..4 {
            let expected = oracle_min_max_weight(&tree, max_num_cuts);
            let actual = tree.min_max_weight(max_num_cuts);
            if actual != expected {
                println!("{}with {} cuts", tree, max_num_cuts);
            }
            assert_eq!(actual, expected);
            assert!(tree.num_cuts() <= max_num_cuts);
        }
    }
}
