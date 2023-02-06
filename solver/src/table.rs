use crate::constraints::{Constraint, YesNoMaybe};
use crate::state::{display_states, State};
use std::collections::HashMap;
use std::fmt;

/// A state of knowledge about the `Value`s that a set of `Var`s might have, represented as a cross
/// product of unions of tuples.
///
/// You can think of this as a Table made of Sections. For example, this Table:
///
/// ```text
///     A C | B | D E F
///     ----+---+------
///     1 1 | 1 | 7 8 9
///     1 2 | 2 |
///     2 1 | 3 |
///         | 4 |
/// ```
///
/// represents the state of knowledge:
///
///  - A and C are either 1,1 or 1,2 or 2,1 respectively.
///  - B is between 1 and 4 inclusive.
///  - D=7, E=8, and F=9
///
/// The table has three sections `(AC, B, DEF)`, and it represents 12 possible states:
///
/// ```text
///     A C B D E F
///     -----------
///     1 1 1 7 8 9
///     1 2 1 7 8 9
///     2 1 1 7 8 9
///     1 1 2 7 8 9
///     1 2 2 7 8 9
///     2 1 2 7 8 9
///     1 1 3 7 8 9
///     1 2 3 7 8 9
///     2 1 3 7 8 9
///     1 1 4 7 8 9
///     1 2 4 7 8 9
///     2 1 4 7 8 9
/// ```
#[derive(Debug)]
pub struct Table<S: State> {
    sections: Vec<Section<S>>,
}

/// One section of a table.
#[derive(Debug)]
struct Section<S: State> {
    header: Vec<S::Var>,
    tuples: Vec<Vec<S::Value>>,
}

impl<S: State> Table<S> {
    /// Construct an empty table.
    pub fn new() -> Table<S> {
        Table {
            sections: Vec::new(),
        }
    }

    /// Add a new column to the table. It will be its own Section.
    pub fn add_column(&mut self, x: S::Var, vals: impl IntoIterator<Item = S::Value>) {
        let vals = vals.into_iter().collect::<Vec<_>>();
        assert!(vals.len() > 0);
        self.sections.push(Section {
            header: vec![x],
            tuples: map_vec(vals, |v| vec![v]),
        });
    }

    /// `params` names a subset of columns of this table; `map` is a function to apply to the
    /// elements of those columns, and `constraint` is a constraint that those mapped elements must
    /// obey. Remove table rows (tuples) that violate this constraint.
    ///
    /// `Err` if some section runs out of tuples (i.e. number of possibilities becomes zero).
    ///
    /// Returns `Ok(true)` if the constraint _always_ holds, and can thus be dropped. Otherwise
    /// `Ok(false)`.
    pub fn apply_constraint<N, C: Constraint<N>>(
        &mut self,
        params: &[S::Var],
        map: &impl Fn(usize, S::Value) -> N,
        constraint: &C,
    ) -> Result<bool, ()> {
        // For each section#i present in the projection, compute (i, prods, sum)
        // where `prod` is the product(and) of each tuple, and `sum` is the sum(or) of those prods.
        let mut partial_sums = Vec::new();
        for (i, subsection) in self.project(params) {
            let (prods, sum) = subsection.apply_constraint(params, map, constraint);
            partial_sums.push((i, prods, sum));
        }
        assert!(!partial_sums.is_empty());

        // Check if the constraint is guranteed to hold from now on
        let mut total_prod = partial_sums[0].2.clone();
        for i in 1..partial_sums.len() {
            total_prod = constraint.and(total_prod, partial_sums[i].2.clone());
        }
        if constraint.check(total_prod) == YesNoMaybe::Yes {
            return Ok(true);
        }

        // Need to special case the len=1 case because the code below needs at least len=2.
        if partial_sums.len() == 1 {
            let (i, prods, _) = partial_sums.remove(0);
            let keep_list = map_vec(prods.clone(), |prod| {
                constraint.check(prod) != YesNoMaybe::No
            });
            let keep_lists = vec![(i, keep_list)];
            self.retain(keep_lists)?;
            return Ok(false);
        }

        // If the partial sums computed above are `A,B,C,D`, then compute `BCD, CDA, DAB, ABC`.
        let mut all_but_one_prods = Vec::new();
        for i in 0..partial_sums.len() {
            let nth_partial_sum = |j: usize| partial_sums[(i + j) % partial_sums.len()].2.clone();
            let mut prod = nth_partial_sum(1);
            for j in 2..partial_sums.len() {
                prod = constraint.and(prod, nth_partial_sum(j));
            }
            all_but_one_prods.push(prod);
        }

        // For each tuple in each section, combine that tuple's prod with the all_but_one_prod, and
        // check if that obeys the constraint.
        let mut keep_lists: Vec<(usize, Vec<bool>)> = Vec::new();
        for (i, (j, prods, _)) in partial_sums.into_iter().enumerate() {
            let keep_list = map_vec(prods, |prod| {
                let total = constraint.and(all_but_one_prods[i].clone(), prod);
                constraint.check(total) != YesNoMaybe::No
            });
            keep_lists.push((j, keep_list));
        }

        // Apply the keep_lists, discarding tuples that violate the constraint.
        self.retain(keep_lists)?;
        Ok(false)
    }

