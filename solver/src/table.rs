// TODO: temporary
#![allow(unused)]

use crate::ring::Ring;
use crate::state::State;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Table<S: State> {
    sections: Vec<Section<S>>,
}

#[derive(Debug)]
struct Section<S: State> {
    header: Vec<S::Var>,
    tuples: Vec<Vec<S::Value>>,
}

impl<S: State> Table<S> {
    pub fn new() -> Table<S> {
        Table {
            sections: Vec::new(),
        }
    }

    pub fn add_column(&mut self, x: S::Var, vals: impl IntoIterator<Item = S::Value>) {
        let vals = vals.into_iter().collect::<Vec<_>>();
        assert!(vals.len() > 0);
        self.sections.push(Section {
            header: vec![x],
            tuples: map_vec(vals, |v| vec![v]),
        });
    }

    pub fn apply_ring_constraint<R: Ring>(
        &mut self,
        header: &[S::Var],
        map: impl Fn(S::Var, S::Value) -> R,
        pred: impl Fn(R) -> bool,
    ) {
        let partial_sums = self.apply_ring(header, map);

        let mut total = R::one();
        for (_, sum, _) in &partial_sums {
            total = total.mul(sum.clone());
        }

        let mut keep_lists: Vec<(usize, Vec<bool>)> = Vec::new();
        for (i, sum, prods) in partial_sums {
            let keep_list = map_vec(prods, |prod| pred(total.clone().div(sum.clone()).mul(prod)));
            keep_lists.push((i, keep_list));
        }
        self.retain(keep_lists);
    }

    pub fn size(&self) -> u64 {
        let mut size = 0;
        for section in &self.sections {
            size += section.tuples.len() as u64;
        }
        size
    }

    pub fn possibilities(&self) -> u64 {
        let mut possibilities = 1;
        for section in &self.sections {
            possibilities *= section.tuples.len() as u64;
        }
        possibilities
    }

    pub fn num_sections(&self) -> usize {
        self.sections.len()
    }

    pub fn merge(&mut self, section_1: usize, section_2: usize) {
        assert!(section_2 > section_1);
        let section_2 = self.sections.swap_remove(section_2);
        let section_1 = self.sections.swap_remove(section_1);

        let mut header = section_1.header;
        header.extend(section_2.header);

        let mut tuples = Vec::new();
        for tuple_1 in section_1.tuples {
            for tuple_2 in &section_2.tuples {
                let mut tuple = tuple_1.clone();
                tuple.extend(tuple_2.clone());
                tuples.push(tuple);
            }
        }

        self.sections.push(Section { header, tuples });
    }

    pub fn state(&self) -> HashMap<S::Var, S::Value> {
        let mut map = HashMap::new();
        for section in &self.sections {
            for (i, x) in section.header.iter().enumerate() {
                let v = &section.tuples[0][i];
                let mut v_varies = false;
                for tuple in &section.tuples {
                    if &tuple[i] != v {
                        v_varies = true;
                    }
                }
                if !v_varies {
                    map.insert(x.clone(), v.clone());
                }
            }
        }
        map
    }

    fn project(&self, header: &[S::Var]) -> Vec<(usize, Section<S>)> {
        let mut sections = Vec::new();
        for (section_index, section) in self.sections.iter().enumerate() {
            if let Some(subsection) = section.project(header) {
                sections.push((section_index, subsection));
            }
        }
        sections
    }

    fn retain(&mut self, keep_lists: Vec<(usize, Vec<bool>)>) {
        for (section_index, keep_list) in keep_lists {
            self.sections[section_index].retain(keep_list);
        }
    }

    fn apply_ring<R: Ring>(
        &self,
        header: &[S::Var],
        map: impl Fn(S::Var, S::Value) -> R,
    ) -> Vec<(usize, R, Vec<R>)> {
        let mut result = Vec::new();
        for (i, subsection) in self.project(header) {
            let (sum, prods) = subsection.apply_ring(&map);
            result.push((i, sum, prods));
        }
        result
    }
}

impl<S: State> Section<S> {
    fn project(&self, header: &[S::Var]) -> Option<Section<S>> {
        let (subheader, mapping) = project_header::<S>(&self.header, header)?;
        let subtuples = map_vec(&self.tuples, |tuple| {
            map_vec(&mapping, |i| tuple[*i].clone())
        });
        Some(Section {
            header: subheader,
            tuples: subtuples,
        })
    }

    fn retain(&mut self, keep_list: Vec<bool>) {
        assert_eq!(self.tuples.len(), keep_list.len());
        for (i, keep) in keep_list.iter().enumerate().rev() {
            if !keep {
                self.tuples.swap_remove(i);
            }
        }
    }

    fn apply_ring<R: Ring>(&self, map: impl Fn(S::Var, S::Value) -> R) -> (R, Vec<R>) {
        let tuple_prod = |tuple: &Vec<S::Value>| -> R {
            let mut prod = R::one();
            for (i, val) in tuple.iter().enumerate() {
                prod = prod.mul(map(self.header[i].clone(), val.clone()));
            }
            prod
        };

        let products = map_vec(&self.tuples, tuple_prod);

        let mut sum = products[0].clone();
        for prod in products.iter().skip(1) {
            sum = sum.add(prod.clone());
        }
        (sum, products)
    }
}

/// Let `subheader` be the intersection of `header_1` and `header_2`. If `subheader` is empty,
/// return None. Otherwise return `(subheader, mapping)`, where `subheader[i] =
/// header_1[mapping[i]]`.
fn project_header<S: State>(
    header_1: &[S::Var],
    header_2: &[S::Var],
) -> Option<(Vec<S::Var>, Vec<usize>)> {
    let (subheader, mapping) = header_2
        .iter()
        .filter_map(|x| header_1.iter().position(|y| y == x).map(|i| (x.clone(), i)))
        .unzip::<_, _, Vec<_>, Vec<_>>();

    if subheader.is_empty() {
        None
    } else {
        Some((subheader, mapping))
    }
}

fn map_vec<A, B>(vec: impl IntoIterator<Item = A>, f: impl Fn(A) -> B) -> Vec<B> {
    vec.into_iter().map(f).collect::<Vec<_>>()
}

impl<S: State> Clone for Section<S> {
    fn clone(&self) -> Section<S> {
        Section {
            header: self.header.clone(),
            tuples: self.tuples.clone(),
        }
    }
}

impl<S: State> Clone for Table<S> {
    fn clone(&self) -> Table<S> {
        Table {
            sections: self.sections.clone(),
        }
    }
}
