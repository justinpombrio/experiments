use image;
use image::{GenericImageView, Rgba, DynamicImage, ImageBuffer};
use std::path::Path;


// ****** Template Layouts ******

const TILE_WIDTH: u32 = 20;
const TILE_HEIGHT: u32 = 24;

const SIMPLE_TEMPLATE_IMAGE_SIZE: (u32, u32) = (60, 40);
// [[a, b], [c, d]] -> x'=ax+b, y'=cy+d
const SIMPLE_TEMPLATE_LINEAR_EQS: [u32; 4] = [10, 0, 8, 4];
const SIMPLE_TEMPLATE_LAYOUT: [(&'static str, &'static str, [u32; 2]); 13] = [
    ("ffff", "eee", [3, 2]),
    ("fffe", "ee4", [3, 3]),
    ("efff", "6ea", [4, 2]),
    ("feff", "8ae", [3, 1]),
    ("ffef", "e4e", [2, 2]),
    ("ffee", "e44", [4, 3]),
    ("effe", "6e0", [4, 1]),
    ("eeff", "0aa", [2, 1]),
    ("feef", "80e", [2, 3]),
    ("fexe", "804", [1, 2]),
    ("efex", "640", [5, 3]),
    ("xefe", "0a0", [5, 2]),
    ("exef", "00a", [1, 1]),
];

const COMPLEX_TEMPLATE_IMAGE_SIZE: (u32, u32) = (80, 120);
// [[a, b], [c, d]] -> x'=ax+b, y'=cy+d
const COMPLEX_TEMPLATE_LINEAR_EQS: [u32; 4] = [10, 0, 8, 4];
const COMPLEX_TEMPLATE_LAYOUT: [(&'static str, &'static str, [u32; 2]); 41] = [
    ("ffff", "eee", [3, 3]),
    ("fffe", "ee4", [3, 4]),
    ("efff", "6ea", [4, 3]),
    ("feff", "8ae", [3, 2]),
    ("ffef", "e4e", [2, 3]),
    ("ffee", "e44", [4, 4]),
    ("effe", "6e0", [4, 2]),
    ("eeff", "0aa", [2, 2]),
    ("feef", "80e", [2, 4]),
    ("fexe", "804", [1, 3]),
    ("efex", "640", [5, 4]),
    ("xefe", "0a0", [5, 3]),
    ("exef", "00a", [1, 2]),
    ("hxef", "40c", [3, 1]),
    ("xhfe", "4a2", [4, 1]),
    ("fexh", "809", [2, 5]),
    ("efhx", "690", [3, 5]),
    // Second block
    ("hxhf", "45c", [4, 7]),
    ("xhfh", "3c5", [5, 7]),
    ("fhhf", "b7e", [4, 8]),
    ("hffe", "ae2", [5, 8]),
    ("exhf", "32a", [1, 9]),
    ("heff", "4ac", [2, 9]),
    ("hhff", "7cc", [3, 9]),
    ("fhff", "bce", [4, 9]),
    ("effh", "6e5", [5, 9]),
    ("fhxe", "b24", [1, 10]),
    ("ffeh", "e49", [2, 10]),
    ("ffhf", "e9e", [3, 10]),
    ("hfff", "aec", [5, 10]),
    ("ehff", "3ca", [6, 10]),
    ("xefh", "0a5", [7, 10]),
    ("fhef", "b2e", [3, 11]),
    ("fffh", "ee9", [4, 11]),
    ("ffhh", "e99", [5, 11]),
    ("ffhe", "e94", [6, 11]),
    ("hfex", "a42", [7, 11]),
    ("fehf", "85e", [3, 12]),
    ("hffh", "ae7", [4, 12]),
    ("fhxh", "b29", [3, 13]),
    ("hfhx", "ae2", [4, 13]),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutType { Simple, Complex }

#[derive(Debug, Clone)]
struct TemplateLayout {
    template_size: (u32, u32),
    tile_size: (u32, u32),
    tiles: Vec<TileInfo>,
}

#[derive(Debug, Clone, Copy)]
struct TileInfo {
    suffix: &'static str,
    center: [u32; 2],
    mask_color: Rgba<u8>,
}

#[derive(Debug)]
struct TemplateLayouts {
    simple: TemplateLayout,
    complex: TemplateLayout,
}

fn color_to_ints(color: &str) -> Rgba<u8> {
    fn hex_to_num(c: char) -> u8 {
        let n = c.to_digit(16).unwrap() as u8;
        16 * n + n
    }
    let mut digits = color.chars();
    let r = digits.next().unwrap();
    let g = digits.next().unwrap();
    let b = digits.next().unwrap();
    Rgba([hex_to_num(r), hex_to_num(g), hex_to_num(b), 255])
}

impl TemplateLayout {
    fn new(layout_type: LayoutType) -> TemplateLayout {
        use LayoutType::*;
        let template_size = match layout_type {
            Simple => SIMPLE_TEMPLATE_IMAGE_SIZE,
            Complex => COMPLEX_TEMPLATE_IMAGE_SIZE,
        };
        let tile_size = (TILE_WIDTH, TILE_HEIGHT);
        let layout: &[(&str, &str, [u32; 2])] = match layout_type {
            Simple => &SIMPLE_TEMPLATE_LAYOUT,
            Complex => &COMPLEX_TEMPLATE_LAYOUT,
        };
        let [a, b, c, d] = match layout_type {
            Simple => SIMPLE_TEMPLATE_LINEAR_EQS,
            Complex => COMPLEX_TEMPLATE_LINEAR_EQS,
        };
        let tiles = layout.iter().map(move |(suffix, mask_color, center)| {
            let center = [a * center[0] + b, c * center[1] + d];
            let mask_color = color_to_ints(mask_color);
            TileInfo {
                suffix,
                mask_color,
                center
            }
        }).collect();
        TemplateLayout {
            template_size,
            tile_size,
            tiles,
        }
    }

    fn tiles(&self) -> impl Iterator<Item = &TileInfo> {
        self.tiles.iter()
    }
}

impl TemplateLayouts {
    fn new() -> TemplateLayouts {
        TemplateLayouts {
            simple: TemplateLayout::new(LayoutType::Simple),
            complex: TemplateLayout::new(LayoutType::Complex),
        }
    }
}


// ****** Tilesets ******

#[derive(Clone)]
struct Tileset<'a> {
    name: String,
    layout: &'a TemplateLayout,
    template_img: DynamicImage,
    mask_img: DynamicImage,
    normal_img: Option<DynamicImage>,
}

