// TODO: temporary
#![allow(unused)]

use crate::ring::Ring;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::ops::Add;

pub trait Var: fmt::Debug + Hash + Eq + Ord + Clone + 'static {}
pub trait Value: fmt::Debug + PartialEq + Clone + 'static {}

impl<X: fmt::Debug + Hash + Eq + Ord + Clone + 'static> Var for X {}
impl<V: fmt::Debug + PartialEq + Clone + 'static> Value for V {}

struct Constraint<X: Var, V: Value> {
    name: String,
    params: Vec<X>,
    pred: Box<dyn Fn(Vec<(Vec<usize>, Vec<Vec<V>>)>) -> bool>,
}

fn ring_constraint<X: Var, V: Value, R: Ring>(
    name: impl Into<String>,
    params: impl IntoIterator<Item = X>,
    mapping: Vec<(X, R)>,
    pred: impl Fn(R) -> bool + 'static,
) -> Constraint<X, V> {
    let params = params.into_iter().collect::<Vec<_>>();
    // TODO: error on unwrap
    let mut param_mapping = params
        .iter()
        .map(|x| mapping.iter().find(|(x2, _)| x2 == x).unwrap().1.clone())
        .collect::<Vec<_>>();

    let pred = move |known: Vec<(Vec<usize>, Vec<Vec<V>>)>| -> bool {
        let mut accum = R::one();
        for (indices, union) in known {
            let mut accum_union = R::zero();
            for tuple in union {
                let mut accum_tuple = R::one();
                for (i, v) in indices.iter().copied().zip(tuple) {
                    accum_tuple = R::mul(accum_tuple, param_mapping[i].clone());
                }
                accum_union = R::add(accum_union, accum_tuple);
            }
            accum = R::mul(accum, accum_union);
        }
        pred(accum)
    };

    Constraint {
        name: name.into(),
        params,
        pred: Box::new(pred),
    }
}

/*
fn known_constraint<X: Var, V: Value>(
    name: impl Into<String>,
    params: impl IntoIterator<Item = X>,
    pred: impl Fn(&[V]) -> bool + 'static,
) -> Constraint<X, V> {
    let params = params.into_iter().collect::<Vec<_>>();
    let params_copy = params.clone();

    let pred = move |known: Vec<(Vec<usize>, Vec<Vec<V>>)>| -> bool {
        let mut args = iter::repeat(None)
            .take(params.len())
            .collect::<Vec<Option<V>>>();
        for (indices, mut union) in known {
            if union.len() == 1 {
                let tuple = union.remove(0);
                for (i, v) in indices.into_iter().zip(tuple.into_iter()) {
                    args[i] = Some(v);
                }
            } else {
                // We're missing an arg, so the constraint might be true
                return true;
            }
        }
        let args = args
            .into_iter()
            .map(|arg| arg.expect("argument to pred missed an index"))
            .collect::<Vec<_>>();
        pred(&args)
    };

    Constraint {
        name: name.into(),
        params: params_copy,
        pred: Box::new(pred),
    }
}
*/

/*

impl<N: Addable> Ring for SumRing<N> {
    const ZERO: SumRing<N> = SumRing::Empty;
    const ONE: SumRing<N> = SumRing::Range(N::ZERO, N::ZERO);

    fn mul(a: Self, b: Self) -> Self {
        use SumRing::{Empty, Range};

        match (a, b) {
            (Empty, Empty) | (Empty, Range(_, _)) | (Range(_, _), Empty) => Empty,
            (Range(a0, a1), Range(b0, b1)) => Range(a0 + b0, a1 + b1),
        }
    }

    fn add(a: Self, b: Self) -> Self {
        use SumRing::{Empty, Range};

        match (a, b) {
            (Empty, Empty) => Empty,
            (Empty, Range(b0, b1)) => Range(b0, b1),
            (Range(a0, a1), Empty) => Range(a0, a1),
            (Range(a0, a1), Range(b0, b1)) => {
                let c0 = if a0 < b0 { a0 } else { b0 };
                let c1 = if a1 < b1 { a1 } else { b1 };
                Range(c0, c1)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MultisetRing<T: Hash + Eq + Clone>(HashMap<T, i32>);

impl<T: Hash + Eq + Clone> Ring for MultisetRing<T> {
    const ZERO: MultisetRing =

    const ZERO: SumRing<N> = SumRing::Empty;
    const ONE: SumRing<N> = SumRing::Range(N::ZERO, N::ZERO);

    fn mul(a: Self, b: Self) -> Self {
        use SumRing::{Empty, Range};

        match (a, b) {
            (Empty, Empty) | (Empty, Range(_, _)) | (Range(_, _), Empty) => Empty,
            (Range(a0, a1), Range(b0, b1)) => Range(a0 + b0, a1 + b1),
        }
    }

    fn add(a: Self, b: Self) -> Self {
        use SumRing::{Empty, Range};

        match (a, b) {
            (Empty, Empty) => Empty,
            (Empty, Range(b0, b1)) => Range(b0, b1),
            (Range(a0, a1), Empty) => Range(a0, a1),
            (Range(a0, a1), Range(b0, b1)) => {
                let c0 = if a0 < b0 { a0 } else { b0 };
                let c1 = if a1 < b1 { a1 } else { b1 };
                Range(c0, c1)
            }
        }
    }
}
*/
