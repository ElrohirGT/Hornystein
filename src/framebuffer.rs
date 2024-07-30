use nalgebra_glm::Vec3;

use crate::{are_equal, bmp::write_bmp_file, color::Color};

#[derive(Debug)]
pub struct Framebuffer {
    width: usize,
    height: usize,
    pub buffer: Buffer,
    background_color: Color,
    current_color: Color,
    empty_buffer: Vec<u32>,
}

type Buffer = Vec<u32>;

fn create_filled_buffer(width: &usize, height: &usize, color: &Color) -> Buffer {
    let color_hex: u32 = color.into();

    (0..(width * height)).map(|_| color_hex).collect()
}

#[derive(Debug)]
pub enum PaintPointErrors {
    XTooLarge,
    XTooSmall,
    YTooLarge,
    YTooSmall,
}
impl std::fmt::Display for PaintPointErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}
impl std::error::Error for PaintPointErrors {}

pub enum GetColorErrors {
    XTooLarge,
    YTooLarge,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let background_color = Color::black();
        let current_color = Color::white();

        Framebuffer {
            width,
            height,
            buffer: vec![],
            background_color,
            current_color,
            empty_buffer: create_filled_buffer(&width, &height, &Color::black()),
        }
    }

    /// Creates an empty buffer according to the corresponding `background_color`.
    ///
    /// The implementation of this method assumes the background color will not change that much.
    pub fn clear(&mut self) {
        self.buffer.clone_from(&self.empty_buffer)
    }

    /// Colors a point in the given location. Rounds x and y.
    /// If either x or y are exactly half between integers then the value is rounded up.
    ///
    /// The paint origin is located on the top left corner of the window.
    ///
    /// The color used is the one provided by `current_color`.
    pub fn paint_point(&mut self, point: glm::Vec3) -> Result<(), PaintPointErrors> {
        let Framebuffer {
            width,
            height,
            buffer,
            current_color,
            ..
        } = self;
        let x = point.x;
        let y = point.y;

        if x < 0.0 {
            Err(PaintPointErrors::XTooSmall)?
        }

        if y < 0.0 {
            Err(PaintPointErrors::YTooSmall)?
        }

        let x = x.round() as usize;
        let y = y.round() as usize;

        match (x < *width, y < *height) {
            (false, _) => Err(PaintPointErrors::XTooLarge),
            (_, false) => Err(PaintPointErrors::YTooLarge),
            _ => {
                buffer[y * *width + x] = current_color.into();
                Ok(())
            }
        }
    }

    /// Paints a line that extends from `p1` to `p2` with the color of `current_color`.
    pub fn paint_line(&mut self, p1: glm::Vec3, p2: glm::Vec3) -> Result<(), PaintPointErrors> {
        let x0 = p1.x;
        let y0 = p1.y;

        let x1 = p2.x;
        let y1 = p2.y;

        let delta_x = (x1 - x0).abs();
        let delta_y = (y1 - y0).abs();

        let dir_x = if x0 < x1 { 1.0 } else { -1.0 };
        let dir_y = if y0 < y1 { 1.0 } else { -1.0 };

        let mut err = delta_x - delta_y;

        let mut current_x = x0;
        let mut current_y = y0;

        loop {
            self.paint_point(Vec3::new(current_x, current_y, 0.0))?;

            let reached_x1 = are_equal(current_x, x1, f32::EPSILON);
            let reached_y1 = are_equal(current_y, y1, f32::EPSILON);

            if reached_x1 && reached_y1 {
                break;
            }

            let e2 = 2.0 * err;

            if e2 > -delta_y {
                err -= delta_y;
                current_x += dir_x;
            }

            if e2 < delta_x {
                err += delta_x;
                current_y += dir_y;
            }
        }

        Ok(())
    }

    /// Paints the given polygon to the screen
    pub fn paint_polygon(&mut self, mut points: Vec<glm::Vec3>) -> Result<(), PaintPointErrors> {
        match points.len() {
            1 => self.paint_point(points.remove(0)),
            _ => {
                let a = points[0];
                points.push(a);

                points
                    .windows(2)
                    .try_for_each(|ps| self.paint_line(ps[0], ps[1]))
            }
        }
    }

    /// Gets the color of a point in the buffer.
    pub fn get_color(&self, x: usize, y: usize) -> Result<Color, GetColorErrors> {
        let Framebuffer {
            width,
            height,
            buffer,
            ..
        } = self;

        match (x <= *width, y <= *height) {
            (_, false) => Err(GetColorErrors::YTooLarge),
            (false, _) => Err(GetColorErrors::XTooLarge),
            _ => Ok(buffer[y * *width + x].into()),
        }
    }

    /// Sets the `background_color` property.
    /// This method should also regenerate the `empty_buffer`.
    ///
    /// * `new_color`: The color to apply.
    pub fn set_background_color(&mut self, new_color: impl Into<Color>) {
        let Framebuffer {
            width,
            height,
            background_color,
            empty_buffer,
            ..
        } = self;

        *background_color = new_color.into();
        *empty_buffer = create_filled_buffer(width, height, background_color);
    }

    /// Sets the `current_color` property.
    ///
    /// * `new_color`: The color to apply.
    pub fn set_current_color(&mut self, new_color: impl Into<Color>) {
        self.current_color = new_color.into();
    }

    /// Saves the pixel data into a .bmp located in the given `file_path`.
    pub fn save(&self, file_path: &str) -> std::io::Result<()> {
        let Framebuffer {
            width,
            height,
            buffer,
            ..
        } = self;

        write_bmp_file(file_path, buffer, *width, *height)
    }
}
