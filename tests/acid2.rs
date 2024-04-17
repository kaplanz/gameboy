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
    ($($test:ident = $path:tt;)*) => {
        $(
            #[test]
            #[should_panic]
            fn $test() {
                let rom = include_bytes!("../roms/test/acid2/dmg-acid2.gb");
                let img = &image(include_bytes!($path)).unwrap();
                emulate(rom, img).unwrap()
            }
        )*
    };
}

#[test]
fn dmg_acid2() -> Result<()> {
    let rom = include_bytes!("../roms/test/acid2/dmg-acid2.gb");
    let img = &image(include_bytes!("../roms/test/acid2/success.png")).unwrap();
    emulate(rom, img)
}

test! {
    dmg_acid2_10_obj_limit              = "../roms/test/acid2/failures/10-obj-limit.png";
    dmg_acid2_8x16_obj_tile_index_bit_0 = "../roms/test/acid2/failures/8x16-obj-tile-index-bit-0.png";
    dmg_acid2_bg_enable                 = "../roms/test/acid2/failures/bg-enable.png";
    dmg_acid2_bg_map                    = "../roms/test/acid2/failures/bg-map.png";
    dmg_acid2_obj_enable                = "../roms/test/acid2/failures/obj-enable.png";
    dmg_acid2_obj_horizontal_flip       = "../roms/test/acid2/failures/obj-horizontal-flip.png";
    dmg_acid2_obj_palette               = "../roms/test/acid2/failures/obj-palette.png";
    dmg_acid2_obj_priority_lower_x      = "../roms/test/acid2/failures/obj-priority-lower-x.png";
    dmg_acid2_obj_priority_same_x       = "../roms/test/acid2/failures/obj-priority-same-x.png";
    dmg_acid2_obj_size                  = "../roms/test/acid2/failures/obj-size.png";
    dmg_acid2_obj_to_bg_priority        = "../roms/test/acid2/failures/obj-to-bg-priority.png";
    dmg_acid2_obj_vertical_flip         = "../roms/test/acid2/failures/obj-vertical-flip.png";
    dmg_acid2_tile_sel                  = "../roms/test/acid2/failures/tile-sel.png";
    dmg_acid2_win_enable                = "../roms/test/acid2/failures/win-enable.png";
    dmg_acid2_win_line_counter          = "../roms/test/acid2/failures/win-line-counter.png";
    dmg_acid2_win_map                   = "../roms/test/acid2/failures/win-map.png";
}
