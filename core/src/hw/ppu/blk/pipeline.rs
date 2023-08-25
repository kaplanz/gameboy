use remus::Cell;

use super::fetch::{Fetch, Location, Stage};
use super::fifo::Fifo;
use super::pixel::{Color, Pixel};
use super::sprite::Sprite;
use super::{Lcdc, Ppu};

#[derive(Debug, Default)]
pub struct Pipeline {
    ready: bool,
    discard: u8,
    xpos: u8,
    bgwin: Channel,
    sprite: Channel,
}

impl Pipeline {
    #[must_use]
    pub fn xpos(&self) -> u8 {
        self.xpos
    }

    pub fn set_discard(&mut self, discard: u8) {
        self.discard = discard;
    }

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
        let reached_window = !self.was_at_win() && self.is_at_win(ppu);
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

    pub fn shift(&mut self, ppu: &Ppu) -> Option<Pixel> {
        // A shift only occurs if there are pixels in the background FIFO
        let pixel = if let Some(mut bgwin) = self.bgwin.fifo.pop() {
            // Overwrite the background/window pixel data if disabled
            let lcdc = ppu.file.lcdc.load();
            if !Lcdc::BgWinEnable.get(lcdc) {
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

    fn is_at_win(&self, ppu: &Ppu) -> bool {
        // Extract scanline info
        let lcdc = ppu.file.lcdc.load();
        let ly = ppu.file.ly.load();
        let wy = ppu.file.wy.load();
        let wx = ppu.file.wx.load();

        // The window is reached if:
        // - The window is enabled
        let enabled = Lcdc::WinEnable.get(lcdc);
        // - The y-position is NOT above the window
        let above = ly < wy;
        // - The x-position is NOT left of the window
        let left = self.xpos + 7 < wx;

        enabled && !above && !left
    }

    pub fn was_at_win(&self) -> bool {
        self.bgwin.loc == Location::Window
    }
}

#[derive(Debug, Default)]
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
