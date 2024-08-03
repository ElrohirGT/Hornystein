use std::time::{SystemTime, UNIX_EPOCH};

use glm::Vec3;
use nalgebra_glm::vec2_to_vec3;

use crate::{
    color::Color,
    framebuffer::Framebuffer,
    raycaster::{cast_ray_2d, cast_ray_3d},
    texture::{GameTextures, Texture},
    BoardCell, GameStatus, Model,
};

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

pub fn init_render(framebuffer: &mut Framebuffer, data: &Model) {
    let (framebuffer_width, framebuffer_height) = data.framebuffer_dimensions;

    // Render sky
    let half_height = framebuffer_height / 2;
    let color = 0x00000f;
    framebuffer.set_current_color(color);
    for i in 0..(framebuffer_width) {
        for j in 0..half_height {
            let _ = framebuffer.paint_point(nalgebra_glm::Vec3::new(i as f32, j as f32, 0.0));
        }
    }

    // Render ground
    let color = 0x140d00;
    framebuffer.set_current_color(color);
    for i in 0..(framebuffer_width) {
        for j in half_height..framebuffer_height {
            let _ = framebuffer.paint_point(nalgebra_glm::Vec3::new(i as f32, j as f32, 0.0));
        }
    }
    framebuffer.save_as_background();

    render(framebuffer, data);
}

pub fn render(framebuffer: &mut Framebuffer, data: &Model) {
    framebuffer.clear();
    render3d(framebuffer, data)
}

pub fn scale_to_fit(framebuffer: &Framebuffer, v: Vec3) -> Vec3 {
    let f_width = framebuffer.width as f32;
    let f_height = framebuffer.height as f32;

    let width = f_width * 0.2;
    let height = f_height * 0.2;

    let padding = 20.0;
    let start_x = f_width - width - padding;
    let start_y = f_height - height - padding;

    let x = v.x / f_width * width + start_x;
    let y = v.y / f_height * height + start_y;
    nalgebra_glm::Vec3::new(x, y, 0.0)
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
                    let point = scale_to_fit(
                        framebuffer,
                        nalgebra_glm::Vec3::new(current_x, current_y, 0.0),
                    );
                    let _ = framebuffer.paint_point(point);
                    current_y += 0.5;
                }
                current_x += 0.5;
            }
        });

    let num_rays = 20;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = data.player.orientation - (data.player.fov / 2.0) + (data.player.fov * current_ray);

        cast_ray_2d(framebuffer, &data.board, &data.player, a);
    }

    framebuffer.set_current_color(0xff0000);
    let half_height = 5;
    let half_width = 5;
    for bunny in &data.lolibunnies {
        let start_x = (bunny.position.x - half_width as f32) as usize;
        let start_y = (bunny.position.y - half_height as f32) as usize;

        for x in start_x..(start_x + half_width * 2) {
            for y in start_y..(start_y + half_height * 2) {
                let point = scale_to_fit(
                    framebuffer,
                    nalgebra_glm::Vec3::new(x as f32, y as f32, 0.0),
                );
                let _ = framebuffer.paint_point(point);
            }
        }
    }

    framebuffer.set_current_color(0x0000ff);
    let point = scale_to_fit(framebuffer, vec2_to_vec3(&data.player.position));
    let _ = framebuffer.paint_point(point);
}

