use nalgebra_glm::vec2_to_vec3;

use crate::{framebuffer::Framebuffer, Board, Player};

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
}

pub fn cast_ray_3d(
    framebuffer: &mut Framebuffer,
    maze: &Board,
    player: &Player,
    orientation: f32,
) -> Intersect {
    let mut distance = 0.0;

    framebuffer.set_current_color(0x000000);
    loop {
        let cos = distance * orientation.cos();
        let sin = distance * orientation.sin();

        let x = player.position.x + cos;
        let y = player.position.y + sin;

        // println!("Checking cords at: {}, {}", x, y);

        let i = (x / maze.cell_dimensions.0) as usize;
        let j = (y / maze.cell_dimensions.1) as usize;
        let cell = maze.cells[j][i];

        // println!("Checking cell [{}] at: {}, {}", cell, x, y);
        if cell == '+' || cell == '-' || cell == '|' {
            return Intersect {
                distance,
                impact: cell,
            };
        }

        distance += 1.0;
    }
}

pub fn cast_ray_2d(framebuffer: &mut Framebuffer, maze: &Board, player: &Player, orientation: f32) {
    let mut d = 0.0;

    framebuffer.set_current_color(0x000000);
    loop {
        let cos = d * orientation.cos();
        let sin = d * orientation.sin();

        let x = player.position.x + cos;
        let y = player.position.y + sin;
        let position = nalgebra_glm::Vec2::new(x, y);

        let _ = framebuffer.paint_point(vec2_to_vec3(&position));

        // println!("Checking cords at: {}, {}", x, y);

        let i = (x / maze.cell_dimensions.0) as usize;
        let j = (y / maze.cell_dimensions.1) as usize;
        let cell = maze.cells[j][i];

        // println!("Checking cell [{}] at: {}, {}", cell, x, y);
        if cell == '+' || cell == '-' || cell == '|' {
            return;
        }

        d += 1.0;
    }
}
