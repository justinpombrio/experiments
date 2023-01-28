// TODO: temporary
#![allow(unused)]

use crate::ring::Ring;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::ops::Add;

pub trait Var: fmt::Debug + Hash + Eq + Ord + Clone + 'static {}
pub trait Value: fmt::Debug + Eq + Ord + Clone + 'static {}

impl<X: fmt::Debug + Hash + Eq + Ord + Clone + 'static> Var for X {}
impl<V: fmt::Debug + Eq + Ord + Clone + 'static> Value for V {}

pub struct PartialKnowledge<V: Value>(pub Vec<(Vec<usize>, Vec<Vec<V>>)>);

struct Constraint<X: Var, V: Value> {
    name: String,
    params: Vec<X>,
    pred: Box<dyn Fn(PartialKnowledge<V>) -> bool>,
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

    let tuple_prod = move |indices: &Vec<usize>, tuple: Vec<V>| -> R {
        let mut prod = R::one();
        for (i, v) in indices.iter().copied().zip(tuple) {
            prod = R::mul(prod, param_mapping[i].clone());
        }
        prod
    };

    let pred = move |known: PartialKnowledge<V>| -> bool {
        let mut total = R::one();
        for (indices, union) in known.0 {
            let mut tuples = union.into_iter();
            let mut sum = tuple_prod(&indices, tuples.next().unwrap());
            for tuple in tuples {
                sum = R::add(sum, tuple_prod(&indices, tuple));
            }
            total = R::mul(total, sum);
        }
        pred(total)
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
