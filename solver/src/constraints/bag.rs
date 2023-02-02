use super::Constraint;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

/// The constraint that `{X1, ..., Xn} = expected`
pub struct Bag<N: Debug + Hash + Eq + Clone + Sized + 'static> {
    expected: HashMap<N, i32>,
}

impl<N: Debug + Hash + Eq + Clone + Sized + 'static> Bag<N> {
    pub fn new(expected: impl IntoIterator<Item = N>) -> Bag<N> {
        let mut map = HashMap::new();
        for x in expected {
            map.entry(x.clone()).and_modify(|n| *n += 1).or_insert(1);
        }
        Bag { expected: map }
    }
}

impl<N: Debug + Hash + Eq + Clone + Sized + 'static> Constraint<N> for Bag<N> {
    type Set = (HashMap<N, i32>, HashMap<N, i32>);

    const NAME: &'static str = "Bag";

    fn singleton(&self, _index: usize, elem: N) -> Self::Set {
        (
            HashMap::from([(elem.clone(), 1)]),
            HashMap::from([(elem, 1)]),
        )
    }

    fn and(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        (
            combine_hashmaps(a.0, b.0, |n, m| n + m),
            combine_hashmaps(a.1, b.1, |n, m| n + m),
        )
    }

    fn or(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        (
            combine_hashmaps(a.0, b.0, std::cmp::min),
            combine_hashmaps(a.1, b.1, std::cmp::max),
        )
    }

    fn check(&self, set: Self::Set) -> bool {
        zip_hashmaps(&set.0, &self.expected).all(|(_, c1, c2)| c1 <= c2)
            && zip_hashmaps(&set.1, &self.expected).all(|(_, c1, c2)| c1 >= c2)
    }
}

fn zip_hashmaps<'a, K: Debug + Eq + Hash + Clone>(
    map_1: &'a HashMap<K, i32>,
    map_2: &'a HashMap<K, i32>,
) -> impl Iterator<Item = (K, i32, i32)> + 'a {
    let mut keys = map_1.keys().cloned().collect::<HashSet<_>>();
    keys.extend(map_2.keys().cloned());
    keys.into_iter().map(|key| {
        (
            key.clone(),
            map_1.get(&key).copied().unwrap_or(0),
            map_2.get(&key).copied().unwrap_or(0),
        )
    })
}

fn combine_hashmaps<K: Debug + Eq + Hash + Clone>(
    map_1: HashMap<K, i32>,
    map_2: HashMap<K, i32>,
    combine: impl Fn(i32, i32) -> i32,
) -> HashMap<K, i32> {
    zip_hashmaps(&map_1, &map_2)
        .map(|(k, v1, v2)| (k, combine(v1, v2)))
        .collect::<HashMap<_, _>>()
}
