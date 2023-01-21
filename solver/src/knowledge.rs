// TODO: temporary
#![allow(unused)]

use crate::constraint::{Value, Var};
use std::iter;

struct Knowledge<X: Var, V: Value> {
    constants: Vec<(X, V)>,
    // cross product of (domain, union of mapping)
    components: Vec<(Vec<X>, Vec<Vec<V>>)>,
}

#[derive(Debug, Clone, Copy)]
struct Assignment {
    component_index: usize,
    union_index: usize,
}

impl<X: Var, V: Value> Knowledge<X, V> {
    fn fill_constants(&self, params: &[X], assignment: Assignment) -> Vec<Option<V>> {
        let mut constants: Vec<Option<V>> = Vec::new();
        for (x, v) in &self.constants {
            if let Some(i) = params.iter().position(|y| y == x) {
                constants[i] = Some(v.clone());
            }
        }
        let (domain, union) = &self.components[assignment.component_index];
        for x in domain {
            if let Some(i) = params.iter().position(|y| y == x) {
                constants[i] = Some(union[assignment.union_index][i].clone());
            }
        }
        constants
    }

    fn merge(&mut self, comp_1: usize, comp_2: usize) {
        let (domain_1, union_1) = self.components.remove(comp_1);
        let (domain_2, union_2) = self.components.remove(comp_2);

        let mut domain = domain_1;
        domain.extend(domain_2);

        let mut union = Vec::new();
        for map_1 in &union_1 {
            for map_2 in &union_2 {
                union.push(
                    map_1
                        .iter()
                        .chain(map_2.iter())
                        .cloned()
                        .collect::<Vec<_>>(),
                );
            }
        }

        self.components.push((domain, union));
    }

    fn filter(&mut self, comp: usize, pred: impl Fn(&[V]) -> bool) -> Result<(), ()> {
        let (domain, union) = &mut self.components[comp];

        let mut has_changed = false;
        union.retain(|tuple| {
            let retain = pred(tuple);
            if !retain {
                has_changed = true;
            }
            retain
        });

        if !has_changed {
            return Ok(());
        }

        if union.is_empty() {
            return Err(());
        }

        let is_const_list = (0..domain.len())
            .map(|i| union.iter().all(|tuple| tuple[i] == union[0][i]))
            .collect::<Vec<_>>();

        if is_const_list.iter().copied().all(|b| !b) {
            return Ok(());
        }

        for i in 0..domain.len() {
            if is_const_list[i] {
                self.constants
                    .push((domain[i].clone(), union[0][i].clone()));
            }
        }

        if is_const_list.iter().copied().all(|b| b) {
            self.components.remove(comp);
            return Ok(());
        }

        for tuple in union {
            *tuple = tuple
                .drain(..)
                .enumerate()
                .filter(|(i, _v)| is_const_list[*i])
                .map(|(_i, v)| v)
                .collect::<Vec<_>>();
        }

        Ok(())
    }

    // fn apply_constraint(&mut self, constraint: impl Constraint<X, V>) { }
}
