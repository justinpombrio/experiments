use crate::tree::{Tree, Weight};

#[derive(Debug, Clone)]
struct FewestCuts {
    num_cuts: u32,
    remaining_weight: Weight,
}

impl Tree {
    pub fn min_max_weight(&mut self, max_cuts: u32) -> Weight {
        let mut lower_bound = 0;
        let mut upper_bound = self.total_weight;
        while lower_bound < upper_bound {
            let mid = lower_bound + (upper_bound - lower_bound) / 2;
            let num_cuts = fewest_cuts(self, mid).map(|cuts| cuts.num_cuts);
            if num_cuts.is_some() && num_cuts.unwrap() <= max_cuts {
                upper_bound = mid;
            } else {
                lower_bound = mid + 1;
            }
        }
        fewest_cuts(self, upper_bound); // make the cuts
        upper_bound
    }
}

fn fewest_cuts(tree: &mut Tree, max_weight: Weight) -> Option<FewestCuts> {
    let mut child_cuts = Vec::new();
    for (i, child) in tree.children.iter_mut().enumerate() {
        let cuts = fewest_cuts(child, max_weight)?;
        child_cuts.push((i, cuts));
    }
    child_cuts.sort_by_key(|(_, cuts)| cuts.remaining_weight);

    let mut weight = tree.weight;
    let mut num_cuts = 0;
    for (i, cuts) in child_cuts {
        num_cuts += cuts.num_cuts;
        if cuts.remaining_weight + weight <= max_weight {
            tree.children[i].is_cut = false;
            weight += cuts.remaining_weight;
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
    use crate::oracle::oracle;

    for mut tree in Tree::all_up_to_weight(9) {
        for max_cuts in 1..4 {
            let expected = oracle(&tree, max_cuts).0;
            let actual = tree.min_max_weight(max_cuts);
            if actual != expected {
                println!("{}with {} cuts", tree, max_cuts);
            }
            assert_eq!(actual, expected);
            assert!(tree.num_cuts() <= max_cuts);
        }
    }
}
