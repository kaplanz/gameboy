#![allow(non_snake_case)]

use std::fmt::{Debug, Display};

use png::Transformations;
use remus::Machine;
use rugby::core::dmg::cart::Cartridge;
use rugby::core::dmg::GameBoy;
use rugby::emu::cart::Support as _;
use rugby::pal::{self, Palette};
use thiserror::Error;

type Result<T, E = Error> = std::result::Result<T, E>;

const TIMEOUT: usize = 1_000_000;
const PALETTE: Palette = pal::MONO;

fn image(data: &[u8]) -> Result<Vec<u8>, png::DecodingError> {
    // Build a reader using a decoder
    let mut decoder = png::Decoder::new(data);
    decoder.set_transformations(Transformations::EXPAND);
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

fn emulate(rom: &[u8], img: &[u8]) -> Result<()> {
    // Instantiate a cartridge
    let cart = Cartridge::new(rom).unwrap();
    // Create an emulator instance
    let mut emu = GameBoy::new();
    // Load the cartridge
    emu.load(cart);

    // Loop until timeout
    for _ in 0..TIMEOUT {
        emu.cycle();
    }
    // Calculate difference
    let delta = compare(&emu, img);
    let total = img.len();

    // Check for success
    Match { delta, total }.check()
}

fn compare(emu: &GameBoy, img: &[u8]) -> usize {
    // Extract frame buffer
    let lcd = emu
        .ppu()
        .screen()
        // convert pixels to bytes
        .map(|pix| u32::from(PALETTE[pix as usize]) as u8);
    // Compare distance to expected
    lcd.iter().zip(img).filter(|(a, b)| a != b).count()
}

#[derive(Error)]
enum Error {
    #[error("failed test: {0}")]
    Failed(Match),
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Debug)]
struct Match {
    delta: usize,
    total: usize,
}

impl Match {
    fn check(self) -> Result<()> {
        if self.delta == 0 {
            Ok(())
        } else {
            Err(Error::Failed(self))
        }
    }
}

impl Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "match rate: {:.2}%, delta: {}/{}",
            100. * (self.total - self.delta) as f64 / self.total as f64,
            self.delta,
            self.total
        )
    }
}

macro_rules! test {
    ($($test:ident,)*) => {
        $(
            #[test]
            fn $test() -> Result<()> {
                let rom = include_bytes!(concat!(
                    "../roms/test/mealybug/",
                    stringify!($test),
                    ".gb",
                ));
                let img = &image(include_bytes!(concat!(
                    "../roms/test/mealybug/expected/",
                    stringify!($test),
                    ".png",
                )))
                .unwrap();
                emulate(rom, img)
            }
        )*
    };
}

test! {
    m2_win_en_toggle,
    m3_bgp_change,
    m3_bgp_change_sprites,
    m3_lcdc_bg_en_change,
    m3_lcdc_bg_map_change,
    m3_lcdc_obj_en_change,
    m3_lcdc_obj_en_change_variant,
    m3_lcdc_obj_size_change,
    m3_lcdc_obj_size_change_scx,
    m3_lcdc_tile_sel_change,
    m3_lcdc_tile_sel_win_change,
    m3_lcdc_win_en_change_multiple,
    m3_lcdc_win_en_change_multiple_wx,
    m3_lcdc_win_map_change,
    m3_obp0_change,
    m3_scx_high_5_bits,
    m3_scx_low_3_bits,
    m3_scy_change,
    m3_window_timing,
    m3_window_timing_wx_0,
    m3_wx_4_change,
    m3_wx_4_change_sprites,
    m3_wx_5_change,
    m3_wx_6_change,
}
