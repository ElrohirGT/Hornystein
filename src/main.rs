
use hornystein::color::Color;
use hornystein::framebuffer::{self, Framebuffer};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use mouse_rs::types::Point;
use mouse_rs::Mouse;
use nalgebra_glm::{vec2_to_vec3, Vec2};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Duration;
use std::usize;

struct Board {
    cells: Vec<Vec<char>>,
    cell_dimensions: (f32, f32),
}

enum GameMode {
    TwoD,
    ThreeD,
}

struct Model {
    pub board: Board,
    pub framebuffer_dimensions: (usize, usize),
    pub player: Player,
    pub mode: GameMode,
}

struct Player {
    pub position: Vec2,
    pub orientation: f32,
    pub fov: f32,
}

enum Message {
    Move(nalgebra_glm::Vec2),
    Rotate(f32),
    TogleMode,
}

const PLAYER_SPEED: f32 = 1.5;
const PLAYER_ROTATION_SPEED: f32 = 0.005;

fn main() {
    let window_width = 800;
    let window_height = 800;

    let framebuffer_width = 1000;
    let framebuffer_height = 1000;

    let mut framebuffer = framebuffer::Framebuffer::new(framebuffer_width, framebuffer_height);

    let window_options = WindowOptions {
        resize: true,
        ..WindowOptions::default()
    };

    let mut window =
        Window::new("Hornystein", window_width, window_height, window_options).unwrap();
    window.set_key_repeat_delay(0.01);
    // window.set_cursor_visibility(false);
    let mouse = Mouse::new();

    let frame_delay = Duration::from_millis(1000 / 240);
    framebuffer.set_background_color(0x00ff00);
    framebuffer.set_current_color(0xc35817);

    let mut data = init(framebuffer_width, framebuffer_height);
    render(&mut framebuffer, &data);

    let mut previous_mouse_x = None;
    while window.is_open() {
        // listen to inputs
        if window.is_key_down(Key::Escape) {
            break;
        }

        let mut messages: Vec<Message> = window
            .get_keys_pressed(KeyRepeat::Yes)
            .into_iter()
            .filter_map(|key| match key {
                Key::W => {
                    let x_delta = PLAYER_SPEED * data.player.orientation.cos();
                    let y_delta = PLAYER_SPEED * data.player.orientation.sin();
                    Some(Message::Move(nalgebra_glm::Vec2::new(x_delta, y_delta)))
                }
                Key::S => {
                    let x_delta = PLAYER_SPEED * data.player.orientation.cos();
                    let y_delta = PLAYER_SPEED * data.player.orientation.sin();
                    Some(Message::Move(nalgebra_glm::Vec2::new(-x_delta, -y_delta)))
                }
                Key::A => Some(Message::Rotate(-PLAYER_ROTATION_SPEED)),
                Key::D => Some(Message::Rotate(PLAYER_ROTATION_SPEED)),
                Key::M => Some(Message::TogleMode),
                _ => None,
            })
            .collect();

        previous_mouse_x = match previous_mouse_x {
            Some(previous_x) => mouse.get_position().ok().map(|Point { x, y }| {
                let current_x = x as f32;
                let delta_x = current_x - previous_x;

                messages.push(Message::Rotate(PLAYER_ROTATION_SPEED * delta_x));

                let (w_width, _) = window.get_size();
                let (w_x, _) = window.get_position();
                let w_width = w_width as f32;
                let w_x = w_x as f32;

                if current_x < w_x || current_x > (w_width + w_x) {
                    let x = w_width / 2.0 + w_x;
                    mouse.move_to(x as i32, y).expect("Unable to move mouse!");
                    x
                } else {
                    current_x
                }
            }),
            None => mouse.get_position().ok().map(|Point { x, .. }| x as f32),
        };

        for msg in messages {
            data = update(data, msg);
        }
        render(&mut framebuffer, &data);

        // Update the window with the framebuffer contents
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .expect("Couldn't update the framebuffer!");

        std::thread::sleep(frame_delay);
    }
}

/// Init the default state
fn init(framebuffer_width: usize, framebuffer_height: usize) -> Model {
    let mut args = env::args();
    args.next();

    let file_name = args.next().expect("No file name received!");
    println!("Reading file name: {}", file_name);

    let file = File::open(file_name).expect("Couldn't open file!");
    let reader = BufReader::new(file);

    let cells: Vec<Vec<char>> = reader
        .lines()
        .filter_map(|line| {
            let line = line.unwrap();
            match line.trim() {
                "" => None,
                not_empty => Some(not_empty.chars().collect()),
            }
        })
        .collect();

    let maze_cell_width = framebuffer_width as f32 / cells[0].len() as f32;
    let maze_cell_height = framebuffer_height as f32 / cells.len() as f32;

    let mut player_position = extract_player_starting_position(&cells);
    player_position.x *= maze_cell_width;
    player_position.x += maze_cell_width / 2.0;

    player_position.y *= maze_cell_height;
    player_position.y += maze_cell_height / 2.0;

    let board = Board {
        cells,
        cell_dimensions: (maze_cell_width, maze_cell_height),
    };

    let player = Player {
        position: player_position,
        orientation: 0.0,
        fov: std::f32::consts::FRAC_PI_2,
    };

    let mode = GameMode::ThreeD;

    Model {
        board,
        player,
        mode,
        framebuffer_dimensions: (framebuffer_width, framebuffer_height),
    }
}

