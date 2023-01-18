use std::iter;

/// Iterator over the cartesian product of a list of sets. (All elements of the first set, cross
/// all elements of the second set, ..., cross all elements of the last set).
pub fn cartesian_prod<T>(inputs: &Vec<Vec<T>>) -> impl Iterator<Item = Vec<&T>> {
    if inputs.into_iter().any(|inp| inp.is_empty()) {
        return CartesianProdIter { inputs, path: None };
    } else {
        CartesianProdIter {
            path: Some(iter::repeat(0).take(inputs.len()).collect::<Vec<_>>()),
            inputs,
        }
    }
}

// This can also implement ExactSizeIter and DoubleEndedIter and such, but we don't need them now
struct CartesianProdIter<'a, T> {
    inputs: &'a Vec<Vec<T>>,
    path: Option<Vec<usize>>,
}

impl<'a, T> Iterator for CartesianProdIter<'a, T> {
    type Item = Vec<&'a T>;

    fn next(&mut self) -> Option<Vec<&'a T>> {
        if let Some(path) = &mut self.path {
            let cur_elem = path
                .iter()
                .copied()
                .enumerate()
                .map(|(i, j)| &self.inputs[i][j])
                .collect::<Vec<_>>();
            let mut i = self.inputs.len();
            loop {
                if i == 0 {
                    // Done!
                    self.path = None;
                    break;
                } else if path[i - 1] == self.inputs[i - 1].len() - 1 {
                    // This digit has overflowed; reset it and increment the next one.
                    path[i - 1] = 0;
                    i -= 1;
                } else {
                    // Increment this digit.
                    path[i - 1] += 1;
                    break;
                }
            }
            Some(cur_elem)
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

    let inputs = vec![];
    let mut iter = cartesian_prod(&inputs);
    assert_eq!(next_sum(&mut iter), 0);
    assert_eq!(iter.next(), None);

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

    let test_cases = vec![
        vec![],
        vec![vec![1]],
        vec![vec![1, 2]],
        vec![vec![1, 2, 3]],
        vec![vec![], vec![10, 20], vec![1, 2, 3]],
        vec![vec![1], vec![], vec![1]],
        vec![vec![1, 2], vec![1, 2], vec![]],
        vec![vec![10, 20], vec![1, 2]],
        vec![vec![10], vec![1, 2]],
        vec![vec![10, 20], vec![1]],
        vec![vec![10, 20, 30], vec![1, 2, 3]],
        vec![vec![100, 200], vec![10], vec![1, 2, 3]],
    ];
    for test_case in &test_cases {
        let expected_count = test_case.iter().map(|inps| inps.len()).product();
        assert_eq!(cartesian_prod(test_case).count(), expected_count);
    }
}
