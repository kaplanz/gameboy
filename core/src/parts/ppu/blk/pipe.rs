use log::trace;
use rugby_arch::reg::Register;
use rugby_arch::{Block, Byte};

use super::fetch::{self, Step};
use super::meta::{Color, Layer, Pixel, Sprite};
use super::{Lcdc, Ppu};

/// Pixel pipeline.
#[derive(Clone, Debug, Default)]
pub struct Pipeline {
    /// Warmup completed.
    pub ready: bool,
    /// Scroll offset.
    pub scroll: u8,
    /// LCD X-coordinate.
    pub lx: Byte,
    /// Background/Window channel.
    pub bgw: fetch::Background,
    /// Sprite channel.
    pub obj: fetch::Sprite,
}

impl Pipeline {
    /// Performs a fetch for the next pixels to the appropriate FIFO.
    pub fn fetch(&mut self, ppu: &mut Ppu, objs: &[Sprite]) {
        // Check if we're at an object
        if let Some(obj) = objs.iter().find(|obj| obj.xpos == self.lx + 8) {
            if ppu.lcdc(Lcdc::ObjEnable) && !self.obj.fifo.is_full() {
                trace!("found sprite: {obj:?}");
                // Fetch the sprite
                self.obj.exec(ppu, obj);
                // Stall the background fetcher
                return;
            }
        }

        // Execute the background fetcher
        self.bgw.exec(ppu);

        // Restart background fetcher when:
        //
        // 1. The first "warm-up" fetch completes
        let done_warmup = !self.ready && matches!(self.bgw.step, Step::Push { .. });
        if done_warmup {
            // We're now ready for real fetches
            self.ready = true;
            trace!("pipeline warmup complete");
        }
        // 2. The window border has been reached
        let window_reached = {
            // 1. The window is enabled
            let win_enabled = ppu.lcdc(Lcdc::WinEnable);
            // 2. Fetcher is still at the background
            let fetch_at_bg = self.bgw.layer == Layer::Background;
            // 3. Y-coordinate is below the window
            let y_below_win = ppu.reg.wy.load() <= ppu.reg.ly.load();
            // 4. X-coordinate is right of window
            let x_right_win = ppu.reg.wx.load() <= self.lx + 7;
            //
            // Determine result:
            win_enabled && fetch_at_bg && y_below_win && x_right_win
        };
        if window_reached {
            trace!(
                "window border reached at: (row: {ly}, col: {lx})",
                ly = ppu.reg.ly.load(),
                lx = self.lx
            );
            // Update the fetcher's location
            self.bgw.layer = Layer::Window;
        }
        //
        // If either condition is met...
        if done_warmup || window_reached {
            // ... reset background fetcher
            self.bgw.reset();
        }
    }

    /// Shift out a blended pixel from the FIFOs.
    pub fn shift(&mut self, ppu: &Ppu) -> Option<Pixel> {
        // Check the sprite FIFO isn't in progress
        if !matches!(self.obj.step, Step::Fetch) {
            return None;
        }

        // Pop from the background FIFO
        let mut bgwin = self.bgw.fifo.pop()?;

        // Overwrite if the background/window is disabled
        if !ppu.lcdc(Lcdc::BgWinEnable) {
            bgwin.col = Color::C0;
        }

        // Pop from the sprite FIFO
        let pixel = if let Some(sprite) = self.obj.fifo.pop() {
            Pixel::blend(bgwin, sprite) // blend the pixels together
        } else {
            bgwin // no sprite; use background/window pixel
        };

        // Discard pixels from initial scroll offset
        if self.scroll > 0 {
            // One fewer pixel to discard
            self.scroll -= 1;
            // Discard this pixel
            None
        } else {
            // Return this pixel
            Some(pixel)
        }
    }
}
