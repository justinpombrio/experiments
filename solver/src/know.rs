// TODO: temporary
#![allow(unused)]

use std::collections::HashSet;

use crate::state::State;

struct Component<S: State> {
    domain: Vec<S::Var>,
    tuples: HashSet<Vec<S::Value>>,
}

impl<S: State> Component<S> {
    fn project(&self, domain: &[S::Var]) -> Option<Component<S>> {
        let (subdomain, mapping) = project_domain::<S>(&self.domain, &domain)?;

        let mut tuples = HashSet::new();
        for tuple in &self.tuples {
            let subtuple = mapping
                .iter()
                .map(|i| tuple[*i].clone())
                .collect::<Vec<_>>();
            tuples.insert(subtuple);
        }

        Some(Component {
            domain: subdomain,
            tuples,
        })
    }

    fn filter(&mut self, other: &Component<S>) {
        let (_, mapping) = match project_domain::<S>(&self.domain, &other.domain) {
            None => return (),
            Some(result) => result,
        };
        self.tuples.retain(|tuple| {
            let other_tuple = mapping
                .iter()
                .map(|i| tuple[*i].clone())
                .collect::<Vec<_>>();
            other.tuples.contains(&other_tuple)
        });
    }
}

/// Let `subdomain` be the intersection of `domain_1` and `domain_2`. If `subdomain` is empty,
/// return None. Otherwise return `(subdomain, mapping)`, where `subdomain[i] =
/// domain_1[mapping[i]]`.
fn project_domain<S: State>(
    domain_1: &[S::Var],
    domain_2: &[S::Var],
) -> Option<(Vec<S::Var>, Vec<usize>)> {
    let (subdomain, mapping) = domain_2
        .iter()
        .filter_map(|x| domain_1.iter().position(|y| y == x).map(|i| (x.clone(), i)))
        .unzip::<_, _, Vec<_>, Vec<_>>();

    if subdomain.is_empty() {
        None
    } else {
        Some((subdomain, mapping))
    }
}

struct Knowledge<S: State> {
    components: Vec<Component<S>>,
}

impl<S: State> Knowledge<S> {
    fn project(&self, subdomain: Vec<S::Var>) -> Knowledge<S> {
        let mut components = Vec::new();
        for component in &self.components {
            if let Some(projected_component) = component.project(&subdomain) {
                components.push(projected_component);
            }
        }
        Knowledge { components }
    }
}
