use std::{
    fs::File,
    io::{BufWriter, Write},
};

use crate::color::Color;

const DIB_HEADER_SIZE: usize = 40;
const BMP_HEADER_SIZE: usize = 54;
const BMP_PIXEL_OFFSET: usize = 54;
const BMP_BITS_PER_PIXEL: usize = 24;

/// Writes a BMP file using the buffer data and the given width and height of the image.
///
/// * `file_path`: The path of the file to generate.
/// * `buffer`: The buffer of pixel color data.
/// * `width`: The width of the image.
/// * `height`: The height of the image.
pub fn write_bmp_file(
    file_path: &str,
    buffer: &[u32],
    width: usize,
    height: usize,
) -> std::io::Result<()> {
    let writer = File::create(file_path)?;
    let mut writer = BufWriter::new(writer);
    let padded_buffer = pad_buffer(buffer, width);
    let header = generate_header(width, height, padded_buffer.len());

    // println!("Padded buffer: {:?}", padded_buffer);

    writer.write_all(&header)?;
    writer.write_all(&padded_buffer)?;
    writer.flush()
}

/// Writes a .bmp header into the given `writer`.
fn generate_header(width: usize, height: usize, data_byte_length: usize) -> Vec<u8> {
    let byte_file_size = BMP_HEADER_SIZE as u32 + data_byte_length as u32;

    // println!(
    //     "File size: {}={} + {}",
    //     byte_file_size, BMP_HEADER_SIZE, data_byte_length
    // );

    // println!(
    //     "The data length (including padding) is: {:?}\n{:?}",
    //     data_byte_length,
    //     data_byte_length.to_le_bytes()
    // );

    [&b'B', &b'M']
        .into_iter()
        .chain(&byte_file_size.to_le_bytes()[0..4])
        .chain(&[0, 0, 0, 0]) // Reserved, must be 0.
        .chain(&BMP_PIXEL_OFFSET.to_le_bytes()[0..4])
        .chain(&DIB_HEADER_SIZE.to_le_bytes()[0..4])
        .chain(&width.to_le_bytes()[0..4])
        .chain(&height.to_le_bytes()[0..4])
        .chain(&[1, 0]) // This must always be 1 and use two bytes.
        .chain(&BMP_BITS_PER_PIXEL.to_le_bytes()[0..4]) // 24 bits per pixel.
        .chain(&[0, 0]) // No compression is being used.
        .chain(&(data_byte_length as u32).to_le_bytes()[0..4]) // Image data size.
        .chain(&[0, 0, 0, 0]) // horizontal resolution (0 by default)
        .chain(&[0, 0, 0, 0]) // vertical resolution (0 by default)
        .chain(&[0, 0, 0, 0]) // the number of colors in the pallete, 0 means 2^n colors.
        // the number of IMPORTANT colors in the pallete
        // 0 means all colors are important.
        .chain(&[0, 0, 0, 0])
        .copied()
        .collect()
}

/// Formats the given buffer data to have the padding necessary according to width
fn pad_buffer(buffer: &[u32], width: usize) -> Vec<u8> {
    let padding_bytes_count = compute_padding_bytes_per_row(width);
    let padding_per_row: Vec<u8> = (0..padding_bytes_count).map(|_| 0).collect();

    let buffer: Vec<u8> = buffer
        .iter()
        .map(|c| c.into())
        .enumerate()
        .flat_map(|(i, Color { r, g, b })| {
            // The order is not a typo
            // Microsoft do be smoking...
            let vec = vec![b, g, r];

            if (i + 1) % width == 0 && i != 0 {
                vec.into_iter().chain(padding_per_row.clone()).collect()
            } else {
                vec
            }
        })
        .collect();

    // The BMP format doesn't start at the top left corner
    // instead it starts at the bottom left corner, so we need to reverse
    // the buffer by chunks to have it in the order the BMP format expects.
    buffer
        .as_slice()
        .chunks(width * 3 + padding_bytes_count)
        .rev()
        .flatten()
        .copied()
        .collect()
}

fn compute_padding_bytes_per_row(width: usize) -> usize {
    let color_bytes_per_row = width * 3;

    ((4.0 * (color_bytes_per_row as f32 / 4.0).ceil()) - color_bytes_per_row as f32).floor()
        as usize
}
