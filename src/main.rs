use hornystein::color::Color;
use hornystein::framebuffer::{self, Framebuffer};
use minifb::{Key, Window, WindowOptions};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::usize;

struct Model {
    pub board: Vec<Vec<char>>,
    pub framebuffer_dimensions: (usize, usize),
}

fn main() {
    let window_width = 800;
    let window_height = 800;

    let framebuffer_width = 500;
    let framebuffer_height = 500;

    let mut framebuffer = framebuffer::Framebuffer::new(framebuffer_width, framebuffer_height);

    let default = WindowOptions {
        resize: true,
        ..WindowOptions::default()
    };

    let mut window = Window::new("Hornystein", window_width, window_height, default).unwrap();

    let mut frame_count: u64 = 0;
    let frame_update_timer: u64 = 6000;
    framebuffer.set_background_color(0x00ff00);
    framebuffer.set_current_color(0xc35817);

    let mut data = init(framebuffer_width, framebuffer_height);
    render(&mut framebuffer, &data);
    while window.is_open() {
        // listen to inputs
        if window.is_key_down(Key::Escape) {
            break;
        }

        // Update the model
        if frame_count == frame_update_timer {
            frame_count = 0;
            data = update(data);
            // Render
            render(&mut framebuffer, &data);
        }

        // Update the window with the framebuffer contents
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        frame_count += 1;
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

    let board = reader
        .lines()
        .filter_map(|line| {
            let line = line.unwrap();
            match line.trim() {
                "" => None,
                not_empty => Some(not_empty.chars().collect()),
            }
        })
        .collect();

    Model {
        board,
        framebuffer_dimensions: (framebuffer_width, framebuffer_height),
    }
}

fn update(data: Model) -> Model {
    data
}

fn from_char_to_color(c: &char) -> Color {
    match c {
        '|' | '+' | '-' => 0xff00ff,
        'p' => 0x00008b,
        'g' => 0x8b0000,
        _ => 0xffffff,
    }
    .into()
}

fn render(framebuffer: &mut Framebuffer, data: &Model) {
    framebuffer.clear();

    let maze_cell_width = data.framebuffer_dimensions.0 as f32 / data.board[0].len() as f32;
    let maze_cell_height = data.framebuffer_dimensions.1 as f32 / data.board.len() as f32;

    data.board
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

            while current_x < end_x {
                let mut current_y = start_y;
                while current_y < end_y {
                    let color = from_char_to_color(char);
                    framebuffer.set_current_color(color);

                    let _ =
                        framebuffer.paint_point(nalgebra_glm::Vec3::new(current_x, current_y, 0.0));
                    current_y += 0.5;
                }
                current_x += 0.5;
            }
        });
}