fn render3d(framebuffer: &mut Framebuffer, data: &Model) {
    match data.status {
        GameStatus::SplashScreen => {
            let (framebuffer_width, framebuffer_height) = data.framebuffer_dimensions;
            let texture = &data.textures.splash_screen;
            let t_frame = (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards!")
                .as_millis()
                / 60)
                % texture.frame_count as u128;
            for x in 0..framebuffer_width {
                for y in 0..framebuffer_height {
                    let tx = x * texture.width as usize / framebuffer_width;
                    let ty = y * texture.height as usize / framebuffer_height;

                    let color = texture.get_pixel_color(t_frame as usize, tx as u32, ty as u32);
                    framebuffer.set_current_color(color);
                    let _ =
                        framebuffer.paint_point(nalgebra_glm::Vec3::new(x as f32, y as f32, 0.0));
                }
            }
        }
        GameStatus::MainMenu => {
            let (framebuffer_width, framebuffer_height) = data.framebuffer_dimensions;
            let texture = &data.textures.start_screen;
            for x in 0..framebuffer_width {
                for y in 0..framebuffer_height {
                    let tx = x * texture.width as usize / framebuffer_width;
                    let ty = y * texture.height as usize / framebuffer_height;

                    let color = texture.get_pixel_color(tx as u32, ty as u32);
                    framebuffer.set_current_color(color);
                    let _ =
                        framebuffer.paint_point(nalgebra_glm::Vec3::new(x as f32, y as f32, 0.0));
                }
            }
        }
        GameStatus::YouLost => {
            let (framebuffer_width, framebuffer_height) = data.framebuffer_dimensions;
            let texture = &data.textures.loose_screen;
            let t_frame = (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards!")
                .as_millis()
                / 60)
                % texture.frame_count as u128;
            for x in 0..framebuffer_width {
                for y in 0..framebuffer_height {
                    let tx = x * texture.width as usize / framebuffer_width;
                    let ty = y * texture.height as usize / framebuffer_height;

                    let color = texture.get_pixel_color(t_frame as usize, tx as u32, ty as u32);
                    framebuffer.set_current_color(color);
                    let _ =
                        framebuffer.paint_point(nalgebra_glm::Vec3::new(x as f32, y as f32, 0.0));
                }
            }
        }
        GameStatus::YouWon => {
            let (framebuffer_width, framebuffer_height) = data.framebuffer_dimensions;
            let texture = &data.textures.win_screen;
            let t_frame = (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards!")
                .as_millis()
                / 60)
                % texture.frame_count as u128;
            for x in 0..framebuffer_width {
                for y in 0..framebuffer_height {
                    let tx = x * texture.width as usize / framebuffer_width;
                    let ty = y * texture.height as usize / framebuffer_height;

                    let color = texture.get_pixel_color(t_frame as usize, tx as u32, ty as u32);
                    framebuffer.set_current_color(color);
                    let _ =
                        framebuffer.paint_point(nalgebra_glm::Vec3::new(x as f32, y as f32, 0.0));
                }
            }
        }
        GameStatus::Gaming => {
            let (framebuffer_width, framebuffer_height) = data.framebuffer_dimensions;
            render_moon(framebuffer, data);

            let _half_width = framebuffer_width as f32 / 2.0;
            let half_height = framebuffer_height as f32 / 2.0;
            let player = &data.player;

            let mut z_buffer = vec![f32::INFINITY; framebuffer_width];

            // Render 3D Screen...
            let num_rays = framebuffer_width;
            (0..num_rays).for_each(|i| {
                let current_ray = i as f32 / num_rays as f32;
                let orientation =
                    player.orientation - (player.fov / 2.0) + (player.fov * current_ray);

                let intersect = cast_ray_3d(framebuffer, data, orientation);

                if intersect.distance < z_buffer[i] {
                    z_buffer[i] = intersect.distance;
                }

                let distance_to_wall = intersect.distance;
                let distance_to_projection_plane = 6.0 * std::f32::consts::PI;

                let stake_height = (half_height / distance_to_wall) * distance_to_projection_plane;

                let stake_top = (half_height - (stake_height / 2.0)) as usize;
                let stake_bottom = (half_height + (stake_height / 2.0)) as usize;

                for y in stake_top..stake_bottom {
                    let distance_from_center = ((framebuffer.width as f32 / 2.0 - i as f32)
                        .powi(2)
                        + (framebuffer.height as f32 / 2.0 - y as f32).powi(2))
                    .sqrt();
                    let color = match from_char_to_texture(&intersect.impact, &data.textures) {
                        Some(texture) => {
                            // Calculate tx and ty.
                            // Return color from texture.
                            let ty = (y as f32 - stake_top as f32) / stake_height
                                * (texture.height as f32);
                            let tx = intersect.bx * texture.width as f32;
                            texture.get_pixel_color(tx as u32, ty as u32)
                        }
                        None => from_cell_to_color(&intersect.impact),
                    };

                    framebuffer.set_current_color(apply_lantern_effect(
                        &color,
                        distance_from_center,
                        framebuffer_width as f32,
                    ));

                    // framebuffer.set_current_color(color);
                    let _ =
                        framebuffer.paint_point(nalgebra_glm::Vec3::new(i as f32, y as f32, 0.0));
                }
            });

            // Render enemies
            render_lolibunny(framebuffer, data, &z_buffer);

            // Render HUD
            render_minimap(framebuffer, data);
        }
    }
}