struct Tile {
    filename: String,
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl Tileset<'_> {
    /// Load a tileset from the images in a directory, that match the given tileset name.
    fn load<'a, P: AsRef<Path>>(path: P, tileset_name: &str, layouts: &'a TemplateLayouts) -> Tileset<'a> {
        let path = path.as_ref();
        // Load the tileset images
        let template_path = Path::new(path).join(format!("{}_template.png", tileset_name));
        let template_img = image::open(template_path).expect("failed to load tileset");
        // Load the mask
        let mask_path = Path::new(path).join(format!("{}_mask.png", tileset_name));
        let mask_img = image::open(mask_path).expect("failed to load tileset mask");
        // Load the normal map
        let normal_path = Path::new(path).join(format!("{}_normal.png", tileset_name));
        let normal_img = image::open(normal_path);
        // Check image sizes
        let template_size = (template_img.width(), template_img.height());
        let layout = if template_size == layouts.simple.template_size {
            &layouts.simple
        } else if template_size == layouts.complex.template_size {
            &layouts.complex
        } else {
            panic!("Template image must be size {:?} or {:?}, but it was {:?}.",
                   SIMPLE_TEMPLATE_IMAGE_SIZE, COMPLEX_TEMPLATE_IMAGE_SIZE, template_size);
        };
        let mask_size = (mask_img.width(), mask_img.height());
        assert_eq!(template_size, mask_size, "Template and mask images must be the same size.");
        if let Ok(img) = &normal_img {
            let normal_size = (img.width(), img.height());
            assert_eq!(template_size, normal_size, "Template and normal images must be the same size.");
        }
        // Everything's OK. Return the images.
        Tileset {
            name: tileset_name.to_owned(),
            layout,
            template_img,
            mask_img,
            normal_img: normal_img.ok(),
        }
    }

    fn tiles(&self) -> Vec<Tile> {
        self.layout.tiles().map(|tile_info| {
            let TileInfo { suffix, mask_color, center } = tile_info;
            // Create an empty tile image
            let (width, height) = self.layout.tile_size;
            let mut tile = image::ImageBuffer::new(width, height);
            // Fill out the tile, taking pixels from the tileset when the mask color matches
            let x_min = center[0] - (width / 2);
            let x_max = center[0] + (width / 2);
            let y_min = center[1] - (height / 2);
            let y_max = center[1] + (height / 2);
            for x in x_min .. x_max {
                for y in y_min .. y_max {
                    if self.mask_img.get_pixel(x, y) == *mask_color {
                        let diffuse = self.template_img.get_pixel(x, y);
                        let normal = self.normal_img.as_ref().map(|img| img.get_pixel(x, y));
                        let pixel = compute_lighting(diffuse, normal);
                        tile.put_pixel(x - x_min, y - y_min, pixel);
                    }
                }
            }
            let filename = format!("{}_{}.png", self.name, suffix);
            Tile {
                filename, 
                image: tile,
            }
        }).collect()
    }
}


