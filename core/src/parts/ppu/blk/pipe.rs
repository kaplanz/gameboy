use rugby_arch::reg::Register;
use rugby_arch::Byte;

use super::fetch::{Fetch, Location, Stage};
use super::fifo::Fifo;
use super::pixel::{Color, Pixel};
use super::sprite::Sprite;
use super::{Lcdc, Ppu};

/// PPU's pixel pipeline.
#[derive(Clone, Debug, Default)]
pub struct Pipeline {
    ready: bool,
    discard: Byte,
    xpos: Byte,
    bgwin: Channel,
    sprite: Channel,
}

impl Pipeline {
    /// Gets the pipeline's current x-position.
    #[must_use]
    pub fn xpos(&self) -> Byte {
        self.xpos
    }

    /// Sets the number of pixels to be discarded by the pipeline.
    pub fn set_discard(&mut self, discard: Byte) {
        self.discard = discard;
    }

    /// Performs a fetch for the next pixels to the appropriate FIFO.
    pub fn fetch(&mut self, ppu: &mut Ppu, objs: &[Sprite]) {
        // Check if we're at an object
        if self.sprite.fifo.len() < 8 {
            if let Some(obj) = objs.iter().find(|obj| obj.xpos == self.xpos + 8) {
                // Configure and fetch the sprite
                self.sprite.fetch(ppu, Some(obj.clone()));

                // Return early (stall the Background fetch)
                return;
            }
        }

        // Cycle the background fetcher when its empty
        if self.bgwin.fifo.is_empty() {
            self.bgwin.fetch(ppu, None);
        }

        // Restart background fetcher when:
        // - The first "warm-up" fetch completes
        let done_warmup = !self.ready && matches!(self.bgwin.fetch.stage(), Stage::Push(_, _));
        if done_warmup {
            // Configure channels (only run once)
            self.bgwin.loc = Location::Background;
            self.sprite.loc = Location::Sprite;
            // We're now ready for real fetches
            self.ready = true;
        }
        // - The window border is encountered
        let reached_window = !self.within_win() && self.found_win(ppu);
        if reached_window {
            // Mark the location as in the window
            self.bgwin.loc = Location::Window;
            // Clear the background FIFO
            self.bgwin.fifo.clear();
        }
        // Perform the reset
        if done_warmup || reached_window {
            self.bgwin.fetch = Fetch::default();
        }
    }

    /// Shift out a blended pixel from the FIFOs.
    pub fn shift(&mut self, ppu: &Ppu) -> Option<Pixel> {
        // A shift only occurs if there are pixels in the background FIFO
        let pixel = if let Some(mut bgwin) = self.bgwin.fifo.pop() {
            // Overwrite the background/window pixel data if disabled
            let lcdc = ppu.reg.lcdc.load();
            if !Lcdc::BgWinEnable.get(&lcdc) {
                bgwin.col = Color::C0;
            }

            // Now also pop the sprite FIFO
            if let Some(sprite) = self.sprite.fifo.pop() {
                // Blend the two pixels together
                Pixel::blend(bgwin, sprite)
            } else {
                // No sprite, so use the background FIFO's pixel
                bgwin
            }
        } else {
            // Nothing in the background FIFO
            return None;
        };

        // Check if this pixel needs to be discarded (as a part of implementing
        // the behaviour of SCX)
        if self.discard > 0 {
            // One fewer pixel to discard
            self.discard -= 1;
            // Discard this pixel
            None
        } else {
            // Move internal x-position
            self.xpos += 1;
            // Return this pixel
            Some(pixel)
        }
    }

    /// Checks if the position is at a window boundary.
    fn found_win(&self, ppu: &Ppu) -> bool {
        // Extract scanline info
        let lcdc = ppu.reg.lcdc.load();
        let ly = ppu.reg.ly.load();
        let wy = ppu.reg.wy.load();
        let wx = ppu.reg.wx.load();

        // The window is reached if:
        // - window is enabled
        let enabled = Lcdc::WinEnable.get(&lcdc);
        // - y-position is NOT above window
        let above = ly < wy;
        // - x-position is NOT left of window
        let left = self.xpos + 7 < wx;

        enabled && !above && !left
    }

    /// Checks if the position is within a window.
    pub fn within_win(&self) -> bool {
        self.bgwin.loc == Location::Window
    }
}

#[derive(Clone, Debug, Default)]
struct Channel {
    loc: Location,
    fetch: Fetch,
    fifo: Fifo,
}

impl Channel {
    pub fn fetch(&mut self, ppu: &mut Ppu, sprite: Option<Sprite>) {
        self.fetch.exec(&mut self.fifo, ppu, self.loc, sprite);
    }
}
