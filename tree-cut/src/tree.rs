use crate::generator::{Generator, Picker};
use std::fmt;

/*************************************
 *             Trees                 *
 *************************************/

pub type Weight = u64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tree {
    pub size: u32,
    pub weight: Weight,
    pub total_weight: Weight,
    pub children: Vec<Tree>,
    // is edge to parent cut?
    pub is_cut: bool,
    // only used by max_of_min
    pub cut_child_index: Option<usize>,
}

impl Tree {
    pub fn new(weight: Weight, children: Vec<Tree>) -> Tree {
        let size = children.iter().map(|child| child.size).sum::<u32>() + 1;
        let total_weight = children
            .iter()
            .map(|child| child.total_weight)
            .sum::<Weight>()
            + weight;
        Tree {
            size,
            weight,
            total_weight,
            children,
            is_cut: false,
            cut_child_index: None,
        }
    }

    /// Cut this tree off from its parent. Meaningless for the root node.
    pub fn cut(mut self) -> Self {
        self.is_cut = true;
        self
    }

    /// Don't cut this tree off from its parent. Meaningless for the root node.
    pub fn uncut(mut self) -> Self {
        self.is_cut = false;
        self
    }

    /// Total number of cuts in the tree, ignoring the root's potential cut.
    pub fn num_cuts(&self) -> u32 {
        let mut num_cuts = 0;
        for child in &self.children {
            num_cuts += child.num_cuts();
            if child.is_cut {
                num_cuts += 1;
            }
        }
        num_cuts
    }

    /// All possible partitions (sets of cuts) of this tree.
    pub fn all_partitions(&self) -> Vec<Tree> {
        self.all_partitions_rec(true)
    }

    fn all_partitions_rec(&self, at_root: bool) -> Vec<Tree> {
        let mut options = if at_root {
            vec![self.clone().uncut()]
        } else {
            vec![self.clone().uncut(), self.clone().cut()]
        };
        for (i, child) in self.children.iter().enumerate() {
            let mut new_options = Vec::new();
            for child_partition in child.all_partitions_rec(false) {
                for option in &options {
                    let mut new_option = option.clone();
                    new_option.children[i] = child_partition.clone();
                    new_options.push(new_option);
                }
            }
            options = new_options;
        }
        options
    }

    /// The minimum weight of any region.
    pub fn min_region_weight(&self) -> Weight {
        let (min, remaining) = self.min_region_weight_rec();
        min.min(remaining)
    }

    // Compute (min weight of any region, remaining weight at root)
    fn min_region_weight_rec(&self) -> (Weight, Weight) {
        let mut min = Weight::MAX;
        let mut remaining = self.weight;

        for child in &self.children {
            let (child_min, child_remaining) = child.min_region_weight_rec();
            min = min.min(child_min);
            remaining += child_remaining;
        }
        if self.is_cut {
            min = min.min(remaining);
            remaining = 0;
        }
        (min, remaining)
    }

    /// The maximum weight of any region.
    pub fn max_region_weight(&self) -> Weight {
        let (max, remaining) = self.max_region_weight_rec();
        max.max(remaining)
    }

    // Compute (max weight of any region, remaining weight at root)
    fn max_region_weight_rec(&self) -> (Weight, Weight) {
        let mut max = 0;
        let mut remaining = self.weight;

        for child in &self.children {
            let (child_max, child_remaining) = child.max_region_weight_rec();
            max = max.max(child_max);
            remaining += child_remaining;
        }
        if self.is_cut {
            max = max.max(remaining);
            remaining = 0;
        }
        (max, remaining)
    }
}

#[cfg(test)]
fn leaf(weight: Weight) -> Tree {
    Tree::new(weight, Vec::new())
}

#[cfg(test)]
fn branch_1(weight: Weight, child: Tree) -> Tree {
    Tree::new(weight, vec![child])
}

#[cfg(test)]
fn branch_2(weight: Weight, child_1: Tree, child_2: Tree) -> Tree {
    Tree::new(weight, vec![child_1, child_2])
}

#[cfg(test)]
fn branch_3(weight: Weight, child_1: Tree, child_2: Tree, child_3: Tree) -> Tree {
    Tree::new(weight, vec![child_1, child_2, child_3])
}

#[cfg(test)]
fn testing_tree() -> Tree {
    branch_2(
        1,
        branch_3(2, leaf(4), leaf(5).cut(), leaf(6)),
        branch_1(3, leaf(7)).cut(),
    )
}

