use crate::arith::Arith;

pub trait Ring: Clone + 'static {
    fn one() -> Self;

    fn mul(self, other: Self) -> Self;
    fn div(self, other: Self) -> Self;
    fn add(self, other: Self) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range<N: Arith>(N, N);

impl<N: Arith> Range<N> {
    pub fn new(n: N) -> Range<N> {
        Range(n.clone(), n)
    }

    pub fn pred(&self, n: N) -> bool {
        self.0 <= n && self.1 >= n
    }
}

impl<N: Arith> Ring for Range<N> {
    fn one() -> Range<N> {
        Range(N::identity(), N::identity())
    }

    fn mul(self, other: Range<N>) -> Range<N> {
        Range(N::add(self.0, other.0), N::add(self.1, other.1))
    }

    fn div(self, other: Range<N>) -> Range<N> {
        Range(N::sub(self.0, other.0), N::sub(self.1, other.1))
    }

    fn add(self, other: Range<N>) -> Range<N> {
        Range(N::min(self.0, other.0), N::max(self.1, other.1))
    }
}
