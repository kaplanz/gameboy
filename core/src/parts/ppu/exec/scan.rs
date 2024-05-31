use std::fmt::Display;

use log::{debug, trace};
use rugby_arch::mem::Memory;
use rugby_arch::reg::Register;
use rugby_arch::Word;

use super::hblank::HBlank;
use super::sprite::Sprite;
use super::vblank::VBlank;
use super::{Lcdc, Mode, Ppu};

/// Mode 2: Scan OAM.
#[derive(Clone, Debug, Default)]
pub struct Scan {
    /// OAM entry index.
    pub(super) addr: Word,
    /// Scanned sprites.
    pub(super) objs: Vec<Sprite>,
}

impl Scan {
    pub fn exec(mut self, ppu: &mut Ppu) -> Mode {
        // Extract the sprite and scanline info
        let lcdc = ppu.reg.lcdc.load();
        let size = Lcdc::ObjSize.get(&lcdc);
        let ht = [8, 16][usize::from(size)];
        let ly = ppu.reg.ly.load();

        // Scanning a single entry takes 2 dots
        if ppu.etc.dot % 2 == 0 {
            // Sprites should only be scanned when:
            //
            // 1. Objects are are enabled
            let objs_enabled = Lcdc::ObjEnable.get(&lcdc);
            // 2. Fewer than 10 sprites have been found
            let not_at_limit = self.objs.len() < 10;
            //
            // When all conditions are met, scan for sprites
            if objs_enabled && not_at_limit {
                // Read OAM entry
                let obj = [0, 1, 2, 3].map(|idx| {
                    let addr = self.addr + idx;
                    ppu.mem
                        .oam
                        .read(addr)
                        .unwrap_or_else(|_| panic!("failed to read from OAM at index: {addr}"))
                });
                // Parse sprite from bytes
                let obj = Sprite::from(obj);

                // Record sprites to be rendered that:
                //
                // 1. Are not hidden (x-coordinate is zero)
                let not_hidden = obj.xpos != 0;
                // 2. Are visible this scanline
                let is_visible = {
                    let top = obj.ypos;
                    let btm = obj.ypos.saturating_add(ht);
                    let cur = ly.saturating_add(16);
                    (top..btm).contains(&cur)
                };
                //
                // When all conditions are met, push scanned sprite
                if not_hidden && is_visible {
                    trace!("scanned sprite: {obj:?}");
                    self.objs.push(obj);
                }
            }
        }

        // Move to next OAM entry
        // NOTE: We're incrementing by 2 here since the PPU has a 16-bit wide
        //       bus to the OAM, allowing it to access one word (2 bytes) per
        //       dot.
        // <https://raw.githubusercontent.com/ISSOtm/pandocs/rendering-internals/src/Rendering_Internals.md>
        self.addr += 2;

        // Determine next mode
        if ppu.etc.dot + 1 < 80 {
            Mode::Scan(self)
        } else {
            // Enter draw
            debug!("entered mode 3: draw pixels");
            Mode::Draw(self.into())
        }
    }
}

impl Display for Scan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─────────────┐")?;
        writeln!(f, "│ {:^11} │", "Scan")?;
        writeln!(f, "├─────────────┤")?;
        writeln!(f, "│ Sprite: {:>3} │", self.addr)?;
        writeln!(f, "│ Found: {:>4} │", self.objs.len())?;
        write!(f, "└─────────────┘")
    }
}

impl From<HBlank> for Scan {
    fn from(HBlank { .. }: HBlank) -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl From<VBlank> for Scan {
    fn from(VBlank { .. }: VBlank) -> Self {
        Self {
            ..Default::default()
        }
    }
}
