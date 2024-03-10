#![allow(unused)]

mod generator;
mod max_of_min;
mod min_of_max;
mod oracle;
mod tree;

use clap::{Parser, ValueEnum};
use oracle::{oracle_max_min_weight, oracle_min_max_weight};
use std::fmt;
use tree::Tree;

#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    Minimax,
    Maximin,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Minimax => write!(f, "minimax"),
            Mode::Maximin => write!(f, "maximin"),
        }
    }
}

/// Cut a random tree into pieces of equalish size
#[derive(Debug, Clone, Parser)]
#[command(version, about)]
struct CommandLineArgs {
    /// How many nodes the tree should have.
    #[arg(long, default_value_t = 30)]
    size: u32,

    /// How many cuts to make.
    #[arg(long, default_value_t = 5)]
    cuts: u32,

    /// How much to branch (maximum number of children per vertex).
    #[arg(long, default_value_t = 4)]
    branching: u32,

    #[arg(default_value_t = Mode::Minimax)]
    mode: Mode,

    /// Random number seed 0..255. Randomly chosen if not specified.
    #[arg(long)]
    seed: Option<u8>,

    /// Flag to compare against the oracle implementation. Be warned: the oracle tries all
    /// possible cuts, so its running time is O(2^size)!
    #[arg(long)]
    oracle: bool,
}

fn main() {
    let args = CommandLineArgs::parse();

    let seed = args.seed.unwrap_or_else(rand::random);
    let mut tree = Tree::random_of_size(args.size, args.branching, seed)
        .next()
        .unwrap();

    let result = match args.mode {
        Mode::Minimax => {
            let max_weight = tree.min_max_weight(args.cuts);
            assert_eq!(max_weight, tree.max_region_weight());
            assert!(tree.num_cuts() <= args.cuts);
            if args.oracle {
                assert_eq!(max_weight, oracle_min_max_weight(&tree, args.cuts));
                println!("(oracle agrees)")
            }
            max_weight
        }
        Mode::Maximin => {
            let min_weight = tree.max_min_weight(args.cuts);
            assert_eq!(min_weight, tree.min_region_weight());
            assert!(tree.num_cuts() >= args.cuts);
            if args.oracle {
                assert_eq!(min_weight, oracle_max_min_weight(&tree, args.cuts));
                println!("(oracle agrees)")
            }
            min_weight
        }
    };

    println!("{}", tree);
    println!("Tree size: {} nodes", tree.size);
    println!("Number of cuts: {}", tree.num_cuts());
    println!("Total weight: {}", tree.total_weight);
    println!(
        "Average weight per region: {}",
        tree.total_weight / (args.cuts as u64 + 1)
    );
    match args.mode {
        Mode::Minimax => println!("Max weight of any region: {}", result),
        Mode::Maximin => println!("Min weight of any region: {}", result),
    }
}