#[cfg(test)]
fn testing_tree_uncut() -> Tree {
    branch_2(
        1,
        branch_3(2, leaf(4), leaf(5), leaf(6)),
        branch_1(3, leaf(7)),
    )
}

#[test]
fn test_tree_num_cuts() {
    assert_eq!(testing_tree().num_cuts(), 2);
}

#[test]
fn test_all_tree_partitions() {
    assert_eq!(testing_tree_uncut().all_partitions().len(), 64);
}

#[test]
fn test_min_region_weight() {
    assert_eq!(testing_tree().min_region_weight_rec(), (5, 13));
    assert_eq!(testing_tree().min_region_weight(), 5);
}

#[test]
fn test_max_region_weight() {
    assert_eq!(testing_tree().max_region_weight_rec(), (10, 13));
    assert_eq!(testing_tree().max_region_weight(), 13);
}

#[test]
fn test_total_weight() {
    assert_eq!(testing_tree().total_weight, 28);
}

/*************************************
 *        Printing Trees             *
 *************************************/

impl Tree {
    fn display(&self, f: &mut fmt::Formatter, indentation: String) -> fmt::Result {
        writeln!(
            f,
            "{}● {}",
            if self.is_cut { "x" } else { "─" },
            self.weight
        )?;
        for (i, child) in self.children.iter().enumerate() {
            if i < self.children.len() - 1 {
                write!(f, "{}├", indentation)?;
                child.display(f, indentation.clone() + "│ ")?;
            } else {
                write!(f, "{}╰", indentation)?;
                child.display(f, indentation.clone() + "  ")?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.display(f, " ".to_owned())
    }
}

#[test]
fn test_tree_display() {
    assert_eq!(
        format!("{}", testing_tree()),
        [
            // force rustfmt
            "─● 1",
            " ├─● 2",
            " │ ├─● 4",
            " │ ├x● 5",
            " │ ╰─● 6",
            " ╰x● 3",
            "   ╰─● 7",
            ""
        ]
        .join("\n")
    );
}

/*************************************
 *         Random Trees              *
 *************************************/

impl Tree {
    pub fn all_up_to_weight(size: u32) -> impl Iterator<Item = Tree> {
        use crate::generator::generate_all_up_to_size;
        generate_all_up_to_size(TreeGenerator, size)
    }

    pub fn all_of_weight(size: u32) -> impl Iterator<Item = Tree> {
        use crate::generator::generate_all_of_size;
        generate_all_of_size(TreeGenerator, size)
    }

    pub fn random_of_size(size: u32) -> impl Iterator<Item = Tree> {
        use crate::generator::generate_random;
        generate_random(BigTreeGenerator, size, [0; 32])
    }
}

#[derive(Clone, Copy)]
struct TreeGenerator;

impl Generator for TreeGenerator {
    type Value = Tree;

    fn generate<P: Picker>(&self, mut size: u32, picker: &mut P) -> Tree {
        assert_ne!(size, 0);
        let weight = picker.pick_int(size) + 1;
        size -= weight;

        // Divvy `size` out to any number of children.
        let mut children = vec![];
        while size > 0 {
            let child_weight = picker.pick_int(size) + 1;
            size -= child_weight as u32;
            children.push(TreeGenerator.generate(child_weight, picker));
        }
        Tree::new(weight as Weight, children)
    }
}

#[derive(Clone, Copy)]
struct BigTreeGenerator;

impl Generator for BigTreeGenerator {
    type Value = Tree;

    fn generate<P: Picker>(&self, mut size: u32, picker: &mut P) -> Tree {
        assert_ne!(size, 0);
        let weight = picker.pick_int(100);
        size -= 1;

        if size == 0 {
            return Tree::new(weight as Weight, Vec::new());
        }

        let max_num_children = picker.pick_int(4.min(size)) + 1;
        let mut indices = Vec::new();
        if size > 1 {
            for _ in 0..max_num_children - 1 {
                indices.push(picker.pick_int(size - 1) + 1);
            }
        }
        indices.sort();

        let mut children = Vec::new();
        let mut i = 0;
        let mut total_children_size = 0;
        for index in indices {
            if index - i > 0 {
                children.push(BigTreeGenerator.generate(index - i, picker));
                total_children_size += index - i;
            }
            i = index;
        }
        if size - i > 0 {
            children.push(BigTreeGenerator.generate(size - i, picker));
            total_children_size += size - i;
        }
        assert_eq!(total_children_size, size);

        Tree::new(weight as Weight, children)
    }
}

#[test]
fn test_tree_generator() {
    let trees = Tree::all_of_weight(5);
    assert_eq!(trees.count(), 51);
}
