mod png;
mod pack;

use std::collections::HashMap;

use self::png::*;
use self::pack::*;
use system::*;
use coord::*;
use tsv::*;
use tile::*;

static GEN_TILE_DIR: &'static str = "assets/gen/terrain/";
static RAW_TILE_DIR: &'static str = "assets/raw/terrain/";
static PURE_TILE_TSV: &'static str = "tiles_pure.tsv";
static PURE_POINT_TSV: &'static str = "points_pure.tsv";

static PURE_MASK_PNG: &'static str = "mask_pure.png";
static MIXED_MASK_PNG: &'static str = "mask_mixed.png";
static PURE_TILE_PNG: &'static str = "tiles_pure.png";
static MIXED_TILE_PNG: &'static str = "tiles_mixed.png";
static DOT_PNG: &'static str = "dot.png";

static PURE_OFFSET: Pixel = Pixel{ x: 210, y: 200 };



#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct TileKind {
    bottom: TileCorner,
    left:   TileCorner,
    top:    TileCorner,
    right:  TileCorner
}
impl TileKind {
    fn to_name(&self) -> String {
        let mut name: String = "".into();
        for corner in [self.bottom, self.left, self.top, self.right].iter() {
            name.push(corner.as_char());
        }
        name
    }
}

fn read_point_tsv(file: &mut RFile) -> HashMap<char, Pixel> {
    let mut tsv = TSV::load(file);
    tsv.arrange(vec!("Point", "X", "Y", "Z"));
    let mut points = HashMap::new();
    for row in tsv.data.iter() {
        let name = parse_name(&row[0]);
        let x = parse_number(&row[1]);
        let y = parse_number(&row[2]);
        let z = parse_number(&row[3]);
        if x + y + z != 0 {
            panic!("{}: cubic coordinates must sum to zero", file.path());
        }
        let pt = Hex::new(x, y).to_pixel(PURE_OFFSET);
        points.insert(name, pt);
    }
    points
}

struct TileInfo {
    point_name: char,
    color: png::Color,
    orientation: usize
}

fn read_tile_tsv(file: &mut RFile) -> HashMap<TileKind, [TileInfo; 6]> {
    let mut tsv = TSV::load(file);
    tsv.arrange(vec!(
        "Bottom", "Left", "Top", "Right", "Color",
        "Pt0", "Pt1", "Pt2", "Pt3", "Pt4", "Pt5", "Offset"));
    let mut tile_locs = HashMap::new();
    fn tile_info(pt: char, color: (u8, u8, u8),
                 orientation: usize, offset: isize) -> TileInfo {
        let (r, g, b) = color;
        let r = r + (17 * ((orientation + offset as usize) % 6)) as u8;
        let g = g + (17 * ((orientation + offset as usize) % 6)) as u8;
        let b = b + (17 * ((orientation + offset as usize) % 6)) as u8;
        TileInfo{
            point_name: pt,
            color: png::rgba(r, g, b, 255),
            orientation: orientation
        }
    }
    for row in tsv.data.iter() {
        let bottom = TileCorner::from_str(&row[0]);
        let left   = TileCorner::from_str(&row[1]);
        let top    = TileCorner::from_str(&row[2]);
        let right  = TileCorner::from_str(&row[3]);
        let color  = parse_color(&row[4]);
        let pt0    = parse_name(&row[5]);
        let pt1    = parse_name(&row[6]);
        let pt2    = parse_name(&row[7]);
        let pt3    = parse_name(&row[8]);
        let pt4    = parse_name(&row[9]);
        let pt5    = parse_name(&row[10]);
        let offset = parse_number(&row[11]);

        let tile_kind = TileKind{
            bottom: bottom,
            left:   left,
            top:    top,
            right:  right
        };

        tile_locs.insert(tile_kind, [
            tile_info(pt0, color, 0, offset),
            tile_info(pt1, color, 1, offset),
            tile_info(pt2, color, 2, offset),
            tile_info(pt3, color, 3, offset),
            tile_info(pt4, color, 4, offset),
            tile_info(pt5, color, 5, offset)]);
    }
    tile_locs
}

fn extract_tiles_folder(mask: &PNG,
                        img: &PNG,
                        dst_dir: &mut Dir,
                        points: &HashMap<char, Pixel>,
                        tiles:  &HashMap<TileKind, [TileInfo; 6]>) {
    for (tile_kind, tile_infos) in tiles {
        for tile_info in tile_infos.iter() {
            let tile_center = points.get(&tile_info.point_name)
                .expect(&format!("Point not found: {}", tile_info.point_name));
            let width = TILE_IMG_SIZE.x;
            let height = TILE_IMG_SIZE.y;
            let mut tile = PNG::new(width as u32, height as u32);
            for x in 0..width {
                for y in 0..height {
                    let pt = Pixel::new(x, y);
                    let src_pt = *tile_center + pt - TILE_IMG_CENTER;
                    let dst_pt = pt;
                    if mask[src_pt] == tile_info.color {
                        tile[dst_pt] = img[src_pt];
                    } else {
                        tile[dst_pt] = png::rgba(0, 0, 0, 0);
                    }
                }
            }
            let filename = &format!("tile_{}_{}.png",
                                    tile_kind.to_name(),
                                    tile_info.orientation);
            dst_dir.save(tile, filename);
        }
    }
}

fn extract_tiles(raw_dir: &Dir, gen_dir: &mut Dir) {
    gen_dir.clear();

    let pure_points = read_point_tsv(&mut raw_dir.open(PURE_POINT_TSV));
    let pure_tiles  = read_tile_tsv(&mut raw_dir.open(PURE_TILE_TSV));
    for name in raw_dir.contents() {
        if raw_dir.dir_exists(&name) {
            let raw_dir = raw_dir.dir(&name);
            let mut gen_dir = gen_dir.mkdir(&name);
            let mut pure_mask: PNG = raw_dir.load(PURE_MASK_PNG);
            let mut pure_template: PNG = raw_dir.load(PURE_TILE_PNG);
            raw_dir.copy_file(DOT_PNG, &mut gen_dir);
            log!("Extracting tiles", "{}", gen_dir.name());
            log_group(|| {
                extract_tiles_folder(&mut pure_mask, &mut pure_template,
                                     &mut gen_dir, &pure_points, &pure_tiles);
            });
            log!("Packing tiles", "{}", gen_dir.name());
            log_group(|| {
                pack_tiles(&mut gen_dir);
            });
        }
    }
}

pub fn generate_assets(home: &mut Dir) {
    let raw_dir = home.dir(RAW_TILE_DIR);
    let mut gen_dir = home.dir(GEN_TILE_DIR);
    clear_assets(home);
    extract_tiles(&raw_dir, &mut gen_dir)
}

pub fn clear_assets(home: &mut Dir) {
    home.dir(GEN_TILE_DIR).clear();
}
