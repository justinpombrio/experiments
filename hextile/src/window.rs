extern crate glium;
extern crate glium_graphics;
extern crate graphics;
extern crate piston;

use std::cmp;
use std::fmt;
use std::path::Path;

use coord::*;

use self::glium_graphics::GliumGraphics;
use self::glium_graphics::{Flip, Glium2d, GliumWindow, OpenGL, TextureSettings};
use self::graphics::*;
use self::piston::input::{Button, MouseCursorEvent, PressEvent, RenderEvent};
use self::piston::window::Window as _unused;
use self::piston::window::WindowSettings;

pub use self::glium_graphics::Texture;
pub use self::piston::input::{Key, MouseButton};

pub trait UserInterface {
    fn on_mouse_move(&mut self, window: &mut Window, cursor: Pixel);
    fn on_key_press(&mut self, window: &mut Window, key: Key);
    fn on_mouse_press(&mut self, window: &mut Window, button: MouseButton);

    fn draw(&self, canvas: Canvas);
}

pub struct Window {
    window: GliumWindow,
    graphics: Glium2d,
    close: bool,
}
impl Window {
    pub fn new(name: &str, size: Pixel) -> Window {
        let opengl = OpenGL::V3_2;
        let window: GliumWindow =
            WindowSettings::new(name.to_string(), [size.x as u32, size.y as u32])
                .opengl(opengl)
                .build()
                .expect("Hextile: error opening window");
        let graphics = Glium2d::new(opengl, &window);

        Window {
            window: window,
            graphics: graphics,
            close: false,
        }
    }

    pub fn run<UI>(mut self, mut ui: UI)
    where
        UI: UserInterface,
    {
        while let Some(event) = self.window.next() {
            if self.close {
                self.window.set_should_close(true);
                break;
            }

            if let Some(renderer) = event.render_args() {
                let mut target = self.window.draw();
                self.graphics
                    .draw(&mut target, renderer.viewport(), |c, g| {
                        ui.draw(Canvas::new(c, g));
                    });
                target.finish().expect("Hextile: error drawing");
            }

            if let Some(cursor) = event.mouse_cursor_args() {
                ui.on_mouse_move(
                    &mut self,
                    Pixel::new(cursor[0] as isize, cursor[1] as isize),
                );
            }

            if let Some(Button::Keyboard(key)) = event.press_args() {
                ui.on_key_press(&mut self, key);
            }

            if let Some(Button::Mouse(button)) = event.press_args() {
                ui.on_mouse_press(&mut self, button);
            }
        }
    }

    pub fn close(&mut self) {
        self.close = true;
    }

    pub fn load_texture<P>(&mut self, path: P) -> Texture
    where
        P: AsRef<Path> + fmt::Debug + Copy,
    {
        Texture::from_path(&mut self.window, path, Flip::None, &TextureSettings::new())
            .expect(&format!("Hextile: missing texture asset {:?}", &path))
    }
}

pub struct Canvas<'a, 'b: 'a, 'c: 'a> {
    context: Context,
    graphics: &'a mut GliumGraphics<'b, 'c, glium::Frame>,
}
impl<'a, 'b, 'c> Canvas<'a, 'b, 'c> {
    pub fn new(
        context: Context,
        graphics: &'a mut GliumGraphics<'b, 'c, glium::Frame>,
    ) -> Canvas<'a, 'b, 'c> {
        Canvas {
            context: context,
            graphics: graphics,
        }
    }

    pub fn clear(&mut self, color: [f32; 4]) {
        clear(color, self.graphics)
    }

    pub fn rectangle(&mut self, color: [f32; 4], posn: [f64; 4]) {
        rectangle(color, posn, self.context.transform, self.graphics)
    }

    pub fn ellipse(&mut self, color: [f32; 4], posn: [f64; 4]) {
        ellipse(color, posn, self.context.transform, self.graphics)
    }

    pub fn circle(&mut self, color: [f32; 4], posn: Pixel, radius: isize) {
        ellipse(
            color,
            [
                (posn.x - radius) as f64,
                (posn.y - radius) as f64,
                (2 * radius) as f64,
                (2 * radius) as f64,
            ],
            self.context.transform,
            self.graphics,
        )
    }

    pub fn image(&mut self, texture: &Texture, posn: Pixel) {
        image(
            texture,
            self.context.transform.trans(posn.x as f64, posn.y as f64),
            self.graphics,
        );
    }

    pub fn tile(&mut self, texture: &Texture, screen_px: Pixel, xform_px: Pixel, size: Pixel) {
        let window_height = self.context.viewport.unwrap().draw_size[1] as isize;
        let x = screen_px.x as u32;
        let y = cmp::max(window_height - screen_px.y - size.y, 0) as u32;
        let w = size.x as u32;
        let h = size.y as u32;
        let clipped = self.context.draw_state.scissor([x, y, w, h]);
        let transform = self
            .context
            .transform
            .trans(xform_px.x as f64, xform_px.y as f64);
        Image::new().draw(texture, &clipped, transform, self.graphics);
    }
}
