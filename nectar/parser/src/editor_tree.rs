use std::iter;

use self::Tree::*;
use self::Context::*;


pub enum Tree<L, B> {
    Leaf(L),
    Branch(B, Vec<Tree<L, B>>)
}

pub enum Context<L, B> {
    Top,
    Ctx(B, Vec<Tree<L, B>>, Box<Context<L, B>>, Vec<Tree<L, B>>)
}

struct Loc<L, B> {
    tree:    Tree<L, B>,
    context: Context<L, B>
}

impl<L, B> Tree<L, B> {
    fn top(self) -> Loc<L, B> {
        Loc::new(self, Top)
    }
}

impl<L, B> Loc<L, B> {
    fn new(tree: Tree<L, B>, context: Context<L, B>) -> Loc<L, B> {
        Loc{
            tree: tree,
            context: context
        }
    }

    pub fn up(self) -> Loc<L, B> {
        match self.context {
            Top =>
                Loc::new(self.tree, Top),
            Ctx(op, left, ctx, right) => {
                let v = left.into_iter()
                    .chain(iter::once(self.tree))
                    .chain(right.into_iter().rev())
                    .collect();
                Loc::new(Branch(op, v), *ctx)
            }
        }
    }

    pub fn down(self) -> Loc<L, B> {
        let ctx = self.context;
        match self.tree {
            Leaf(leaf) => Loc::new(Leaf(leaf), ctx),
            Branch(op, mut trees) => {
                trees.reverse();
                match trees.pop() {
                    None =>
                        Loc::new(Branch(op, trees), ctx),
                    Some(tree) =>
                        Loc::new(tree, Ctx(op, vec!(), Box::new(ctx), trees))
                }
            }
        }
    }

    pub fn left(self) -> Loc<L, B> {
        match self.context {
            Top => Loc::new(self.tree, Top),
            Ctx(op, mut left, ctx, mut right) =>
                match left.pop() {
                    None =>
                        Loc::new(self.tree, Ctx(op, left, ctx, right)),
                    Some(tree) => {
                        right.push(self.tree);
                        Loc::new(tree, Ctx(op, left, ctx, right))
                    }
                }
        }
    }

    pub fn right(self) -> Loc<L, B> {
        match self.context {
            Top => Loc::new(self.tree, Top),
            Ctx(op, mut left, ctx, mut right) =>
                match right.pop() {
                    None =>
                        Loc::new(self.tree, Ctx(op, left, ctx, right)),
                    Some(tree) => {
                        left.push(self.tree);
                        Loc::new(tree, Ctx(op, left, ctx, right))
                    }
                }
        }
    }
}
