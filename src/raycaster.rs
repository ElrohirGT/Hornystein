use nalgebra_glm::vec2_to_vec3;

use crate::{framebuffer::Framebuffer, Board, BoardCell, Model, Player};

pub struct Intersect {
    pub distance: f32,
    pub impact: BoardCell,
    pub bx: f32,
}

pub fn cast_ray_3d(framebuffer: &mut Framebuffer, data: &Model, orientation: f32) -> Intersect {
    let mut distance = 0.0;

    framebuffer.set_current_color(0x000000);
    let Model { board, player, .. } = data;
    loop {
        let cos = distance * orientation.cos();
        let sin = distance * orientation.sin();

        let x = player.position.x + cos;
        let y = player.position.y + sin;

        // println!("Checking cords at: {}, {}", x, y);

        let i = (x / board.cell_dimensions.0) as usize;
        let j = (y / board.cell_dimensions.1) as usize;
        let cell = &board.cells[j][i];

        let (block_width, block_height) = board.cell_dimensions;

        let hitx = x - i as f32 * block_width;
        let hity = y - j as f32 * block_height;

        let bx = if 1.0 < hitx && hitx < (block_width - 1.0) {
            hitx / block_width
        } else {
            // println!("Had to use hity. hitx {}, blockwidth {}", hitx, block_width);
            hity / block_height
        };

        // println!("Checking cell [{}] at: {}, {}", cell, x, y);
        match cell {
            BoardCell::HorizontalWall | BoardCell::VerticalWall | BoardCell::PillarWall => {
                return Intersect {
                    distance,
                    impact: cell.clone(),
                    bx,
                }
            }
            _ => {}
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
        let cell = &maze.cells[j][i];

        // println!("Checking cell [{}] at: {}, {}", cell, x, y);
        match cell {
            BoardCell::HorizontalWall | BoardCell::VerticalWall | BoardCell::PillarWall => {
                return;
            }
            _ => {}
        }

        d += 1.0;
    }
}
