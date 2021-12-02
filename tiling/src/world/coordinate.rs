use std::fmt;

pub type SqCoord = [i32; 2];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn coord_delta(&self) -> SqCoord {
        use Direction::*;
        match self {
            North => [0, 1],
            East => [1, 0],
            South => [0, -1],
            West => [-1, 0],
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Direction::*;
        let text = match self {
            North => "N",
            East => "E",
            South => "S",
            West => "W",
        };
        write!(f, "{}", text)
    }
}
