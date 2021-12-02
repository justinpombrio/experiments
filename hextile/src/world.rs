use terrain::*;


static WORLD_RADIUS: usize = 5;

pub struct World {
    pub terrain: Terrain
}
impl World {
    pub fn new() -> World {
        World {
            terrain: Terrain::new()
        }
    }
}
