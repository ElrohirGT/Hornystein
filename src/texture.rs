use image::{GenericImageView, ImageReader, Pixel};

use crate::color::Color;

pub struct Texture {
    pub width: u32,
    pub height: u32,
    colors: Vec<Color>,
}

impl Texture {
    pub fn new(file_path: &str) -> Self {
        let image = ImageReader::open(file_path).unwrap().decode().unwrap();
        let width = image.width();
        let height = image.height();

        let size = width * height;
        let mut colors = vec![0xffffff.into(); size as usize];

        // If I use flatmap and all that this get's reordered...
        // I don't know why
        for x in 0..width {
            for y in 0..height {
                let pixel = image.get_pixel(x, y).to_rgb();
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];

                let idx = y * width + x;
                colors[idx as usize] = Color { r, g, b };
            }
        }

        Texture {
            width,
            height,
            colors,
        }
    }

    pub fn get_pixel_color(&self, x: u32, y: u32) -> Color {
        let idx = y * self.width + x;
        self.colors[idx as usize]
    }
}
