use std::fmt;

use coord::*;

pub use self::TileCorner::*;



pub type Orientation = u8; // Must be in 0..6!

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Rhomb {
    hex: Hex,
    orientation: Orientation
}
impl Rhomb {
    pub fn new(hex: Hex, orientation: Orientation) -> Rhomb {
        if orientation >= 6 {
            panic!("Attempted to create invalid orientation")
        }
        Rhomb{
            hex: hex,
            orientation: orientation
        }
    }
    pub fn center(self) -> Hex {
        self.hex
    }
    pub fn left(self) -> Hex {
        self.hex + HEX_NEIGHBORS[(self.orientation as usize + 5) % 6]
    }
    pub fn right(self) -> Hex {
        self.hex + HEX_NEIGHBORS[(self.orientation as usize + 1) % 6]
    }
    pub fn top(self) -> Hex {
        self.hex + HEX_NEIGHBORS[self.orientation as usize % 6]
    }
}
impl fmt::Display for Rhomb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rhomb[{} {}]", self.hex, self.orientation)
    }
}
impl fmt::Debug for Rhomb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}


pub type Material = u8;


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileCorner {
    SameC,
    DiffC,
    SpecC
}
impl fmt::Display for TileCorner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }    
}
impl fmt::Debug for TileCorner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl TileCorner {
    pub fn as_char(self) -> char {
        match self {
            SameC => 's', // _S_ame
            DiffC => 'o', // _O_ther
            SpecC => 'x'  // _X_ecial
        }
    }

    pub fn from_str(s: &str) -> TileCorner {
        match s {
            "s" => SameC,
            "o" => DiffC,
            "x" => SpecC,
            _ => panic!("Invalid tile kind: {}", s)
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tile {
    orientation: Orientation,
    material: Material,
    left:  TileCorner,
    top:   TileCorner,
    right: TileCorner
}
impl Tile {
    pub fn new(orientation: Orientation, material: Material,
               left: TileCorner, top: TileCorner, right: TileCorner) -> Tile {
        Tile{
            orientation: orientation,
            material: material,
            left:  left,
            top:   top,
            right: right
        }
    }
}
impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tile_s{}{}{}_{}", self.left, self.top, self.right, self.orientation)
    }    
}
