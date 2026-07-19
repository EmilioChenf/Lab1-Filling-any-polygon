use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::framebuffer::Framebuffer;

/// Guarda un BMP RGB de 24 bits, sin compresion y con filas alineadas a 4 bytes.
pub fn write_bmp(path: impl AsRef<Path>, framebuffer: &Framebuffer) -> io::Result<()> {
    let width = framebuffer.width();
    let height = framebuffer.height();
    let row_size = (width * 3).div_ceil(4) * 4;
    let pixel_data_size = row_size * height;
    let file_size = 14 + 40 + pixel_data_size;
    let mut writer = BufWriter::new(File::create(path)?);

    writer.write_all(b"BM")?;
    writer.write_all(&(file_size as u32).to_le_bytes())?;
    writer.write_all(&[0; 4])?;
    writer.write_all(&54_u32.to_le_bytes())?;
    writer.write_all(&40_u32.to_le_bytes())?;
    writer.write_all(&(width as i32).to_le_bytes())?;
    writer.write_all(&(height as i32).to_le_bytes())?;
    writer.write_all(&1_u16.to_le_bytes())?;
    writer.write_all(&24_u16.to_le_bytes())?;
    writer.write_all(&0_u32.to_le_bytes())?;
    writer.write_all(&(pixel_data_size as u32).to_le_bytes())?;
    writer.write_all(&2835_i32.to_le_bytes())?;
    writer.write_all(&2835_i32.to_le_bytes())?;
    writer.write_all(&0_u32.to_le_bytes())?;
    writer.write_all(&0_u32.to_le_bytes())?;

    let padding = vec![0_u8; row_size - width * 3];
    for y in (0..height).rev() {
        for pixel in &framebuffer.pixels()[y * width..(y + 1) * width] {
            writer.write_all(&[pixel.b, pixel.g, pixel.r])?;
        }
        writer.write_all(&padding)?;
    }
    writer.flush()
}
