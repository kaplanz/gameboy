use log::trace;
use rugby_arch::reg::Register;
use rugby_arch::Byte;

use super::fetch::{Fetch, Layer, Step};
use super::fifo::Fifo;
use super::pixel::{Color, Pixel};
use super::sprite::Sprite;
use super::{Lcdc, Ppu};

/// Pixel pipeline.
#[derive(Clone, Debug, Default)]
pub struct Pipeline {
    /// Warm-up completed.
    pub ready: bool,
    /// LCD X-coordinate.
    pub lx: Byte,
    /// Background/Window channel.
    pub bgwin: Channel,
    /// Sprite channel.
    pub sprite: Channel,
}

impl Pipeline {
    /// Performs a fetch for the next pixels to the appropriate FIFO.
    pub fn fetch(&mut self, ppu: &mut Ppu, objs: &[Sprite]) {
        // Check if we're at an object
        if let Some(obj) = objs.iter().find(|obj| obj.xpos == self.lx + 8) {
            trace!("found sprite: {obj:?}");
            // Ensure the sprite is not overridden
            if self.sprite.fifo.is_empty() {
                // Fetch the sprite
                self.sprite.exec(ppu, Some(obj.clone()));
                // Stall the background fetcher
                return;
            }
            trace!("ignored; sprite already being drawn");
        }

        // Execute the background fetcher
        self.bgwin.exec(ppu, None);

        // Restart background fetcher when:
        //
        // 1. The first "warm-up" fetch completes
        let done_warmup = !self.ready && matches!(self.bgwin.fetch.step, Step::Push(_, _));
        if done_warmup {
            trace!("pipeline warmup complete");
            // Configure channels (only run once)
            self.bgwin.loc = Layer::Background;
            self.sprite.loc = Layer::Sprite;
            // We're now ready for real fetches
            self.ready = true;
        }
        // 2. The window border has been reached
        let window_reached = {
            // 1. The window is enabled
            let win_enabled = Lcdc::WinEnable.get(&ppu.reg.lcdc.load());
            // 2. Fetcher is still at the background
            let fetch_at_bg = self.bgwin.loc == Layer::Background;
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
            self.bgwin.loc = Layer::Window;
        }
        //
        // If either condition is met...
        if done_warmup || window_reached {
            // ... reset background fetcher, and...
            std::mem::take(&mut self.bgwin.fetch);
            // ... clear background FIFO.
            self.bgwin.fifo.clear();
        }
    }

    /// Shift out a blended pixel from the FIFOs.
    pub fn shift(&mut self, ppu: &Ppu) -> Option<Pixel> {
        // Check the sprite FIFO isn't in progress
        if !matches!(self.sprite.fetch.step, Step::ReadTile) {
            return None;
        }

        // Pop from the background/window FIFO
        let Some(mut bgwin) = self.bgwin.fifo.pop() else {
            return None; // FIFO is empty
        };

        // Overwrite if the background/window disabled
        let lcdc = ppu.reg.lcdc.load();
        if !Lcdc::BgWinEnable.get(&lcdc) {
            bgwin.col = Color::C0;
        }

        // Pop from the sprite FIFO
        let pixel = if let Some(sprite) = self.sprite.fifo.pop() {
            Pixel::blend(bgwin, sprite) // blend the pixels together
        } else {
            bgwin // no sprite; use background/window pixel
        };

        Some(pixel)
    }
}

/// Pixel FIFO channel.
#[derive(Clone, Debug, Default)]
pub struct Channel {
    /// Fetch location.
    pub loc: Layer,
    /// Pixel fetcher.
    pub fetch: Fetch,
    /// Pixel FIFO.
    pub fifo: Fifo,
}

impl Channel {
    /// Executes a step of the pipeline channel.
    pub fn exec(&mut self, ppu: &mut Ppu, sprite: Option<Sprite>) {
        self.fetch.exec(&mut self.fifo, ppu, self.loc, sprite);
    }
}
