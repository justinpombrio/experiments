use std::collections::HashMap;

use system::*;
use coord::*;
use tsv::*;
use window::*;
use world::*;
use view::*;

use tile::*;



struct Assets {
    logo: Texture,
    hex: Texture,
    dot: Texture,
    texture: Texture,
    tile_offsets: HashMap<Tile, (Pixel, Pixel, Pixel)>
}
impl Assets{
    fn new(logo: Texture, hex: Texture, dot: Texture, texture: Texture,
           offsets: HashMap<Tile, (Pixel, Pixel, Pixel)>)
           -> Assets {
        Assets{
            logo: logo,
            hex: hex,
            dot: dot,
            texture: texture,
            tile_offsets: offsets
        }
    }
}

struct Editor {
    world:  World,
    assets: Assets,
    view:   View,
    cursor: Pixel
}
impl Editor {
    fn new(world: World, assets: Assets, view: View) -> Editor {
        Editor{
            world:  world,
            assets: assets,
            view:   view,
            cursor: Pixel::new(0, 0)
        }
    }
}
impl UserInterface for Editor {
    fn on_key_press(&mut self, window: &mut Window, key: Key) {
        if key == Key::Escape {
            window.close();
            return;
        }
        log!("Press", "{:?}", key);
    }

    fn on_mouse_press(&mut self, _: &mut Window, button: MouseButton) {
        let origin = self.view.origin;
        let hex = self.cursor.to_hex(origin);
        log!("Click", "{} {}", self.cursor, hex);
        if self.world.terrain.is_valid_hex(hex) {
            self.world.terrain[hex] = !self.world.terrain[hex];
        }
    }

    fn on_mouse_move(&mut self, _: &mut Window, cursor: Pixel) {
        self.cursor = cursor;
    }

    fn draw(&self, mut canvas: Canvas) {
        canvas.clear([1.0, 1.0, 1.0, 1.0]);

        let origin = self.view.origin;

        for hex in &self.view.positions {
            let hex = *hex;
            let px = hex.to_pixel(origin);
            if self.world.terrain.is_valid_hex(hex) {
                for orientation in 0..6 {
                    let material = self.world.terrain[hex];
                    if material {
                        let rhomb = Rhomb::new(hex, orientation);
                        let left  = if self.world.terrain[rhomb.left()]  { SameC } else { DiffC };
                        let top   = if self.world.terrain[rhomb.top()]   { SameC } else { DiffC };
                        let right = if self.world.terrain[rhomb.right()] { SameC } else { DiffC };
                        let tile  = Tile::new(orientation, 1, left, top, right);
                        let (offset, posn, size) = self.assets.tile_offsets[&tile];
                        let sprite = &self.assets.texture;
                        canvas.tile(sprite, px - offset, px - posn - offset, size);
                    }
                }
            }
        }

        let rounded_cursor = self.cursor.to_hex(origin).to_pixel(origin);
        canvas.image(&self.assets.hex, rounded_cursor - TILE_IMG_CENTER);
    }
}

pub fn play(dir: &Dir) {
    log!("Loading...");
    let world = World::new();
    let view = View::new(6);
    let mut window = Window::new("Hextile window", view.size);
    let rust_png = window.load_texture("assets/rust.png");
    let hex_png = window.load_texture("assets/hex.png");
    let dot_png = window.load_texture("assets/dot.png");
    let tile_texture: Texture = window.load_texture("assets/gen/terrain/grass/texture.png");

    let mut tile_offsets_by_name = HashMap::new();
    let mut tile_tsv: TSV = dir.dir("assets/gen/terrain/grass").load("texture.tsv");
    tile_tsv.arrange(vec!("Tile", "OffsetX", "OffsetY", "PosnX", "PosnY", "SizeX", "SizeY"));
    for row in tile_tsv.data.into_iter() {
        let name = &row[0];
        let x = parse_number(&row[1]);
        let y = parse_number(&row[2]);
        let offset = Pixel::new(x, y);
        let x = parse_number(&row[3]);
        let y = parse_number(&row[4]);
        let posn = Pixel::new(x, y);
        let x = parse_number(&row[5]);
        let y = parse_number(&row[6]);
        let size = Pixel::new(x, y);
        tile_offsets_by_name.insert(name.clone(), (offset, posn, size));
    }
    let mut tile_offsets = HashMap::new();
    for left in [SameC, DiffC].iter() {
        for right in [SameC, DiffC].iter() {
            for top in [SameC, DiffC].iter() {
                for orientation in 0..6 {
                    let material = 1; // !
                    let tile = Tile::new(orientation, material, *left, *top, *right);
                    let name = format!("{}", tile);
                    tile_offsets.insert(tile, tile_offsets_by_name[&name]);
                }
            }
        }
    }
    
    let assets = Assets::new(rust_png, hex_png, dot_png, tile_texture, tile_offsets);
    let ui = Editor::new(world, assets, view);
    log!("Playing");
    log_group(|| {
        window.run(ui);
    });
    log!("Goodbye!");
}
