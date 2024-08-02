use hornystein::audio::AudioPlayer;
use hornystein::enemies::LoliBunny;
use hornystein::render::{init_render, render};
use hornystein::texture::GameTextures;
use hornystein::{framebuffer, BoardCell};
use hornystein::{Board, GameMode, Message, Model, Player};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use mouse_rs::types::Point;
use mouse_rs::Mouse;
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};

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
    window.set_cursor_visibility(false);
    let mouse = Mouse::new();

    let target_framerate = 60;
    let frame_delay = Duration::from_millis(1000 / target_framerate);

    let mut data = init(framebuffer_width, framebuffer_height);
    data.audio_player.background.play();
    init_render(&mut framebuffer, &data);

    let mut previous_mouse_x = None;
    let mode_cooldown = 5;
    let mut mode_cooldown_timer = 0;

    let last_recorded_frames_max_count = 10;
    let mut last_recorded_frames = VecDeque::with_capacity(last_recorded_frames_max_count);
    while window.is_open() {
        let start = Instant::now();
        mode_cooldown_timer = (mode_cooldown_timer - 1).max(0);
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
                Key::A => Some(Message::Rotate(-PLAYER_ROTATION_SPEED * 10.0)),
                Key::D => Some(Message::Rotate(PLAYER_ROTATION_SPEED * 10.0)),
                Key::M => {
                    if mode_cooldown_timer == 0 {
                        mode_cooldown_timer = mode_cooldown;
                        Some(Message::TogleMode)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();
        messages.push(Message::TickMoon);

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

        let end = Instant::now();
        if last_recorded_frames.len() == last_recorded_frames_max_count {
            last_recorded_frames.pop_front();
        }
        last_recorded_frames.push_back((end - start).as_millis());

        let avg_millis: f32 = last_recorded_frames.iter().map(|&u| u as f32).sum::<f32>()
            / last_recorded_frames_max_count as f32;
        let avg_frames = 1000.0 / avg_millis;
        window.set_title(format!("Hornystein - {:.2} fps", avg_frames).as_ref());
        std::thread::sleep(frame_delay);
    }
}

/// Init the default state
fn init(framebuffer_width: usize, framebuffer_height: usize) -> Model {
    let mut args = env::args();
    args.next();

    let file_name = args.next().expect("No maze file name received!");
    println!("Reading file name: {}", file_name);

    let assets_dir = args.next().expect("No asset dir received!");

    println!("Loading textures from: {}...", file_name);
    let textures = GameTextures::new(&assets_dir);

    println!("Loading audios from: {}...", file_name);
    let audio_player = AudioPlayer::new(&assets_dir);

    let file = File::open(file_name).expect("Couldn't open maze file!");
    let reader = BufReader::new(file);

    let cells: Vec<Vec<BoardCell>> = reader
        .lines()
        .filter_map(|line| {
            let line = line.unwrap();
            match line.trim() {
                "" => None,
                not_empty => Some(
                    not_empty
                        .chars()
                        .filter_map(|c| {
                            Some(match c {
                                '|' => BoardCell::VerticalWall,
                                '-' => BoardCell::HorizontalWall,
                                '+' => BoardCell::PillarWall,
                                'g' => BoardCell::Goal,
                                'p' => BoardCell::Player,
                                ' ' => BoardCell::Empty,
                                _ => return None,
                            })
                        })
                        .collect(),
                ),
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

    let lolibunnies = vec![LoliBunny {
        position: nalgebra_glm::Vec2::new(250.0, 250.0),
    }];

    Model {
        board,
        player,
        mode,
        textures,
        audio_player,
        lolibunnies,
        framebuffer_dimensions: (framebuffer_width, framebuffer_height),
        moon_phase: 0.0,
    }
}

fn extract_player_starting_position(cells: &[Vec<BoardCell>]) -> nalgebra_glm::Vec2 {
    for (j, row) in cells.iter().enumerate() {
        for (i, cell) in row.iter().enumerate() {
            if cell == &BoardCell::Player {
                return nalgebra_glm::Vec2::new(i as f32, j as f32);
            }
        }
    }

    nalgebra_glm::Vec2::zeros()
}

pub fn is_border(c: &BoardCell) -> bool {
    matches!(
        c,
        BoardCell::VerticalWall | BoardCell::HorizontalWall | BoardCell::PillarWall
    )
}

fn update(data: Model, msg: Message) -> Model {
    let Model {
        player,
        mode,
        moon_phase,
        ..
    } = data;

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
        Message::TickMoon => {
            let moon_phase = (moon_phase + 5e-4).min(1.0);
            Model {
                player,
                mode,
                moon_phase,
                ..data
            }
        }
    }
}
