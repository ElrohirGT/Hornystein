use image::{DynamicImage, GenericImageView, ImageReader, Pixel};

use crate::color::Color;

pub struct Texture {
    image: DynamicImage,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(file_path: &str) -> Self {
        let image = ImageReader::open(file_path).unwrap().decode().unwrap();
        let width = image.width();
        let height = image.height();

        Texture {
            image,
            width,
            height,
        }
    }

    pub fn get_pixel_color(&self, x: u32, y: u32) -> Color {
        let pixel = self.image.get_pixel(x, y).to_rgb();
        let r = pixel[0] as u32;
        let g = pixel[1] as u32;
        let b = pixel[2] as u32;

        ((r << 16) | (g << 8) | b).into()
        // Color { r, g, b}
    }
}
