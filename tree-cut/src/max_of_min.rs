use crate::oracle::oracle;
use crate::tree::{Tree, Weight};

#[derive(Debug, Clone)]
struct MostCuts {
    num_cuts: u32,
    remaining_weight: Weight,
}

impl Tree {
    pub fn max_min_weight(&mut self, min_cuts: u32) -> Weight {
        let mut lower_bound = 0;
        let mut upper_bound = Weight::MAX;
        while lower_bound < upper_bound {
            let mid = lower_bound + (upper_bound - lower_bound) / 2;
            let num_cuts = most_cuts(self, mid).map(|cuts| cuts.num_cuts);
            if num_cuts.is_some() && num_cuts.unwrap() >= min_cuts {
                upper_bound = mid;
            } else {
                lower_bound = mid + 1;
            }
        }
        most_cuts(self, upper_bound); // make the cuts
        upper_bound
    }
}

fn most_cuts(tree: &mut Tree, min_weight: Weight) -> Option<MostCuts> {
    let mut child_cuts = Vec::new();
    for (i, child) in tree.children.iter_mut().enumerate() {
        let cuts = most_cuts(child, min_weight)?;
        child_cuts.push((i, cuts));
    }
    child_cuts.sort_by_key(|(_, cuts)| -(cuts.remaining_weight as i64));

    let mut weight = tree.weight;
    let mut num_cuts = 0;
    for (i, cuts) in child_cuts {
        num_cuts += cuts.num_cuts;
        if weight < min_weight {
            tree.children[i].is_cut = false;
            weight += cuts.remaining_weight;
        } else {
            tree.children[i].is_cut = true;
            num_cuts += 1;
        }
    }
    if weight < min_weight {
        None
    } else {
        Some(MostCuts {
            num_cuts,
            remaining_weight: weight,
        })
    }
}

#[test]
fn test_max_min_weight() {
    for mut tree in Tree::all_of_size(8) {
        println!("{:?}", most_cuts(&mut tree, 2).is_some());
        println!("{}", tree);
        /*
        for min_cuts in 1..5 {
            let expected = oracle(&tree, min_cuts).1;
            let actual = tree.max_min_weight(min_cuts);
            assert!(tree.num_cuts() <= min_cuts);
            if actual != expected {
                println!("{}with {} cuts", tree, min_cuts);
            }
            assert_eq!(actual, expected);
        }
        */
    }
    assert!(false);
}
