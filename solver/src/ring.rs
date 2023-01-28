use crate::arith::Arith;

pub trait Ring: Clone + 'static {
    fn one() -> Self;

    fn mul(a: Self, b: Self) -> Self;
    fn add(a: Self, b: Self) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RangeRing<N: Arith>(N, N);

impl<N: Arith> Ring for RangeRing<N> {
    fn one() -> RangeRing<N> {
        RangeRing(N::identity(), N::identity())
    }

    fn mul(a: RangeRing<N>, b: RangeRing<N>) -> RangeRing<N> {
        RangeRing(N::add(a.0, b.0), N::add(a.1, b.1))
    }

    fn add(a: RangeRing<N>, b: RangeRing<N>) -> RangeRing<N> {
        RangeRing(N::min(a.0, b.0), N::max(a.1, b.1))
    }
}
