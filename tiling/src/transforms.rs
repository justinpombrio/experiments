use crate::engine::Transform;

// In pixels
const TILE_SIZE: [f64; 2] = [20.0, 24.0];
const SQUARE_SIZE: [f64; 2] = [10.0, 8.0];

#[derive(Debug)]
pub struct TransformArgs {
    pub window_transform: Transform,
    pub window_size: [f64; 2],
    pub camera_pos: [f64; 2],
    pub world_zoom: u32,
    pub gui_zoom: u32,
}

#[derive(Debug, Clone)]
pub struct Transforms {
    pub window_size: [f64; 2],
    pub window_half_size: [f64; 2],
    pub window_upper_left: Transform,
    pub window_to_gui: Transform,
    pub gui_to_window: Transform,
    pub window_to_square: Transform,
    pub square_to_window: Transform,
    pub square_to_tile: Transform,
}

pub fn compute_transforms(args: TransformArgs) -> Transforms {
    let window_area = args.window_size[0] * args.window_size[1];
    let gui_zoom = (window_area / args.gui_zoom as f64).sqrt();
    let world_scale = {
        let square_size = SQUARE_SIZE[0] * SQUARE_SIZE[1];
        let desired_world_area = args.world_zoom as f64 * square_size;
        let x = (window_area / desired_world_area).sqrt();
        let y = x;
        [x, y]
    };

    let id = Transform::id();
    // Window
    let window_size = args.window_size;
    let window_half_size = [window_size[0] / 2.0, window_size[1] / 2.0];
    let window_upper_left = args.window_transform;
    // GUI
    let window_to_gui = id.zoom(gui_zoom);
    let gui_to_window = window_to_gui.inverse();
    // World
    let window_to_world = id.scale_by(world_scale).flip_vert();
    let window_to_square = window_to_world.scale_by(SQUARE_SIZE);
    let square_to_window = window_to_square.inverse();
    let square_to_tile = id.scale_by_inv(SQUARE_SIZE).scale_by(TILE_SIZE).flip_vert();

    Transforms {
        window_size,
        window_half_size,
        window_upper_left,
        window_to_gui,
        gui_to_window,
        window_to_square,
        square_to_window,
        square_to_tile,
    }
}
