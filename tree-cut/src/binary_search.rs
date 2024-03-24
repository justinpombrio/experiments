use crate::tree::Weight;

/// Find the Weight `w` such that `predicate(w) == true` but `predicate(w + 1) == false`. Requires
/// that:
///
/// - For all `n`, `predicate(n + 1)` implies `predicate(n)`.
/// - `predicate(lower)`
/// - `!predicate(upper)`
pub fn binary_search(
    mut lower: Weight,
    mut upper: Weight,
    mut predicate: impl FnMut(Weight) -> bool,
) -> Weight {
    assert!(predicate(lower));
    assert!(!predicate(upper));

    while lower + 1 < upper {
        let mid = lower + (upper - lower) / 2;
        if predicate(mid) {
            lower = mid;
        } else {
            upper = mid;
        }
    }
    lower
}

/// Find the Weight `w` such that `predicate(w) == true` but `predicate(w - 1) == false`. Requires
/// that:
///
/// - For all `n`, `predicate(n)` implies `predicate(n + 1)`.
/// - `!predicate(lower)`
/// - `predicate(upper)`
pub fn reverse_binary_search(
    mut lower: Weight,
    mut upper: Weight,
    mut predicate: impl FnMut(Weight) -> bool,
) -> Weight {
    binary_search(lower, upper, |w| !predicate(w)) + 1
}
