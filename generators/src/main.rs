struct Gen {
    started: bool,
    v: Vec<(u32, u32)>,
    p: usize,
}

impl Gen {
    fn new() -> Gen {
        Gen {
            started: false,
            v: Vec::new(),
            p: 0,
        }
    }
    fn done(&mut self) -> bool {
        if !self.started {
            self.started = true;
            return false;
        }

        for i in (0..self.v.len()).rev() {
            if self.v[i].0 < self.v[i].1 {
                self.v[i].0 += 1;
                self.v.truncate(i + 1);
                self.p = 0;
                return false;
            }
        }

        true
    }
    fn gen(&mut self, bound: u32) -> u32 {
        if self.p == self.v.len() {
            self.v.push((0, 0));
        }
        self.p += 1;
        self.v[self.p - 1].1 = bound;
        self.v[self.p - 1].0
    }
}

#[derive(Debug)]
enum Tree {
    L,
    B(Box<Tree>, Box<Tree>),
}

fn gen_tree(g: &mut Gen, size: u32) -> Tree {
    if size == 0 {
        return Tree::L;
    }

    let left_size = g.gen(size - 1);
    let right_size = size - left_size - 1;
    Tree::B(
        Box::new(gen_tree(g, left_size)),
        Box::new(gen_tree(g, right_size)),
    )
}

fn main() {
    let size = 3;
    let mut g = Gen::new();
    let mut trees = vec![];
    while !g.done() {
        trees.push(gen_tree(&mut g, size));
    }
    for tree in &trees {
        println!("{:?}", tree);
    }
    println!("{}", trees.len());
}
