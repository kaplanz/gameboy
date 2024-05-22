use remus::Block;
use rugby::core::dmg::{Cartridge, GameBoy};
use rugby::prelude::*;

mod common;

use common::image::{self, Result};

/// Number of cycles after which the test is ready to be checked.
const TIMEOUT: usize = 1_000_000;

/// Perform integration test emulation.
fn emulate(rom: &[u8], img: &[u8], diff: usize) -> Result<()> {
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
    let delta = image::cmp(emu.video().screen(), img).abs_diff(diff);
    let total = img.len();

    // Check for success
    image::check(delta, total)
}

macro_rules! test {
    ($($test:ident = ($diff:tt, $path:tt);)*) => {
        $(
            #[test]
            fn $test() -> Result<()> {
                let rom = include_bytes!("../roms/test/acid2/dmg-acid2.gb");
                let img = &image::png(include_bytes!($path)).unwrap();
                emulate(rom, img, $diff)
            }
        )*
    };
}

#[test]
fn success() -> Result<()> {
    let rom = include_bytes!("../roms/test/acid2/dmg-acid2.gb");
    let img = &image::png(include_bytes!("../roms/test/acid2/success.png")).unwrap();
    emulate(rom, img, 0)
}

test! {
    failure_10_obj_limit              = ( 28, "../roms/test/acid2/failures/10-obj-limit.png");
    failure_8x16_obj_tile_index_bit_0 = (256, "../roms/test/acid2/failures/8x16-obj-tile-index-bit-0.png");
    failure_bg_enable                 = (120, "../roms/test/acid2/failures/bg-enable.png");
    failure_bg_map                    = (265, "../roms/test/acid2/failures/bg-map.png");
    failure_obj_enable                = ( 64, "../roms/test/acid2/failures/obj-enable.png");
    failure_obj_horizontal_flip       = (119, "../roms/test/acid2/failures/obj-horizontal-flip.png");
    failure_obj_palette               = (144, "../roms/test/acid2/failures/obj-palette.png");
    failure_obj_priority_lower_x      = ( 12, "../roms/test/acid2/failures/obj-priority-lower-x.png");
    failure_obj_priority_same_x       = ( 12, "../roms/test/acid2/failures/obj-priority-same-x.png");
    failure_obj_size                  = (640, "../roms/test/acid2/failures/obj-size.png");
    failure_obj_to_bg_priority        = (144, "../roms/test/acid2/failures/obj-to-bg-priority.png");
    failure_obj_vertical_flip         = (868, "../roms/test/acid2/failures/obj-vertical-flip.png");
    failure_tile_sel                  = (100, "../roms/test/acid2/failures/tile-sel.png");
    failure_win_enable                = (115, "../roms/test/acid2/failures/win-enable.png");
    failure_win_line_counter          = (818, "../roms/test/acid2/failures/win-line-counter.png");
    failure_win_map                   = (256, "../roms/test/acid2/failures/win-map.png");
}
