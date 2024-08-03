use std::{fs::File, io::BufReader};

use image::{
    codecs::gif::GifDecoder, AnimationDecoder, Frame, GenericImageView, ImageDecoder, ImageReader,
    Pixel,
};

use crate::color::Color;

pub struct GameTextures {
    pub horizontal_wall: Texture,
    pub vertical_wall: Texture,
    pub corner_wall: Texture,
    pub lolibunny: Texture,
    pub moon: Texture,
    pub start_screen: Texture,
    pub loose_screen: AnimatedTexture,
    pub win_screen: AnimatedTexture,
}

impl GameTextures {
    pub fn new(asset_dir: &str) -> Self {
        let horizontal_wall = format!("{}{}", asset_dir, "small_wall.jpg");
        let vertical_wall = format!("{}{}", asset_dir, "large_wall.jpg");
        let corner_wall = format!("{}{}", asset_dir, "corner.jpg");
        let lolibunny = format!("{}{}", asset_dir, "lolibunny.jpg");
        let moon = format!("{}{}", asset_dir, "moon.jpg");
        let start_screen = format!("{}{}", asset_dir, "start_screen.jpg");
        let loose_screen = format!("{}{}", asset_dir, "loose_screen.gif");
        let win_screen = format!("{}{}", asset_dir, "win_screen.gif");

        let horizontal_wall = Texture::new(&horizontal_wall);
        let vertical_wall = Texture::new(&vertical_wall);
        let corner_wall = Texture::new(&corner_wall);
        let lolibunny = Texture::new(&lolibunny);
        let start_screen = Texture::new(&start_screen);
        let loose_screen = AnimatedTexture::new(&loose_screen);
        let win_screen = AnimatedTexture::new(&win_screen);

        let moon = Texture::new(&moon);

        GameTextures {
            horizontal_wall,
            vertical_wall,
            corner_wall,
            lolibunny,
            moon,
            start_screen,
            loose_screen,
            win_screen,
        }
    }
}

pub struct Texture {
    pub width: u32,
    pub height: u32,
    colors: Vec<Color>,
}

pub struct AnimatedTexture {
    pub width: u32,
    pub height: u32,
    frames: Vec<Frame>,
    pub frame_count: usize,
}

impl AnimatedTexture {
    pub fn new(file_path: &str) -> Self {
        let file_in = BufReader::new(File::open(file_path).unwrap());
        let decoder = GifDecoder::new(file_in).unwrap();
        let (width, height) = decoder.dimensions();
        let frames = decoder.into_frames();
        let frames = frames.collect_frames().expect("error decoding gif");
        let frame_count = frames.len();

        Self {
            width,
            height,
            frames,
            frame_count,
        }
    }

    /// Get's the color of the pixel positioned on the frame `t`.
    pub fn get_pixel_color(&self, t: usize, x: u32, y: u32) -> Color {
        let pixel = self.frames[t].buffer().get_pixel(x, y).to_rgb();
        let r = pixel[0];
        let g = pixel[1];
        let b = pixel[2];

        Color { r, g, b }
    }
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
