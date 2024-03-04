use crate::tree::{Tree, TreeGenerator, Weight};

#[derive(Debug, Clone)]
struct BestMaxCuts {
    impossible: bool,
    num_cuts: u32,
    remaining_weight: Weight,
}

impl Tree {
    pub fn min_max(&mut self, max_weight: Weight) -> Option<u32> {
        let cuts = best_max_cuts(self, max_weight);
        if cuts.impossible {
            None
        } else {
            Some(cuts.num_cuts)
        }
    }
}

fn best_max_cuts(tree: &mut Tree, max_weight: Weight) -> BestMaxCuts {
    let mut best_child_cuts = tree
        .children
        .iter_mut()
        .map(|child| best_max_cuts(child, max_weight))
        .enumerate()
        .collect::<Vec<_>>();
    best_child_cuts.sort_by_key(|(_, cuts)| cuts.remaining_weight);

    let mut impossible = false;
    let mut weight = tree.weight;
    let mut num_cuts = 0;
    for (i, cuts) in best_child_cuts {
        num_cuts += cuts.num_cuts;
        if cuts.impossible {
            impossible = true;
        }
        if cuts.remaining_weight + weight <= max_weight {
            weight += cuts.remaining_weight;
        } else {
            tree.children[i].is_cut = true;
            num_cuts += 1;
        }
    }
    if weight > max_weight {
        impossible = true;
    }

    BestMaxCuts {
        impossible,
        num_cuts,
        remaining_weight: weight,
    }
}

fn oracle_max_cuts(tree: &Tree, max_weight: Weight) -> Option<u32> {
    let mut min_cuts_req: Option<u32> = None;
    for cut_tree in tree.all_cuts() {
        let (max, remaining) = cut_tree.max_cuttree_weight();
        if max.max(remaining) <= max_weight {
            if let Some(min) = min_cuts_req {
                min_cuts_req = Some(min.min(cut_tree.num_cuts()));
            } else {
                min_cuts_req = Some(cut_tree.num_cuts());
            }
        }
    }
    min_cuts_req
}

#[test]
fn test_best_max_cuts() {
    use crate::generator::generate_all_up_to_size;

    let max_weight = 3;
    for mut tree in generate_all_up_to_size(TreeGenerator, 10) {
        let expected = oracle_max_cuts(&tree, max_weight);
        let actual = tree.min_max(max_weight);
        if actual != expected {
            println!("{}", tree);
        }
        assert_eq!(actual, expected);
    }
}
