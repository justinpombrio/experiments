extern crate image;
extern crate texture_packer;

use std::path::Path;

use coord::*;
use system::*;
use tsv::*;
//use self::image::*;
use self::texture_packer::exporter::ImageExporter;
use self::texture_packer::importer::ImageImporter;
use self::texture_packer::texture::Texture;
use self::texture_packer::{TexturePacker, TexturePackerConfig};

static TEXTURE_PNG: &'static str = "texture.png";
static TEXTURE_TSV: &'static str = "texture.tsv";

const MAX_IMG_WIDTH: u32 = 640;
const MAX_IMG_HEIGHT: u32 = 640;

pub fn pack_tiles(dir: &mut Dir) {
    let mut config = TexturePackerConfig::default();
    config.max_width = MAX_IMG_WIDTH;
    config.max_height = MAX_IMG_HEIGHT;
    config.allow_rotation = false;
    config.texture_outlines = false;
    config.border_padding = 0;

    let ref mut packer = TexturePacker::new_skyline(config);

    log!("Packing tiles");
    log_group(|| {
        for name in dir.contents() {
            if name.starts_with("tile") && name.ends_with(".png") && dir.file_exists(&name) {
                let len = name.len();
                let short_name = name[..len - 4].to_string();
                log!("Tile", "{}", name);
                // TODO: pass in file directly
                let path = Path::new(dir.path()).join(&name);
                let texture = ImageImporter::import_from_file(&path).unwrap();
                packer.pack_own(short_name, texture);
            }
        }
    });

    let packed = ImageExporter::export(packer).unwrap();
    log!("Total size", "{} x {}", packer.width(), packer.height());

    packed.save(TEXTURE_PNG).unwrap();

    let mut tsv = TSV::new(vec![
        "Tile", "PosnX", "PosnY", "SizeX", "SizeY", "OffsetX", "OffsetY",
    ]);
    log_group(|| {
        for frame in packer.get_frames().values() {
            let frame_px = Pixel::new(frame.frame.x as isize, frame.frame.y as isize);
            let source_px = Pixel::new(frame.source.x as isize, frame.source.y as isize);
            let offset = TILE_IMG_CENTER - source_px;
            let size = Pixel::new(frame.frame.w as isize, frame.frame.h as isize);
            tsv.add_row(vec![
                frame.key.clone(),
                format!("{}", frame_px.x),
                format!("{}", frame_px.y),
                format!("{}", size.x),
                format!("{}", size.y),
                format!("{}", offset.x),
                format!("{}", offset.y),
            ]);
            log!("Frame", "{} {} {} {}", frame.key, frame_px, size, offset);
        }
    });
    dir.save(tsv, TEXTURE_TSV);
}
