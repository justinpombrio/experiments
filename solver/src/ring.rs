use crate::arith::Arith;

pub trait Ring: Clone + 'static {
    fn one() -> Self;

    fn mul(self, other: Self) -> Self;
    fn div(self, other: Self) -> Self;
    fn add(self, other: Self) -> Self;
}

// TODO: private
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeRing<N: Arith>(pub N, pub N);

impl<N: Arith> RangeRing<N> {
    pub fn new(n: N) -> RangeRing<N> {
        RangeRing(n.clone(), n)
    }
}

impl<N: Arith> Ring for RangeRing<N> {
    fn one() -> RangeRing<N> {
        RangeRing(N::identity(), N::identity())
    }

    fn mul(self, other: RangeRing<N>) -> RangeRing<N> {
        RangeRing(N::add(self.0, other.0), N::add(self.1, other.1))
    }

    fn div(self, other: RangeRing<N>) -> RangeRing<N> {
        RangeRing(N::sub(self.0, other.0), N::sub(self.1, other.1))
    }

    fn add(self, other: RangeRing<N>) -> RangeRing<N> {
        RangeRing(N::min(self.0, other.0), N::max(self.1, other.1))
    }
}
