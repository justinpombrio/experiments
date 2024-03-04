use crate::generator::{Generator, Picker};
use std::fmt;

/*************************************
 *             Trees                 *
 *************************************/

pub type Weight = u32;

#[derive(Debug, Clone)]
pub struct Tree {
    pub size: u32,
    pub weight: Weight,
    pub total_weight: Weight,
    pub children: Vec<Tree>,
    // is edge to parent cut?
    pub is_cut: bool,
}

impl Tree {
    pub fn new(weight: Weight, children: Vec<Tree>) -> Tree {
        let size = children.iter().map(|child| child.size).sum::<u32>() + 1;
        let total_weight = children.iter().map(|child| child.weight).sum();
        Tree {
            size,
            weight,
            total_weight,
            children,
            is_cut: false,
        }
    }

    pub fn cut(mut self) -> Self {
        self.is_cut = true;
        self
    }

    pub fn num_cuts(&self) -> u32 {
        let mut num_cuts = 0;
        if self.is_cut {
            num_cuts += 1;
        }
        for child in &self.children {
            num_cuts += child.num_cuts();
        }
        num_cuts
    }

    pub fn all_cuts(&self) -> Vec<Tree> {
        let mut options = vec![self.clone(), self.clone().cut()];
        for (i, child) in self.children.iter().enumerate() {
            let mut new_options = Vec::new();
            for cut_child in child.all_cuts() {
                for option in &options {
                    let mut new_option = option.clone();
                    new_option.children[i] = cut_child.clone();
                    new_options.push(new_option);
                }
            }
            options = new_options;
        }
        options
    }

    /// Compute (max weight of any cuttree, remaining weight at root)
    pub fn max_cuttree_weight(&self) -> (Weight, Weight) {
        let mut max = 0;
        let mut remaining = self.weight;

        for child in &self.children {
            let (child_max, child_remaining) = child.max_cuttree_weight();
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
fn testing_tree() -> Tree {
    fn leaf(weight: Weight) -> Tree {
        Tree::new(weight, Vec::new())
    }
    Tree::new(
        1,
        vec![
            Tree::new(2, vec![leaf(4), leaf(5).cut(), leaf(6)]),
            Tree::new(3, vec![leaf(7)]).cut(),
        ],
    )
}

#[cfg(test)]
fn testing_tree_uncut() -> Tree {
    fn leaf(weight: Weight) -> Tree {
        Tree::new(weight, Vec::new())
    }
    Tree::new(
        1,
        vec![
            Tree::new(2, vec![leaf(4), leaf(5), leaf(6)]),
            Tree::new(3, vec![leaf(7)]),
        ],
    )
}

#[test]
fn test_tree_num_cuts() {
    assert_eq!(testing_tree().num_cuts(), 2);
}

#[test]
fn test_all_tree_cuts() {
    assert_eq!(testing_tree_uncut().all_cuts().len(), 128);
}

#[test]
fn test_max_cuttree_weight() {
    assert_eq!(testing_tree().max_cuttree_weight(), (10, 13));
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

#[derive(Clone, Copy)]
pub struct TreeGenerator;

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
            size -= child_weight;
            children.push(TreeGenerator.generate(child_weight, picker));
        }
        Tree::new(weight, children)
    }
}

#[test]
fn test_tree_generator() {
    use crate::generator::generate_all_of_size;

    let trees = generate_all_of_size(TreeGenerator, 5).collect::<Vec<_>>();
    // for tree in &trees {
    //     println!("{}", tree);
    // }
    assert_eq!(trees.len(), 51);

    // use crate::generator::generate_random;
    // let trees = generate_random(TreeGenerator, 1000, [0; 32]);
    // for tree in trees.take(100) {
    //     println!("{}", tree);
    // }
    // assert!(false);
}
