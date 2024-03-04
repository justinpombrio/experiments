#![allow(unused)]

mod generator;
mod min_of_max;
mod tree;

#[cfg(test)]
mod oracle;

use tree::Tree;

const NUM_CUTS: u32 = 10_000;
const TREE_SIZE: u32 = 1_000_000;

fn main() {
    let mut tree = Tree::random_of_size(TREE_SIZE).next().unwrap();
    let max_weight = tree.min_max_weight(NUM_CUTS);
    assert_eq!(max_weight, tree.max_region_weight());
    assert!(tree.num_cuts() <= NUM_CUTS);
    println!("{}", tree);
    println!("Tree size: {} nodes", tree.size);
    println!("Number of cuts: {}", tree.num_cuts());
    println!("Total weight: {}", tree.total_weight);
    println!(
        "Naive lower bound on max weight of any region: {}",
        tree.total_weight / (NUM_CUTS + 1)
    );
    println!("Max weight of any region: {}", max_weight);
}
