use std::collections::HashMap;

pub mod engine;
pub mod tiling;
pub mod transforms;
pub mod world;

use engine::{Application, Canvas, Color, Engine, EngineSettings, Image, Key, MouseButton};
use tiling::{Link, TileKey, Tiles};
use transforms::{compute_transforms, TransformArgs, Transforms};
use world::{demo_world, Block, Layer, SqCoord, Square, World};

const BACKGROUND_COLOR: &str = "222222";
const GUI_BACKGROUND_COLOR_1: &str = "333333";
const GUI_BACKGROUND_COLOR_2: &str = "3f3f3f";
const GUI_TEXT_COLOR: &str = "ffffff";

const NUM_WORLD_SQUARES: u32 = 100;
const NUM_GUI_SQUARES: u32 = 10000;
const NUM_GUI_LINES: usize = 24;

struct App {
    world: World,
    tiles: Tiles,
    sprites: HashMap<String, Image>,
    block_mode: Block,
    log: Vec<String>,
    info: HashMap<String, String>,
    transforms: Option<Transforms>,
    camera_pos: [f64; 2],
    cached_block_keys: HashMap<char, Block>,
}

impl Application for App {
    fn render(&mut self, canvas: &mut Canvas, cursor_pos: Option<[f64; 2]>) {
        let ts = compute_transforms(TransformArgs {
            window_transform: canvas.transform(),
            window_size: canvas.window_size,
            camera_pos: self.camera_pos,
            world_zoom: NUM_WORLD_SQUARES,
            gui_zoom: NUM_GUI_SQUARES,
        });
        self.transforms = Some(ts.clone());

        // Clear the canvas
        canvas.clear(Color::new(BACKGROUND_COLOR));

        // Draw each tile
        for coord in self.world.all_tile_coords() {
            let t = ts
                .window_upper_left
                .trans_by(ts.window_half_size)
                .then(ts.window_to_square)
                .trans(coord[0] as f64, coord[1] as f64)
                .trans(0.5, 0.5)
                .then(ts.square_to_tile)
                .centered();
            let squares = self.world.get_squares_around_tile(coord);
            for layer in Layer::all_layers() {
                let mut blocks = vec![];
                for block in squares.iter().filter_map(|sq| sq.get_layer(*layer)) {
                    if !blocks.contains(&block) {
                        blocks.push(block);
                    }
                }
                for tile in self.get_tiles(&blocks, squares) {
                    canvas.draw_image(t, tile);
                }
            }
            // canvas.draw_image(t, self.hex_images["debug"]);
        }

        // Highlight the selected square
        let selected_sq = if let Some([x, y]) = cursor_pos {
            let [p, q] = ts
                .square_to_window
                .trans_by_neg(ts.window_half_size)
                .trans(x, y)
                .point();
            Some([p.round() as i32, q.round() as i32])
        } else {
            None
        };
        if let Some([p, q]) = selected_sq {
            let t = ts
                .window_upper_left
                .trans_by(ts.window_half_size)
                .then(ts.window_to_square)
                .trans(p as f64, q as f64)
                .then(ts.square_to_tile)
                .centered();
            canvas.draw_image(t, self.sprites["selected"]);
        };

        // Draw backgound for the log
        for i in 0..NUM_GUI_LINES {
            let t = ts
                .window_upper_left
                .then(ts.window_to_gui)
                .scale(24.0, 2.0)
                .trans(0.0, i as f64);
            let color = if i % 2 == 0 {
                Color::new(GUI_BACKGROUND_COLOR_1)
            } else {
                Color::new(GUI_BACKGROUND_COLOR_2)
            };
            canvas.draw_rectangle(t, color);
        }

        // Prepare to show log messages
        let mut row = 0;
        let mut show_message = |msg: &str, row: usize| {
            let t = ts
                .window_upper_left
                .then(ts.window_to_gui)
                .trans(0.0, 2.0 * row as f64 + 1.6);
            let color = Color::new(GUI_TEXT_COLOR);
            canvas.draw_text(t, msg, color);
        };

        // Display each `info` in the log
        self.info("block", self.block_mode.name());
        for (key, value) in &self.info {
            let message = format!("{}: {}", key, value);
            show_message(&message, row);
            row += 1;
        }
        show_message("------------------------", row);
        row += 1;
        // Display each log message in the log
        let remaining_lines = NUM_GUI_LINES.saturating_sub(row);
        let num_to_skip = self.log.len().saturating_sub(remaining_lines);
        let log_msgs = self.log.iter().skip(num_to_skip);
        for msg in log_msgs {
            show_message(msg, row);
            row += 1;
        }
    }

