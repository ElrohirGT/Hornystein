pub mod bmp;
pub mod color;
pub mod framebuffer;
extern crate nalgebra_glm as glm;

pub fn are_equal(first: f32, second: f32, eps: f32) -> bool {
    (first - second).abs() <= eps
}
