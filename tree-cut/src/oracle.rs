use crate::tree::{Tree, Weight};

/// When making at most `max_num_cuts` to this tree,
/// what is the smallest possible maximum region weight?
pub fn oracle_min_max_weight(tree: &Tree, max_num_cuts: u32) -> Weight {
    let mut min_max_weight = Weight::MAX;
    for partition in tree.all_partitions() {
        if partition.num_cuts() <= max_num_cuts {
            min_max_weight = min_max_weight.min(partition.max_region_weight());
        }
    }
    min_max_weight
}

/// When making at least `min_num_cuts` to this tree,
/// what is the largest possible minimum region weight?
pub fn oracle_max_min_weight(tree: &Tree, min_num_cuts: u32) -> Weight {
    let mut max_min_weight = 0;
    for partition in tree.all_partitions() {
        if partition.num_cuts() >= min_num_cuts {
            max_min_weight = max_min_weight.max(partition.min_region_weight());
        }
    }
    max_min_weight
}
