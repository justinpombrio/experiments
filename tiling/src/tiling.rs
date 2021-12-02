use crate::engine::{Engine, Image};
use crate::world::Block;
use std::fmt;
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DrawOrder {
    First,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Link {
    Full,
    Half,
    Empty,
    Wild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileKey {
    pub block: Block,
    pub links: [Link; 4],
}

pub struct Tiles {
    tiles: HashMap<Block, Vec<(TileKey, Image)>>,
}

impl Tiles {
    pub fn new() -> Tiles {
        Tiles {
            tiles: HashMap::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, engine: &mut Engine, path: P) {
        let path = path.as_ref();
        println!("Loading tilesets from {:?}", path);
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_file() {
                continue;
            }
            if let Some(key) = TileKey::from_filename(entry.path()) {
                let image = engine.load_image(entry.path());
                if !self.tiles.contains_key(&key.block) {
                    self.tiles.insert(key.block, vec![]);
                }
                self.tiles.get_mut(&key.block).unwrap().push((key, image));
            }
        }
    }

    pub fn get_tiles(&self, keys: &[TileKey]) -> Vec<Image> {
        let mut images = vec![];
        for key in keys {
            for (existing_key, image) in &self.tiles[&key.block] {
                if key.matches(*existing_key) {
                    images.push((existing_key, *image));
                }
            }
        }
        images.sort_unstable_by_key(|(key, _)| key.draw_order());
        images.into_iter().map(|(_, img)| img).collect()
    }
}

impl TileKey {
    pub fn from_filename<P: AsRef<Path>>(path: P) -> Option<TileKey> {
        let path = path.as_ref();
        let extension = path.extension().unwrap().to_str().unwrap();
        if extension != "png" {
            return None;
        }
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let stem_parts = stem.split('_').collect::<Vec<_>>();
        assert_eq!(stem_parts.len(), 2, "Wrong number of _-sep parts in tile filename");
        let (tileset_name, links_str) = (stem_parts[0], stem_parts[1]);
        assert_eq!(links_str.chars().count(), 4, "Wrong length of links in tile filename");
        let mut links = links_str.chars().map(Link::from_char);
        let links = [links.next().unwrap(), links.next().unwrap(), links.next().unwrap(), links.next().unwrap()];
        Some(TileKey {
            // TODO: efficiency
            block: Block::block_names()[tileset_name],
            links,
        })
    }

    pub fn to_filename(self) -> String {
        format!("{}", self)
    }

    pub fn matches(self, other: TileKey) -> bool {
        self.block == other.block
            && self.links[0].matches(other.links[0]) && self.links[1].matches(other.links[1])
            && self.links[2].matches(other.links[2]) && self.links[3].matches(other.links[3])
    }

    pub fn draw_order(self) -> DrawOrder {
        use Link::*;
        if (self.links[0] == Full && self.links[3] != Full)
            || (self.links[1] == Full && self.links[2] != Full) {
            DrawOrder::First
        } else {
            DrawOrder::Last
        }
    }
}

impl Link {
    pub fn from_char(ch: char) -> Link {
        use Link::*;
        match ch {
            'f' => Full,
            'e' => Empty,
            'h' => Half,
            'x' => Wild,
            c => panic!("Invalid link character {}", c),
        }
    }

    pub fn to_char(self) -> char {
        use Link::*;
        match self {
            Full => 'f',
            Half => 'h',
            Empty => 'e',
            Wild => 'x',
        }
    }

    pub fn matches(self, other: Link) -> bool {
        self == other || other == Link::Wild
    }
}

impl fmt::Display for TileKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}_{}{}{}{}.png",
            self.block.name(),
            self.links[0],
            self.links[1],
            self.links[2],
            self.links[3],
        )
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}
