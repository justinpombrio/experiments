extern crate rand;

use std::ops::{Index, IndexMut};
use self::rand::Rng;

use coord::*;



const RADIUS: usize = 5;
const MAP_DIMENSION: usize = 2 * RADIUS + 1;


static VOID: bool = false;

pub struct Terrain {
    radius: usize,
    ground: [[bool; MAP_DIMENSION]; MAP_DIMENSION]
}
impl Terrain {
    pub fn new() -> Terrain {
        let mut terrain = Terrain{
            radius: RADIUS,
            ground: [[false; MAP_DIMENSION]; MAP_DIMENSION]
        };
        let mut rng = rand::thread_rng();
        for hex in Hex::range(RADIUS) {
            terrain[hex] = rng.gen();
        }
        terrain
    }

    pub fn is_valid_hex(&self, hex: Hex) -> bool {
        hex.max_coord() <= RADIUS
    }

    fn validate_hex(&self, hex: Hex) {
        if !self.is_valid_hex(hex) {
            panic!("Terrain: invalid coordinate {}", hex);
        }
    }
}
impl IndexMut<Hex> for Terrain {
    fn index_mut(&mut self, hex: Hex) -> &mut bool {
        self.validate_hex(hex);
        let col = hex.p + self.radius as isize;
        let row = hex.q + (hex.p + (hex.p & 1)) / 2 + self.radius as isize;
        &mut self.ground[row as usize][col as usize]
    }
}
impl Index<Hex> for Terrain {
    type Output = bool;
    fn index(&self, hex: Hex) -> &bool {
        if self.is_valid_hex(hex) {
            let col = hex.p + self.radius as isize;
            let row = hex.q + (hex.p + (hex.p & 1)) / 2 + self.radius as isize;
            &self.ground[row as usize][col as usize]
        } else {
            &VOID
        }
    }
}
