use hornystein::color::Color;
use hornystein::framebuffer::{self, Framebuffer};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nalgebra_glm::{vec2_to_vec3, Vec2};
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::usize;

struct Board {
    cells: Vec<Vec<char>>,
    cell_dimensions: (f32, f32),
}

struct Model {
    pub board: Board,
    pub framebuffer_dimensions: (usize, usize),
    pub player: Player,
}

struct Player {
    pub position: Vec2,
    pub orientation: f32,
}

enum Message {
    Advance(nalgebra_glm::Vec2),
    Backwards(nalgebra_glm::Vec2),
    RotateClockwise(f32),
    RotateCounterClockwise(f32),
}

const PLAYER_SPEED: f32 = 1.5;
const PLAYER_ROTATION_SPEED: f32 = std::f32::consts::FRAC_PI_8 / 4.0;

fn main() {
    let window_width = 800;
    let window_height = 800;

    let framebuffer_width = 500;
    let framebuffer_height = 500;

    let mut framebuffer = framebuffer::Framebuffer::new(framebuffer_width, framebuffer_height);

    let window_options = WindowOptions {
        resize: true,
        ..WindowOptions::default()
    };

    let mut window =
        Window::new("Hornystein", window_width, window_height, window_options).unwrap();

    let mut frame_count: usize = 0;
    let frame_update_timer: usize = 1;
    framebuffer.set_background_color(0x00ff00);
    framebuffer.set_current_color(0xc35817);

    let mut data = init(framebuffer_width, framebuffer_height);
    render(&mut framebuffer, &data);

    let mut message_queue = VecDeque::with_capacity(100);
    while window.is_open() {
        // listen to inputs
        if window.is_key_down(Key::Escape) {
            break;
        }

        // Update and render the model
        if frame_count != frame_update_timer {
            frame_count += 1;
            // Update the window with the framebuffer contents
            window
                .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
                .expect("Couldn't update the framebuffer!");
            continue;
        }

        frame_count = 0;
        window
            .get_keys_pressed(KeyRepeat::Yes)
            .iter()
            .for_each(|key| match (key, message_queue.front()) {
                (Key::W, Some(Message::Advance(_))) => {}
                (Key::W, _) => {
                    let x_delta = PLAYER_SPEED * data.player.orientation.cos();
                    let y_delta = PLAYER_SPEED * data.player.orientation.sin();
                    message_queue
                        .push_back(Message::Advance(nalgebra_glm::Vec2::new(x_delta, y_delta)));
                }
                (Key::S, Some(Message::Backwards(_))) => {}
                (Key::S, _) => {
                    let x_delta = PLAYER_SPEED * data.player.orientation.cos();
                    let y_delta = PLAYER_SPEED * data.player.orientation.sin();
                    message_queue.push_back(Message::Backwards(nalgebra_glm::Vec2::new(
                        -x_delta, -y_delta,
                    )));
                }
                (Key::A, Some(Message::RotateCounterClockwise(_))) => {}
                (Key::A, _) => {
                    message_queue
                        .push_back(Message::RotateCounterClockwise(-PLAYER_ROTATION_SPEED));
                }
                (Key::D, Some(Message::RotateClockwise(_))) => {}
                (Key::D, _) => {
                    message_queue.push_back(Message::RotateClockwise(PLAYER_ROTATION_SPEED));
                }
                _ => {}
            });

        while let Some(msg) = message_queue.pop_front() {
            data = update(data, msg);
        }
        render(&mut framebuffer, &data);
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
    };

    Model {
        board,
        player,
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

fn update(data: Model, msg: Message) -> Model {
    let Model {
        board,
        framebuffer_dimensions,
        player,
    } = data;

    match msg {
        Message::Advance(delta) | Message::Backwards(delta) => {
            let position = player.position + delta;
            let player = Player { position, ..player };
            Model {
                board,
                framebuffer_dimensions,
                player,
            }
        }
        Message::RotateCounterClockwise(delta) | Message::RotateClockwise(delta) => {
            let orientation = player.orientation + delta;
            let player = Player {
                orientation,
                ..player
            };
            Model {
                board,
                framebuffer_dimensions,
                player,
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

    cast_ray_2d(framebuffer, &data.board, &data.player);

    framebuffer.set_current_color(0x0000ff);
    let _ = framebuffer.paint_point(vec2_to_vec3(&data.player.position));
}

fn ray_2d(maze: &Board, player: &Player) -> nalgebra_glm::Vec2 {
    let mut d = 0.0;

    loop {
        let cos = d * player.orientation.cos();
        let sin = d * player.orientation.sin();

        let x = player.position.x + cos;
        let y = player.position.y + sin;
        let position = nalgebra_glm::Vec2::new(x, y);

        // println!("Checking cords at: {}, {}", x, y);

        let i = (x / maze.cell_dimensions.0) as usize;
        let j = (y / maze.cell_dimensions.1) as usize;
        let cell = maze.cells[j][i];

        // println!("Checking cell [{}] at: {}, {}", cell, x, y);
        if cell == '+' || cell == '-' || cell == '|' {
            return position;
        }

        d += 1.0;
    }
}

fn cast_ray_2d(framebuffer: &mut Framebuffer, maze: &Board, player: &Player) {
    let mut d = 0.0;

    framebuffer.set_current_color(0x000000);
    loop {
        let cos = d * player.orientation.cos();
        let sin = d * player.orientation.sin();

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
