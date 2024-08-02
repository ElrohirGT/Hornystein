use nalgebra_glm::vec2_to_vec3;

use crate::{
    color::Color,
    framebuffer::Framebuffer,
    raycaster::{cast_ray_2d, cast_ray_3d},
    texture::Texture,
    BoardCell, GameMode, Model,
};

pub struct GameTextures {
    pub horizontal_wall: Texture,
    pub vertical_wall: Texture,
    pub corner_wall: Texture,
    pub lolibunny: Texture,
}

impl GameTextures {
    pub fn new(asset_dir: &str) -> Self {
        let horizontal_wall = format!("{}{}", asset_dir, "small_wall.jpg");
        let vertical_wall = format!("{}{}", asset_dir, "large_wall.jpg");
        let corner_wall = format!("{}{}", asset_dir, "corner.jpg");
        let lolibunny = format!("{}{}", asset_dir, "lolibunny.jpg");

        let horizontal_wall = Texture::new(&horizontal_wall);
        let vertical_wall = Texture::new(&vertical_wall);
        let corner_wall = Texture::new(&corner_wall);
        let lolibunny = Texture::new(&lolibunny);

        GameTextures {
            horizontal_wall,
            vertical_wall,
            corner_wall,
            lolibunny,
        }
    }
}

fn from_char_to_texture<'a>(c: &BoardCell, textures: &'a GameTextures) -> Option<&'a Texture> {
    match c {
        BoardCell::HorizontalWall => Some(&textures.horizontal_wall),
        BoardCell::VerticalWall => Some(&textures.vertical_wall),
        BoardCell::PillarWall => Some(&textures.corner_wall),
        _ => None,
    }
}

fn from_cell_to_color(c: &BoardCell) -> Color {
    match c {
        BoardCell::Goal => 0x8b0000,
        BoardCell::HorizontalWall | BoardCell::VerticalWall | BoardCell::PillarWall => 0xff00ff,
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

            let color = from_cell_to_color(char);
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

    framebuffer.set_current_color(0xffff00);
    let half_height = 5;
    let half_width = 5;
    for bunny in &data.lolibunnies {
        let start_x = (bunny.position.x - half_width as f32) as usize;
        let start_y = (bunny.position.y - half_height as f32) as usize;

        for x in start_x..(start_x + half_width * 2) {
            for y in start_y..(start_y + half_height * 2) {
                let _ = framebuffer.paint_point(nalgebra_glm::Vec3::new(x as f32, y as f32, 0.0));
            }
        }
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

    // Render 3D Screen...
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let orientation = player.orientation - (player.fov / 2.0) + (player.fov * current_ray);

        let intersect = cast_ray_3d(framebuffer, data, orientation);

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
                None => from_cell_to_color(&intersect.impact),
            };
            framebuffer.set_current_color(color);
            let _ = framebuffer.paint_point(nalgebra_glm::Vec3::new(i as f32, y as f32, 0.0));
        }
    }

    // Render enemies
    render_lolibunny(framebuffer, data);
}

fn render_lolibunny(framebuffer: &mut Framebuffer, data: &Model) {
    let Model {
        player,
        lolibunnies,
        textures,
        ..
    } = data;
    lolibunnies.iter().for_each(|enemy| {
        let sprite_a =
            (enemy.position.y - player.position.y).atan2(enemy.position.x - player.position.x);

        if sprite_a < 0.0 {
            return;
        }

        let sprite_distance = ((player.position.x - enemy.position.x).powi(2)
            + (player.position.y - enemy.position.y).powi(2))
        .sqrt();

        let (framebuffer_width, framebuffer_height) = data.framebuffer_dimensions;
        let framebuffer_height = framebuffer_height as f32;
        let framebuffer_width = framebuffer_width as f32;

        let sprite_width = textures.lolibunny.width as f32;
        let sprite_height = textures.lolibunny.height as f32;

        let sprite_ratio = sprite_width / sprite_height; // width / height
        let rendered_sprite_height = (framebuffer_height / sprite_distance) * 20.0;
        let rendered_sprite_width = rendered_sprite_height * sprite_ratio;
        let start_y = ((framebuffer_height / 2.0) - (rendered_sprite_height / 2.0)) as isize;
        let start_x = ((sprite_a - player.orientation) * (framebuffer_height / player.fov)
            + (framebuffer_width / 2.0)
            - (rendered_sprite_height / 2.0)) as isize;

        let end_x = (start_x as f32 + rendered_sprite_width) as isize;
        let end_y = (start_y as f32 + rendered_sprite_height) as isize;

        for x in start_x..(end_x) {
            for y in start_y..(end_y) {
                let tx = (x as f32 - start_x as f32) * sprite_width / rendered_sprite_width;
                let ty = (y as f32 - start_y as f32) * sprite_height / rendered_sprite_height;

                let color = textures.lolibunny.get_pixel_color(tx as u32, ty as u32);
                framebuffer.set_current_color(color);
                let _ = framebuffer.paint_point(nalgebra_glm::Vec3::new(x as f32, y as f32, 0.0));
            }
        }
    })
}
