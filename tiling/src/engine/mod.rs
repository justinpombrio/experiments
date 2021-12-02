mod transform;

use glutin_window::GlutinWindow;
use graphics::{clear, color, ellipse, rectangle, DrawState, Image as GlImage, Text};
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{
    Button, ButtonEvent, ButtonState, CursorEvent, MouseButton as PistonMouseButton,
    MouseCursorEvent, RenderEvent, UpdateEvent,
};
use piston::window::WindowSettings;
use std::path::Path;

pub use piston::input::Key;
pub use transform::Transform;

const FONT_SIZE: u32 = 36;
const FONT_CONSTANT: f64 = 1.5 / 0.996264;

// NOTE from Piston getting-started tutorial:
// Change this to OpenGL::V2_1 if not working.
const GL_VERSION: OpenGL = OpenGL::V3_2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseButton {
    Left,
    Right,
}

pub trait Application {
    // TODO: Might be inefficient to do _all_ rendering work inside the draw() call.
    fn render(&mut self, canvas: &mut Canvas, cursor_pos: Option<[f64; 2]>);
    fn update(&mut self, dt: f64);
    fn mouse_click(&mut self, button: MouseButton, cursor_pos: [f64; 2], window_size: [f64; 2]);
    fn key_press(&mut self, key: Key);
}

pub struct Canvas<'a> {
    pub window_size: [f64; 2],
    font_cache: &'a mut GlyphCache<'static>,
    gl: &'a mut GlGraphics,
    transform: [[f64; 3]; 2],
    textures: &'a [Texture],
}

pub struct EngineSettings {
    pub font_paths: &'static [&'static str],
}

pub struct Engine {
    gl: GlGraphics,
    window: GlutinWindow,
    font_cache: GlyphCache<'static>,
    textures: Vec<Texture>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Image(usize);

#[derive(Clone, Copy, Debug)]
pub struct Color([f32; 4]);

impl Color {
    pub fn new(hex: &str) -> Color {
        Color(color::hex(hex))
    }
}

impl<'a> Canvas<'a> {
    pub fn transform(&self) -> Transform {
        Transform::from_matrix(self.transform)
    }

    pub fn clear(&mut self, color: Color) {
        clear(color.0, self.gl)
    }

    pub fn draw_rectangle(&mut self, transform: Transform, color: Color) {
        let square = rectangle::square(0.0, 0.0, 1.0);
        rectangle(color.0, square, transform.to_matrix(), self.gl)
    }

    #[allow(unused)]
    pub fn draw_ellipse(&mut self, transform: Transform, color: Color) {
        let circle = ellipse::circle(0.5, 0.5, 0.5);
        ellipse(color.0, circle, transform.to_matrix(), self.gl)
    }

    pub fn draw_image(&mut self, transform: Transform, handle: Image) {
        let image = GlImage::new().rect(rectangle::square(0.0, 0.0, 1.0));
        let texture = &self.textures[handle.0];
        image.draw(
            texture,
            &DrawState::default(),
            transform.to_matrix(),
            self.gl,
        );
    }

    pub fn draw_text(&mut self, transform: Transform, message: &str, color: Color) {
        let text = Text::new_color(color.0, FONT_SIZE);
        let transform = transform.zoom(FONT_CONSTANT / FONT_SIZE as f64);
        let draw_state = DrawState::default();
        text.draw(
            message,
            self.font_cache,
            &draw_state,
            transform.to_matrix(),
            self.gl,
        )
        .unwrap();
    }
}

impl Engine {
    pub fn new(settings: EngineSettings) -> Engine {
        let window: GlutinWindow = WindowSettings::new("Automata", [800, 600])
            .graphics_api(GL_VERSION)
            .exit_on_esc(true)
            .resizable(true)
            .controllers(true)
            .samples(4)
            .build()
            .unwrap();

        let font_cache = {
            let font_path = settings
                .font_paths
                .into_iter()
                .map(|p| Path::new(p))
                .find(|p| p.exists());
            if let Some(font_path) = font_path {
                let texture_settings = TextureSettings::new().filter(Filter::Linear);
                GlyphCache::new(font_path, (), texture_settings).unwrap()
            } else {
                panic!("Did not find a font file at {:?}", settings.font_paths);
            }
        };

        Engine {
            window,
            gl: GlGraphics::new(GL_VERSION),
            font_cache,
            textures: vec![],
        }
    }

    pub fn load_image<P: AsRef<Path>>(&mut self, path: P) -> Image {
        let texture_settings = TextureSettings::new().filter(Filter::Nearest);
        let image =
            Texture::from_path(path.as_ref(), &texture_settings).expect("Failed to load image");
        let index = self.textures.len();
        self.textures.push(image);
        println!("Loaded image #{} at {}", index, path.as_ref().display());
        Image(index)
    }

    pub fn run_application<A: Application>(&mut self, mut app: A) {
        use piston::window::Window;

        let mut cursor_pos = None;
        let mut events = Events::new(EventSettings::new());
        while let Some(e) = events.next(&mut self.window) {
            if let Some(args) = e.render_args() {
                let context = self.gl.draw_begin(args.viewport());
                let mut canvas = Canvas {
                    window_size: args.window_size,
                    font_cache: &mut self.font_cache,
                    gl: &mut self.gl,
                    transform: context.transform,
                    textures: &self.textures,
                };
                app.render(&mut canvas, cursor_pos);
                self.gl.draw_end();
            }

            if let Some(false) = e.cursor_args() {
                cursor_pos = None;
            }

            if let Some([x, y]) = e.mouse_cursor_args() {
                cursor_pos = Some([x, y]);
            }

            if let Some(args) = e.button_args() {
                if args.state == ButtonState::Press {
                    match args.button {
                        Button::Keyboard(key) => app.key_press(key),
                        Button::Mouse(button) => {
                            if let Some(pos) = cursor_pos {
                                let size = self.window.size();
                                match button {
                                    PistonMouseButton::Left => {
                                        app.mouse_click(MouseButton::Left, pos, size.into())
                                    }
                                    PistonMouseButton::Right => {
                                        app.mouse_click(MouseButton::Right, pos, size.into())
                                    }
                                    _ => (),
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }

            if let Some(args) = e.update_args() {
                app.update(args.dt);
            }
        }
    }
}