    /// A measure of the size of this table: the sum of the number of rows in each section. The
    /// number of possibilities is the _product_ of the number of rows in each section, so the
    /// `size` can be exponentially smaller.
    pub fn size(&self) -> u64 {
        let mut size = 0;
        for section in &self.sections {
            size += section.tuples.len() as u64;
        }
        size
    }

    /// The total number of possible states that have not yet been ruled out.
    pub fn possibilities(&self) -> u128 {
        let mut possibilities = 1;
        for section in &self.sections {
            possibilities *= section.tuples.len() as u128;
        }
        possibilities
    }

    /// The total number of columns this table has.
    pub fn num_columns(&self) -> usize {
        self.sections.iter().map(|s| s.header.len()).sum()
    }

    /// The number of sections this table has.
    pub fn num_sections(&self) -> usize {
        self.sections.len()
    }

    /// Merge all constant sections (those of height 1) together.
    pub fn merge_constants(&mut self) {
        let mut const_secs = Vec::new();
        for (i, sec) in self.sections.iter().enumerate() {
            if sec.tuples.len() == 1 {
                const_secs.push(i);
            }
        }
        if const_secs.len() <= 1 {
            return;
        }

        for i in (0..const_secs.len() - 1).rev() {
            // Relying on the merge being put back at index `const_secs[i]`!
            self.merge(const_secs[i], const_secs[i + 1]);
        }
    }

    /// Merge two table sections (identified by index) together. Places the merged section at the
    /// index `sec_1.min(sec_2)`.
    pub fn merge(&mut self, sec_1: usize, sec_2: usize) {
        let (sec_1, sec_2) = (sec_1.min(sec_2), sec_1.max(sec_2));
        let section_2 = self.sections.remove(sec_2);
        let section_1 = self.sections.remove(sec_1);

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

        self.sections.insert(sec_1, Section { header, tuples });
    }