// ****** Lighting ******

const SUN_POSITION: [f32; 3] = [-0.5, -0.5, 0.707];
const SUN_BRIGHTNESS: f32 = 0.9;
const AMBIENT_BRIGHTNESS: f32 = 0.1;

fn compute_lighting(diffuse: Rgba<u8>, normal: Option<Rgba<u8>>) -> Rgba<u8> {
    let brightness = if let Some(normal) = normal {
        compute_brightness(normal)
    } else {
        1.0
    };
    let brighten = |component: u8| -> u8 {
        (brightness * (component as f32)) as u8
    };
    let px = diffuse.0;
    Rgba([brighten(px[0]), brighten(px[1]), brighten(px[2]), px[3]])
}

fn compute_brightness(normal: Rgba<u8>) -> f32 {
    // Convert color component to sphere coordinate.
    fn to_sphere(c: u8) -> f32 {
        (c as f32 - 128.0) / 127.0
    }
    // Compute dot product of sphere coordinate and light coordinate, ignoring meaningless
    // negatives
    fn dot(x1: f32, x2: f32) -> f32 {
        (x1 * x2).max(0.0)
    }
    let diffuse_brightness =
        dot(SUN_POSITION[0], to_sphere(normal.0[0])) +
        dot(SUN_POSITION[1], to_sphere(normal.0[1])) +
        dot(SUN_POSITION[2], to_sphere(normal.0[2]));
    AMBIENT_BRIGHTNESS + diffuse_brightness * SUN_BRIGHTNESS
}


// ****** Main ******

const TILESET_INPUT_PATH: &'static str = "assets/raw/tilesets";
const TILESET_OUTPUT_PATH: &'static str = "assets/gen/tilesets";
// Must be kept in sync!
const MATERIALS: [&str; 4] = ["debug", "dirt", "water", "bridge"];

fn main() {
    println!("Reading tilesets from {}, and", TILESET_INPUT_PATH);
    println!("Generating tileset images in {}:", TILESET_OUTPUT_PATH);
    let layouts = TemplateLayouts::new();
    for tileset_name in &MATERIALS {
        println!("    Tileset '{}':", tileset_name);
        let tileset = Tileset::load(TILESET_INPUT_PATH, tileset_name, &layouts);
        // Split the tileset into tiles according to the mask
        for tile in tileset.tiles() {
            // Save the tile image
            let output_path = Path::new(TILESET_OUTPUT_PATH).join(&tile.filename);
            tile.image.save(output_path).expect("failed to save tiles");
            println!("        Wrote {}", tile.filename);
        }
    }
}
