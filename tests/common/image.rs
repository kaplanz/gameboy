//! Image-based tests.

use std::fmt::{Debug, Display};

use rugby::core::dmg::ppu::{Color, Frame};
use thiserror::Error;

/// Loads a PNG image from its raw binary data.
pub fn png(data: &[u8]) -> Result<Vec<u8>, png::DecodingError> {
    // Build a reader using a decoder
    let mut decoder = png::Decoder::new(data);
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info()?;
    // Allocate the output buffer
    let mut buf = vec![0; reader.output_buffer_size()];
    // Read the next frame (an APNG might contain multiple frames)
    let info = reader.next_frame(&mut buf).unwrap();
    // Grab the bytes of the image
    let img = &buf[..info.buffer_size()];
    // Return the first frame
    Ok(img.to_vec())
}

/// Compare an LCD frame to reference frame data.
pub fn cmp(lcd: &Frame, img: &[u8]) -> usize {
    // expand pixels to bytes
    lcd.map(|pix| match pix {
        Color::C0 => 0xff,
        Color::C1 => 0xaa,
        Color::C2 => 0x55,
        Color::C3 => 0x00,
    })
    .iter()
    // compare to source image
    .zip(img)
    // filter out matching pixels
    .filter(|(a, b)| a != b)
    // count remaining differences
    .count()
}

/// Check if an image-based test has failed.
pub fn check(delta: usize, total: usize) -> Result<()> {
    if delta == 0 {
        Ok(())
    } else {
        Err(Error { delta, total })
    }
}

/// A convenient type alias for [`Result`](std::result::Result).
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error for an image-based test.
#[derive(Error)]
#[error(
    "test failed with deltas: {:.2}% ({delta}/{total})",
    100. * (*.delta as f64) / (*.total as f64)
)]
pub struct Error {
    delta: usize,
    total: usize,
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
