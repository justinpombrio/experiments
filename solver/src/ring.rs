use crate::arith::Arith;

pub trait Ring: Clone + 'static {
    fn zero() -> Self;
    fn one() -> Self;

    fn mul(a: Self, b: Self) -> Self;
    fn add(a: Self, b: Self) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RangeRing<N: Arith> {
    Empty,
    Range(N, N),
}

impl<N: Arith> Ring for RangeRing<N> {
    fn zero() -> RangeRing<N> {
        RangeRing::Empty
    }
    fn one() -> RangeRing<N> {
        RangeRing::Range(N::identity(), N::identity())
    }

    fn mul(a: RangeRing<N>, b: RangeRing<N>) -> RangeRing<N> {
        use RangeRing::{Empty, Range};

        match (a, b) {
            (Empty, Empty) | (Empty, Range(_, _)) | (Range(_, _), Empty) => Empty,
            (Range(a0, a1), Range(b0, b1)) => Range(N::add(a0, a1), N::add(b0, b1)),
        }
    }

    fn add(a: RangeRing<N>, b: RangeRing<N>) -> RangeRing<N> {
        use RangeRing::{Empty, Range};

        match (a, b) {
            (Empty, Empty) => Empty,
            (Empty, Range(b0, b1)) => Range(b0, b1),
            (Range(a0, a1), Empty) => Range(a0, a1),
            (Range(a0, a1), Range(b0, b1)) => Range(N::min(a0, b0), N::max(a1, b1)),
        }
    }
}
