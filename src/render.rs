use nalgebra_glm::vec2_to_vec3;

use crate::{
    color::Color,
    framebuffer::Framebuffer,
    raycaster::{cast_ray_2d, cast_ray_3d},
    texture::Texture,
    GameMode, Model,
};

pub struct GameTextures {
    pub walls: Texture,
}

fn from_char_to_texture<'a>(c: &char, textures: &'a GameTextures) -> Option<&'a Texture> {
    match c {
        '|' | '+' | '-' => Some(&textures.walls),
        _ => None,
    }
}

fn from_char_to_color(c: &char) -> Color {
    match c {
        '|' | '+' | '-' => 0xff00ff,
        // 'p' => 0x00008b,
        'g' => 0x8b0000,
        _ => 0xffffff,
    }
    .into()
}

pub fn render(framebuffer: &mut Framebuffer, data: &Model) {
    framebuffer.clear();

    match data.mode {
        GameMode::TwoD => render2d(framebuffer, data),
        GameMode::ThreeD => render3d(framebuffer, data),
    }
}

fn render2d(framebuffer: &mut Framebuffer, data: &Model) {
    let (maze_cell_width, maze_cell_height) = data.board.cell_dimensions;
    data.board
        .cells
        .iter()
        .enumerate()
        .flat_map(|(j, row)| {
            row.iter()
                .enumerate()
                .map(move |(i, cell)| (i as f32, j as f32, cell))
        })
        .for_each(|(i, j, char)| {
            let mut current_x = i * maze_cell_width;
            let start_y = j * maze_cell_height;

            let end_x = current_x + maze_cell_width;
            let end_y = start_y + maze_cell_height;

            let color = from_char_to_color(char);
            framebuffer.set_current_color(color);

            while current_x < end_x {
                let mut current_y = start_y;
                while current_y < end_y {
                    let _ =
                        framebuffer.paint_point(nalgebra_glm::Vec3::new(current_x, current_y, 0.0));
                    current_y += 0.5;
                }
                current_x += 0.5;
            }
        });

    let num_rays = 5;
    for i in 0..num_rays {
        let curren_ray = i as f32 / num_rays as f32;
        let a = data.player.orientation - (data.player.fov / 2.0) + (data.player.fov * curren_ray);

        cast_ray_2d(framebuffer, &data.board, &data.player, a);
    }

    framebuffer.set_current_color(0x0000ff);
    let _ = framebuffer.paint_point(vec2_to_vec3(&data.player.position));
}

fn render3d(framebuffer: &mut Framebuffer, data: &Model) {
    let (framebuffer_width, framebuffer_height) = data.framebuffer_dimensions;
    let num_rays = framebuffer_width;

    let _half_width = framebuffer_width as f32 / 2.0;
    let half_height = framebuffer_height as f32 / 2.0;
    let player = &data.player;

    // Render ground
    for i in 0..(framebuffer_width) {
        let ground_color = 0x383838;
        framebuffer.set_current_color(ground_color);
        for j in (half_height as usize)..framebuffer_height {
            let _ = framebuffer.paint_point(nalgebra_glm::Vec3::new(i as f32, j as f32, 0.0));
        }
    }

    // Render 3D Screen...
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let orientation = player.orientation - (player.fov / 2.0) + (player.fov * current_ray);

        let intersect = cast_ray_3d(framebuffer, &data.board, player, orientation);

        let distance_to_wall = intersect.distance;
        let distance_to_projection_plane = 6.0 * std::f32::consts::PI;

        // let distance_to_wall = intersect.distance * (orientation - player.orientation).cos();
        let stake_height = (half_height / distance_to_wall) * distance_to_projection_plane;

        let stake_top = (half_height - (stake_height / 2.0)) as usize;
        let stake_bottom = (half_height + (stake_height / 2.0)) as usize;

        for y in stake_top..stake_bottom {
            let color = match from_char_to_texture(&intersect.impact, &data.textures) {
                Some(texture) => {
                    // Calculate tx and ty.
                    // Return color from texture.
                    let ty = (y as f32 - stake_top as f32) / stake_height * (texture.height as f32);
                    let tx = intersect.bx * texture.width as f32;
                    texture.get_pixel_color(tx as u32, ty as u32)
                }
                None => from_char_to_color(&intersect.impact),
            };
            framebuffer.set_current_color(color);
            let _ = framebuffer.paint_point(nalgebra_glm::Vec3::new(i as f32, y as f32, 0.0));
        }
    }
}