fn extract_player_starting_position(cells: &[Vec<char>]) -> nalgebra_glm::Vec2 {
    for (j, row) in cells.iter().enumerate() {
        for (i, cell) in row.iter().enumerate() {
            if cell == &'p' {
                return nalgebra_glm::Vec2::new(i as f32, j as f32);
            }
        }
    }

    nalgebra_glm::Vec2::zeros()
}

pub fn is_border(c: &char) -> bool {
    matches!(c, '+' | '|' | '-')
}

fn update(data: Model, msg: Message) -> Model {
    let Model { player, mode, .. } = data;

    match msg {
        Message::Move(delta) => {
            let mut position = player.position + delta;

            let i = (position.x / data.board.cell_dimensions.0) as usize;
            let j = (position.y / data.board.cell_dimensions.1) as usize;

            if is_border(&data.board.cells[j][i]) {
                position = player.position;
            }

            let player = Player { position, ..player };
            Model {
                player,
                mode,
                ..data
            }
        }
        Message::Rotate(delta) => {
            let orientation = player.orientation + delta;
            let player = Player {
                orientation,
                ..player
            };
            Model {
                player,
                mode,
                ..data
            }
        }
        Message::TogleMode => {
            let mode = match mode {
                GameMode::TwoD => GameMode::ThreeD,
                GameMode::ThreeD => GameMode::TwoD,
            };
            Model {
                player,
                mode,
                ..data
            }
        }
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

fn render(framebuffer: &mut Framebuffer, data: &Model) {
    framebuffer.clear();

    match data.mode {
        GameMode::TwoD => {
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
                            let _ = framebuffer
                                .paint_point(nalgebra_glm::Vec3::new(current_x, current_y, 0.0));
                            current_y += 0.5;
                        }
                        current_x += 0.5;
                    }
                });

            let num_rays = 5;
            for i in 0..num_rays {
                let curren_ray = i as f32 / num_rays as f32;
                let a = data.player.orientation - (data.player.fov / 2.0)
                    + (data.player.fov * curren_ray);

                cast_ray_2d(framebuffer, &data.board, &data.player, a);
            }

            framebuffer.set_current_color(0x0000ff);
            let _ = framebuffer.paint_point(vec2_to_vec3(&data.player.position));
        }
        GameMode::ThreeD => {
            let (framebuffer_width, framebuffer_height) = data.framebuffer_dimensions;
            let player = &data.player;
            let num_rays = framebuffer_width;
            let _half_width = framebuffer_width as f32 / 2.0;
            let half_height = framebuffer_height as f32 / 2.0;

            for i in 0..num_rays {
                let current_ray = i as f32 / num_rays as f32;
                let orientation =
                    player.orientation - (player.fov / 2.0) + (player.fov * current_ray);

                let intersect = cast_ray_3d(framebuffer, &data.board, player, orientation, false);
                let color = from_char_to_color(&intersect.impact);
                framebuffer.set_current_color(color);

                let distance_to_wall = intersect.distance;
                let distance_to_projection_plane = 6.0 * std::f32::consts::PI;

                let stake_height = (half_height / distance_to_wall) * distance_to_projection_plane;

                let stake_top = (half_height - (stake_height / 2.0)) as usize;
                let stake_bottom = (half_height + (stake_height / 2.0)) as usize;

                for y in stake_top..stake_bottom {
                    let _ =
                        framebuffer.paint_point(nalgebra_glm::Vec3::new(i as f32, y as f32, 0.0));
                }
            }
        }
    }
}

struct Intersect {
    pub distance: f32,
    pub impact: char,
}

fn cast_ray_3d(
    framebuffer: &mut Framebuffer,
    maze: &Board,
    player: &Player,
    orientation: f32,
    should_draw: bool,
) -> Intersect {
    let mut distance = 0.0;

    framebuffer.set_current_color(0x000000);
    loop {
        let cos = distance * orientation.cos();
        let sin = distance * orientation.sin();

        let x = player.position.x + cos;
        let y = player.position.y + sin;
        let position = nalgebra_glm::Vec2::new(x, y);

        if should_draw {
            let _ = framebuffer.paint_point(vec2_to_vec3(&position));
        }

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

fn cast_ray_2d(framebuffer: &mut Framebuffer, maze: &Board, player: &Player, orientation: f32) {
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
