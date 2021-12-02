// An dumbified wrapper around image //

extern crate image;
use self::image::{ImageBuffer};
use std::ops::{Index, IndexMut};

use coord::*;
use system::*;


pub type Color  = image::Rgba<u8>;


pub struct PNG {
    buffer: image::RgbaImage
}
impl PNG {
    pub fn new(width: u32, height: u32) -> PNG {
        PNG{
            buffer: ImageBuffer::new(width, height)
        }
    }
}

impl Loadable for PNG {
    fn load(file: &mut RFile) -> PNG {
        let img = match image::load(file, image::PNG) {
            Ok(img)  => img,
            Err(err) => panic!("PNG: Failed to load image. {}", err)
        };
        PNG{
            buffer: img.to_rgba()
        }
    }
}

impl Savable for PNG {
    fn save(self, file: &mut WFile) {
        let img = image::ImageRgba8(self.buffer);
        match img.save(file, image::PNG) {
            Ok(()) => (),
            Err(err) => panic!("PNG: Failed to save image. {}", err)
        }
    }
}

impl Index<Pixel> for PNG {
    type Output = Color;
    fn index(&self, px: Pixel) -> &Color {
        self.buffer.get_pixel(px.x as u32, px.y as u32)
    }
}

impl IndexMut<Pixel> for PNG {
    fn index_mut(&mut self, px: Pixel) -> &mut Color {
        self.buffer.get_pixel_mut(px.x as u32, px.y as u32)
    }
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    image::Rgba([r, g, b, a])
}