    // TODO: remove?
    pub fn state(&self) -> HashMap<S::Var, S::Value> {
        let mut map = HashMap::new();
        for section in &self.sections {
            if section.tuples.is_empty() {
                continue;
            }
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

    /// Construct new sections that are limited to the columns present in `params` and also in
    /// `self`. Return these new sections together with the index of the section they came from.
    /// Each new section has the same number of tuples, in the same order, as the section it came
    /// from. (This way a `keep_list` constructed from the new section can safely be applied to the
    /// original section.)
    fn project(&self, params: &[S::Var]) -> Vec<(usize, Section<S>)> {
        let mut sections = Vec::new();
        for (section_index, section) in self.sections.iter().enumerate() {
            if let Some(subsection) = section.project(params) {
                sections.push((section_index, subsection));
            }
        }
        sections
    }

    /// Discard tuples such that:
    ///
    /// ```text
    ///     for some (i, keep_list) in keep_lists:
    ///         self.sections[i].tuples[j] not in keep_list
    /// ```
    ///
    /// `Err` iff any tuple list becomes empty (i.e. `possibilities()` becomes 0).
    fn retain(&mut self, keep_lists: Vec<(usize, Vec<bool>)>) -> Result<(), ()> {
        for (section_index, keep_list) in keep_lists {
            self.sections[section_index].retain(keep_list)?;
        }
        Ok(())
    }
}

impl<S: State> Section<S> {
    /// Construct a `Section` using only the columns present in `params`. Return `None` if there
    /// would be zero columns.
    fn project(&self, params: &[S::Var]) -> Option<Section<S>> {
        let (subheader, mapping) = project_header::<S>(&self.header, params)?;
        let subtuples = map_vec(&self.tuples, |tuple| {
            map_vec(&mapping, |i| tuple[*i].clone())
        });
        Some(Section {
            header: subheader,
            tuples: subtuples,
        })
    }

    /// Retain the `i`'th tuple iff `keep_list[i]`. `Err` if no tuples remain.
    fn retain(&mut self, keep_list: Vec<bool>) -> Result<(), ()> {
        assert_eq!(self.tuples.len(), keep_list.len());
        for (i, keep) in keep_list.iter().enumerate().rev() {
            if !keep {
                self.tuples.swap_remove(i);
            }
        }
        if self.tuples.is_empty() {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Apply `map` to all tuple elements, then return:
    /// (i) the product(and) of the (mapped) elements of each tuple
    /// (ii) the sum(or) of all those products
    fn apply_constraint<N, C: Constraint<N>>(
        &self,
        params: &[S::Var],
        map: impl Fn(usize, S::Value) -> N,
        constraint: &C,
    ) -> (Vec<C::Set>, C::Set) {
        let tuple_prod = |tuple: &Vec<S::Value>| -> C::Set {
            let nth_elem = |i| {
                let var = &self.header[i];
                let param_index = params.iter().position(|v| v == var).unwrap();
                let val_ref: &S::Value = &tuple[i];
                let mapped_val: N = map(param_index, val_ref.clone());
                constraint.singleton(param_index, mapped_val)
            };
            let mut prod = nth_elem(0);
            for i in 1..tuple.len() {
                prod = constraint.and(prod, nth_elem(i));
            }
            prod
        };

        let products = map_vec(&self.tuples, tuple_prod);

        let mut sum = products[0].clone();
        for prod in products.iter().skip(1) {
            sum = constraint.or(sum, prod.clone());
        }
        (products, sum)
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

impl<S: State> fmt::Display for Table<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tuple_to_state = |header: &[S::Var], tuple: &[S::Value]| -> HashMap<S::Var, S::Value> {
            let mut map = HashMap::new();
            for (i, x) in header.iter().enumerate() {
                map.insert(x.clone(), tuple[i].clone());
            }
            map
        };

        let show_section = |f: &mut fmt::Formatter, section: &Section<S>| -> fmt::Result {
            let mut states = Vec::new();
            if section.tuples.len() == 1 {
                states.push(tuple_to_state(&section.header, &section.tuples[0]));
            } else {
                for tuple in &section.tuples {
                    states.push(tuple_to_state(&section.header, &tuple));
                }
            }

            display_states::<S>(f, states)
        };

        let mut sections = self.sections.iter();
        let section = match sections.next() {
            None => return write!(f, "State is empty!"),
            Some(section) => section,
        };
        writeln!(f, "State is one of {}:", section.tuples.len())?;
        show_section(f, section)?;
        while let Some(section) = sections.next() {
            writeln!(f)?;
            writeln!(f, "and one of {}:", section.tuples.len())?;
            show_section(f, section)?;
        }
        Ok(())
    }
}
