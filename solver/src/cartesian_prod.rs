/// Iterator over the cartesian product of a list of sets. (All elements of the first set, cross
/// all elements of the second set, ..., cross all elements of the last set).
pub fn cartesian_prod<T>(inputs: &Vec<Vec<T>>) -> impl Iterator<Item = Vec<&T>> {
    // Do a DFS, starting with the leftmost path / the number 0.
    CartesianProdIter::new(inputs)
}

// This can also implement ExactSizeIter and DoubleEndedIter and such, but we don't need them now
struct CartesianProdIter<'a, T> {
    inputs: &'a Vec<Vec<T>>,
    path: Option<Vec<usize>>,
}

impl<'a, T> CartesianProdIter<'a, T> {
    fn new(inputs: &'a Vec<Vec<T>>) -> CartesianProdIter<'a, T> {
        let mut path = Vec::new();
        for input in inputs {
            if input.is_empty() {
                return CartesianProdIter { inputs, path: None };
            }
            path.push(0);
        }

        CartesianProdIter {
            inputs,
            path: Some(path),
        }
    }

    /// Use our path indices to construct the output tuple.
    fn current_elem(&self) -> Option<Vec<&'a T>> {
        if let Some(path) = &self.path {
            Some(
                path.iter()
                    .copied()
                    .enumerate()
                    .map(|(i, j)| &self.inputs[i][j])
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        }
    }

    /// Count up by 1, or from another perspective pick the next DFS path.
    fn inc(&mut self) {
        if let Some(path) = &mut self.path {
            if path.len() == 0 {
                self.path = None;
                return;
            }
            for i in (0..path.len()).rev() {
                if path[i] + 1 == self.inputs[i].len() {
                    if i == 0 {
                        // We're done!
                        self.path = None;
                        return;
                    } else {
                        // This digit has overflowed; reset it and increment the next one.
                        path[i] = 0;
                    }
                } else {
                    // Increment this digit.
                    path[i] += 1;
                    return;
                }
            }
        }
    }
}

impl<'a, T> Iterator for CartesianProdIter<'a, T> {
    type Item = Vec<&'a T>;

    fn next(&mut self) -> Option<Vec<&'a T>> {
        if let Some(result) = self.current_elem() {
            self.inc();
            Some(result)
        } else {
            None
        }
    }
}

#[test]
fn test_cartesian_prod() {
    fn next_sum<'a>(iter: &mut impl Iterator<Item = Vec<&'a i32>>) -> i32 {
        iter.next().unwrap().into_iter().copied().sum()
    }

    // Empty outer vector
    let inputs = vec![];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(next_sum(&mut iter), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    // Empty inner vector -> cross product is empty
    let inputs: Vec<Vec<i32>> = vec![vec![]];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    let inputs = vec![vec![], vec![10, 20], vec![1, 2, 3]];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    let inputs = vec![vec![1], vec![], vec![1]];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    let inputs = vec![vec![1, 2], vec![1, 2], vec![]];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    // One dimensional
    let inputs = vec![vec![1]];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(next_sum(&mut iter), 1);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    let inputs = vec![vec![1, 2]];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(next_sum(&mut iter), 1);
    assert_eq!(next_sum(&mut iter), 2);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    let inputs = vec![vec![1, 2, 3]];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(next_sum(&mut iter), 1);
    assert_eq!(next_sum(&mut iter), 2);
    assert_eq!(next_sum(&mut iter), 3);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    // Two dimensional
    let inputs = vec![vec![10, 20], vec![1, 2]];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(next_sum(&mut iter), 11);
    assert_eq!(next_sum(&mut iter), 12);
    assert_eq!(next_sum(&mut iter), 21);
    assert_eq!(next_sum(&mut iter), 22);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    let inputs = vec![vec![10], vec![1, 2]];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(next_sum(&mut iter), 11);
    assert_eq!(next_sum(&mut iter), 12);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    let inputs = vec![vec![10, 20], vec![1]];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(next_sum(&mut iter), 11);
    assert_eq!(next_sum(&mut iter), 21);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    let inputs = vec![vec![10, 20, 30], vec![1, 2, 3]];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(next_sum(&mut iter), 11);
    assert_eq!(next_sum(&mut iter), 12);
    assert_eq!(next_sum(&mut iter), 13);
    assert_eq!(next_sum(&mut iter), 21);
    assert_eq!(next_sum(&mut iter), 22);
    assert_eq!(next_sum(&mut iter), 23);
    assert_eq!(next_sum(&mut iter), 31);
    assert_eq!(next_sum(&mut iter), 32);
    assert_eq!(next_sum(&mut iter), 33);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    // Three dimensional
    let inputs = vec![vec![100, 200], vec![10], vec![1, 2, 3]];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(next_sum(&mut iter), 111);
    assert_eq!(next_sum(&mut iter), 112);
    assert_eq!(next_sum(&mut iter), 113);
    assert_eq!(next_sum(&mut iter), 211);
    assert_eq!(next_sum(&mut iter), 212);
    assert_eq!(next_sum(&mut iter), 213);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
}