fn render_minimap(framebuffer: &mut Framebuffer, data: &Model) {
    render2d(framebuffer, data);
}

fn apply_lantern_effect(color: &Color, distance_from_center: f32, framebuffer_width: f32) -> Color {
    *color
    // color.change_brightness_by((framebuffer_width / distance_from_center - 5.0).clamp(0.2, 1.0))
}

fn render_moon(framebuffer: &mut Framebuffer, data: &Model) {
    let Model {
        textures,
        moon_phase,
        ..
    } = data;

    let radius = 100.0;
    let center_x = moon_phase * framebuffer.width as f32;

    let center_y = 0.0005 * (center_x - framebuffer.width as f32 / 2.0).powi(2) + 100.0;

    let start_x = (center_x - radius) as isize;
    let start_y = (center_y - radius) as isize;

    let end_x = (center_x + radius) as isize;
    let end_y = (center_y + radius) as isize;

    let texture = &textures.moon;
    for x in start_x..end_x {
        for y in start_y..end_y {
            let distance_to_center =
                ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
            if distance_to_center <= radius {
                // let t_frame_idx = center_x as usize % textures.moon.frame_count;
                let tx = ((x - start_x) as f32 * texture.width as f32) / (radius * 2.0);
                let ty = ((y - start_y) as f32 * texture.height as f32) / (radius * 2.0);
                let color = texture.get_pixel_color(tx as u32, ty as u32);
                framebuffer.set_current_color(color);
                let _ = framebuffer.paint_point(nalgebra_glm::Vec3::new(x as f32, y as f32, 0.0));
            }
        }
    }
}

fn render_lolibunny(framebuffer: &mut Framebuffer, data: &Model, z_buffer: &[f32]) {
    let Model {
        player,
        lolibunnies,
        textures,
        ..
    } = data;
    lolibunnies.iter().for_each(|enemy| {
        let sprite_a =
            (enemy.position.y - player.position.y).atan2(enemy.position.x - player.position.x);

        let sprite_distance = ((player.position.x - enemy.position.x).powi(2)
            + (player.position.y - enemy.position.y).powi(2))
        .sqrt();

        let (framebuffer_width, framebuffer_height) = data.framebuffer_dimensions;
        let framebuffer_height = framebuffer_height as f32;
        let framebuffer_width = framebuffer_width as f32;

        let sprite_width = textures.lolibunny.width as f32;
        let sprite_height = textures.lolibunny.height as f32;

        let sprite_ratio = sprite_width / sprite_height; // width / height
        let rendered_sprite_height = (framebuffer_height / sprite_distance) * 9.0;
        let rendered_sprite_width = rendered_sprite_height * sprite_ratio;
        let start_y = ((framebuffer_height / 2.0) - (rendered_sprite_height / 2.0)) as isize;
        let start_x = ((sprite_a - player.orientation) * (framebuffer_height / player.fov)
            + (framebuffer_width / 2.0)
            - (rendered_sprite_height / 2.0)) as isize;

        let end_x = (start_x as f32 + rendered_sprite_width) as isize;
        let end_y = (start_y as f32 + rendered_sprite_height) as isize;

        for x in start_x..(end_x) {
            if sprite_distance >= z_buffer[x.clamp(0, framebuffer_width as isize - 1) as usize] {
                continue;
            }
            for y in start_y..(end_y) {
                let tx = (x as f32 - start_x as f32) * sprite_width / rendered_sprite_width;
                let ty = (y as f32 - start_y as f32) * sprite_height / rendered_sprite_height;

                let color = textures.lolibunny.get_pixel_color(tx as u32, ty as u32);
                let distance_from_center = ((x as f32 - framebuffer_width / 2.0).powi(2)
                    + (y as f32 - framebuffer_height / 2.0).powi(2))
                .sqrt();
                // framebuffer.set_current_color(color);

                framebuffer.set_current_color(apply_lantern_effect(
                    &color,
                    distance_from_center,
                    framebuffer_width,
                ));
                let _ = framebuffer.paint_point(nalgebra_glm::Vec3::new(x as f32, y as f32, 0.0));
            }
        }
    })
}
