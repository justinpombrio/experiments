use crate::tree::{Tree, Weight};

/// When making `num_cuts` cuts to this tree,
/// what is the smallest possible maximum region weight
/// and largest possible minimum region weight?
pub fn oracle(tree: &Tree, num_cuts: u32) -> (Weight, Weight) {
    let mut min_max_weight = Weight::MAX;
    let mut max_min_weight = 0;
    for partition in tree.all_partitions() {
        if partition.num_cuts() <= num_cuts {
            min_max_weight = min_max_weight.min(partition.max_region_weight());
        }
        if partition.num_cuts() >= num_cuts {
            max_min_weight = max_min_weight.max(partition.min_region_weight());
        }
    }
    (min_max_weight, max_min_weight)
}