    fn update(&mut self, dt: f64) {
        let fps = (1.0 / dt) as usize;
        self.info("FPS", format!("{}", fps));
    }

    fn mouse_click(&mut self, button: MouseButton, cursor_pos: [f64; 2], _window_size: [f64; 2]) {
        let ts = match &self.transforms {
            None => return,
            Some(ts) => ts,
        };
        let [p, q] = ts
            .square_to_window
            .trans_by_neg(ts.window_half_size)
            .trans_by(cursor_pos)
            .point();
        let selected_square = [p.round() as i32, q.round() as i32];
        match button {
            MouseButton::Left => self.add_block(selected_square),
            MouseButton::Right => self.remove_block(selected_square),
        }
        let (x, y) = (cursor_pos[0] as u32, cursor_pos[1] as u32);
        self.log(format!("{:?} click at {},{}", button, x, y));
    }

    fn key_press(&mut self, key: Key) {
        self.log(format!("Key press {:?}", key));
        match key {
            // TODO: need a way to turn Key into letter
            Key::Z => self.block_mode = self.cached_block_keys[&'z'],
            Key::D => self.block_mode = self.cached_block_keys[&'d'],
            Key::G => self.block_mode = self.cached_block_keys[&'g'],
            Key::W => self.block_mode = self.cached_block_keys[&'w'],
            Key::B => self.block_mode = self.cached_block_keys[&'b'],
            Key::Left => self.camera_pos[0] -= 0.5,
            Key::Right => self.camera_pos[0] += 0.5,
            Key::Up => self.camera_pos[1] += 0.5,
            Key::Down => self.camera_pos[1] -= 0.5,
            _ => (),
        }
    }
}

impl App {
    fn info<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.info.insert(key.into(), value.into());
    }

    fn log<S: Into<String>>(&mut self, message: S) {
        let message = message.into();
        println!("{}", &message);
        self.log.push(message);
    }

    fn add_block(&mut self, pos: SqCoord) {
        if let Some(square) = self.world.get_square_mut(pos) {
            square.set_block(self.block_mode);
        }
    }

    fn remove_block(&mut self, pos: SqCoord) {
        if let Some(square) = self.world.get_square_mut(pos) {
            square.remove_block(self.block_mode);
        }
    }

    fn get_tiles(&self, blocks: &[Block], squares: [&Square; 4]) -> Vec<Image> {
        fn link(block: Block, square: &Square) -> Link {
            if block == Block::Bridge {
                if square.has_block(Block::Bridge) {
                    Link::Full
                } else if square.has_block(Block::Water) {
                    Link::Empty
                } else if square.has_block(Block::Dirt) {
                    Link::Half
                } else {
                    Link::Empty
                }
            } else if square.has_block(block) {
                // Same kind of block
                Link::Full
            } else {
                // Different kind of block
                Link::Empty
            }
        }
        let [a, b, c, d] = squares;
        let mut keys = vec![];
        for &block in blocks {
            let links = [
                link(block, a),
                link(block, b),
                link(block, c),
                link(block, d),
            ];
            keys.push(TileKey { block, links });
        }
        self.tiles.get_tiles(&keys)
    }
}

pub fn run_game() {
    let font_paths = &["assets/triplicate.ttf", "assets/inconsolata.ttf"];
    let settings = EngineSettings { font_paths };
    let mut engine = Engine::new(settings);

    let mut sprites = HashMap::new();
    sprites.insert(
        "selected".to_string(),
        engine.load_image("assets/squares/selected.png"),
    );
    let mut tiles = Tiles::new();
    tiles.load(&mut engine, "assets/gen/tilesets");

    let world = demo_world();
    let app = App {
        world,
        block_mode: Block::Debug,
        log: vec![],
        info: HashMap::new(),
        sprites,
        tiles,
        transforms: None,
        camera_pos: [0.0, 0.0],
        cached_block_keys: Block::keys(),
    };

    engine.run_application(app);
}
