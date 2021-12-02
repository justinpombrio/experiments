use std::collections::HashMap;
use std::default::Default;

mod coordinate;

pub use coordinate::{Direction, SqCoord};

#[derive(Debug, Clone)]
pub struct World {
    radius: u32,
    squares: Vec<Vec<Square>>,
    out_of_bounds_square: Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Debug,
    Dirt,
    Grass,
    Water,
    Bridge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    Ground,
    Vegetation,
    Flow,
    Object,
}

#[derive(Debug, Clone)]
pub struct Square {
    blocks: HashMap<Layer, Option<Block>>,
}

impl Layer {
    pub fn all_layers() -> &'static [Layer] {
        &[Layer::Ground, Layer::Vegetation, Layer::Flow, Layer::Object]
    }
}

impl Block {
    fn layer(&self) -> Layer {
        match self {
            Block::Debug => Layer::Ground,
            Block::Dirt => Layer::Ground,
            Block::Grass => Layer::Vegetation,
            Block::Water => Layer::Flow,
            Block::Bridge => Layer::Object,
        }
    }

    pub fn keys() -> HashMap<char, Block> {
        let mut map = HashMap::new();
        map.insert('z', Block::Debug);
        map.insert('d', Block::Dirt);
        map.insert('g', Block::Grass);
        map.insert('w', Block::Water);
        map.insert('b', Block::Bridge);
        map
    }

    pub fn all_blocks() -> &'static [Block] {
        use Block::*;
        &[Debug, Dirt, Grass, Water, Bridge]
    }

    pub fn name(&self) -> &'static str {
        use Block::*;
        match self {
            Debug => "debug",
            Dirt => "dirt",
            Grass => "grass",
            Water => "water",
            Bridge => "bridge",
        }
    }

    pub fn block_names() -> HashMap<String, Block> {
        let mut map = HashMap::new();
        for block in Block::all_blocks() {
            map.insert(block.name().to_string(), *block);
        }
        map
    }
}

impl Default for Square {
    fn default() -> Square {
        Square {
            blocks: Layer::all_layers()
                .iter()
                .map(|layer| (*layer, None))
                .collect(),
        }
    }
}

impl Square {
    pub fn has_block(&self, block: Block) -> bool {
        self.blocks[&block.layer()] == Some(block)
    }

    pub fn set_block(&mut self, block: Block) {
        self.blocks.insert(block.layer(), Some(block));
    }

    pub fn remove_block(&mut self, block: Block) {
        self.blocks.insert(block.layer(), None);
    }

    pub fn blocks(&self) -> impl Iterator<Item = Block> {
        Layer::all_layers()
            .iter()
            .filter_map(|layer| self.blocks[layer])
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn get_layer(&self, layer: Layer) -> Option<Block> {
        match self.blocks.get(&layer) {
            Some(opt) => *opt,
            None => None,
        }
    }
}

impl World {
    pub fn new(radius: u32) -> World {
        let mut squares = vec![];
        let radius = radius as i32;
        for _ in -radius..=radius {
            let mut row = vec![];
            for _ in -radius..=radius {
                row.push(Square::default());
            }
            squares.push(row);
        }
        let out_of_bounds_square = Square::default();
        World {
            radius: radius as u32,
            squares,
            out_of_bounds_square,
        }
    }

    fn is_valid_coord(&self, [p, q]: SqCoord) -> bool {
        p.abs() as u32 <= self.radius && q.abs() as u32 <= self.radius
    }

    pub fn get_square(&self, [p, q]: SqCoord) -> &Square {
        if self.is_valid_coord([p, q]) {
            let radius = self.radius as i32;
            &self.squares[(p + radius) as usize][(q + radius) as usize]
        } else {
            &self.out_of_bounds_square
        }
    }

    pub fn get_squares_around_tile(&self, [p, q]: SqCoord) -> [&Square; 4] {
        [
            self.get_square([p + 1, q + 1]),
            self.get_square([p, q + 1]),
            self.get_square([p, q]),
            self.get_square([p + 1, q]),
        ]
    }

    pub fn get_square_mut(&mut self, [p, q]: SqCoord) -> Option<&mut Square> {
        if self.is_valid_coord([p, q]) {
            let radius = self.radius as i32;
            Some(&mut self.squares[(p + radius) as usize][(q + radius) as usize])
        } else {
            None
        }
    }

    pub fn all_coords(&self) -> impl Iterator<Item = SqCoord> + 'static {
        let radius = self.radius as i32;
        SqCoordIter::new((-radius, radius), (-radius, radius))
    }

    pub fn all_tile_coords(&self) -> impl Iterator<Item = SqCoord> + 'static {
        let radius = self.radius as i32;
        SqCoordIter::new((-radius - 1, radius), (-radius - 1, radius))
    }
}

struct SqCoordIter {
    min_p: i32,
    max_p: i32,
    min_q: i32,
    p: i32,
    q: i32,
}

impl SqCoordIter {
    fn new((min_p, max_p): (i32, i32), (min_q, max_q): (i32, i32)) -> SqCoordIter {
        SqCoordIter {
            min_p,
            max_p,
            min_q,
            p: min_p,
            q: max_q,
        }
    }
}

impl Iterator for SqCoordIter {
    type Item = SqCoord;
    fn next(&mut self) -> Option<SqCoord> {
        if self.q < self.min_q {
            return None;
        }
        let coord = [self.p, self.q];
        if self.p == self.max_p {
            self.q -= 1;
            self.p = self.min_p;
        } else {
            self.p += 1;
        }
        Some(coord)
    }
}

#[cfg(test)]
mod test_world {
    use super::*;

    #[test]
    fn test_world_coords() {
        let world = World::new(2);
        assert_eq!(world.all_coords().count(), 25);
        for coord in world.all_coords() {
            assert!(world.is_valid_coord(coord));
            world.get_square(coord);
        }
        assert_eq!(world.all_tile_coords().count(), 36);
    }
}

pub fn demo_world() -> World {
    let mut world = World::new(4);
    for sq in world.all_coords() {
        world.get_square_mut(sq).unwrap().set_block(Block::Dirt);
    }
    let bridge = Block::Bridge;
    world.get_square_mut([0, 0]).unwrap().set_block(bridge);
    world.get_square_mut([0, 2]).unwrap().set_block(bridge);
    world.get_square_mut([1, 0]).unwrap().set_block(bridge);
    world.get_square_mut([1, 1]).unwrap().set_block(bridge);
    world.get_square_mut([2, 2]).unwrap().set_block(bridge);
    world.get_square_mut([3, 1]).unwrap().set_block(bridge);
    world.get_square_mut([-1, -1]).unwrap().set_block(bridge);
    world.get_square_mut([-2, -3]).unwrap().set_block(bridge);
    world
}
