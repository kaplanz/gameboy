//! Application structures.

#[cfg(feature = "doc")]
use std::io::Write;
#[cfg(feature = "gbd")]
use std::sync::mpsc;
use std::time::Instant;

use anyhow::Context as _;
#[allow(unused)]
use log::{debug, error, trace};
use remus::{Clock, Machine};
use rugby::app::joypad::Joypad;
use rugby::app::serial::Serial;
use rugby::app::video::Video;
#[cfg(any(feature = "doc", feature = "win"))]
use rugby::core::dmg;
use rugby::core::dmg::{Cartridge, GameBoy};
use rugby::emu::cart::Support as _;
use rugby::emu::joypad::{Joypad as _, Support as _};
use rugby::emu::serial::{Serial as _, Support as _};
use rugby::emu::video::{Support as _, Video as _};
#[cfg(feature = "gbd")]
use rugby::gbd;
#[cfg(feature = "gbd")]
use rugby::gbd::Debugger;

use self::ctx::Counter;
#[cfg(feature = "win")]
use self::gui::dbg::Region;
#[cfg(feature = "doc")]
use crate::dbg::doc::Doctor;
use crate::def::NAME;

mod ctx;

pub mod gui;

pub use self::ctx::Context;
pub use self::gui::{Frontend, Graphics};

/// Clock divider.
///
/// As an optimization for more efficient synchronization, divide the target
/// frequency by this number, but clock this number as many cycles each time.
const DIVIDER: u32 = 0x100;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Configuration data.
    pub cfg: Options,
    /// Runtime context.
    pub ctx: Option<Context>,
    /// Debug features.
    #[cfg(feature = "debug")]
    pub dbg: Debug,
    /// Emulator instance.
    pub emu: GameBoy,
    /// Graphical frontend.
    pub gui: Frontend,
}

/// Configuration data.
#[derive(Debug)]
pub struct Options {
    /// Clock frequency.
    pub spd: Option<u32>,
}

/// Debug features.
#[cfg(feature = "debug")]
#[derive(Debug)]
pub struct Debug {
    /// Introspective logging.
    #[cfg(feature = "doc")]
    pub doc: Option<Doctor>,
    /// Interactive debugger.
    #[cfg(feature = "gbd")]
    pub gbd: Option<Debugger>,
}

impl App {
    /// Runs the application.
    #[allow(clippy::too_many_lines)]
    pub fn run(mut self) -> crate::Result<()> {
        // Construct clock for emulator sync
        let mut clk = self
            .cfg
            .spd
            .map(|freq| freq / DIVIDER)
            .map(Clock::with_freq);

        // Initialize timer, counters
        let mut count = Counter::new();
        let mut timer = Instant::now();

        // Install signal handlers
        #[cfg(feature = "gbd")]
        let sigint = {
            let (tx, rx) = mpsc::channel();
            ctrlc::set_handler(move || {
                trace!("triggered interrupt");
                if let Err(err) = tx.send(()) {
                    error!("could not send interrupt: {err}");
                }
            })
            .context("failed to set interrupt handler")?;
            rx
        };

        // Enable doctor
        #[cfg(feature = "doc")]
        self.emu.doctor(self.dbg.doc.is_some());

        // Prepare debugger
        #[cfg(feature = "gbd")]
        if let Some(gbd) = self.dbg.gbd.as_mut() {
            // Enable debugger
            gbd.enable();
            // Sync initial console state
            gbd.sync(&self.emu);
        }

        // Emulation loop
        loop {
            // Break when GUI is closed
            if !self.gui.win.as_ref().map_or(true, Graphics::alive) {
                break;
            }

            // Handle signals
            #[cfg(feature = "gbd")]
            if sigint.try_recv().is_ok() {
                debug!("received interrupt");
                if let Some(gbd) = self.dbg.gbd.as_mut() {
                    gbd.enable();
                } else {
                    break;
                }
            }

            // Run debugger when enabled
            #[cfg(feature = "gbd")]
            if let Some(gbd) = self.dbg.gbd.as_mut() {
                // Sync with console
                gbd.sync(&self.emu);

                // Run debugger when enabled
                if gbd.enabled() {
                    let res = gbd.run(&mut self.emu, &mut clk);
                    // Quit if requested
                    if matches!(res, Err(gbd::Error::Quit)) {
                        return Ok(());
                    }
                }

                // Cycle debugger to remain synchronized with emulator
                gbd.cycle();
            }

            // Synchronize with wall-clock
            if count.cycle() % DIVIDER == 0 {
                // Delay until clock is ready
                clk.as_mut().map(Iterator::next);
            }

            // Emulate a single cycle
            self.emu.cycle();

            // Send joypad input
            if count.cycle() % 40 == 0 {
                // Fetch keys
                let keys = self.gui.input();
                // Update emulator
                self.emu.joypad_mut().recv(keys);
            }

            // Sync serial data
            let serial = self.emu.serial_mut();
            if count.cycle() % 0x10000 == 0 {
                // Receive data from emulator
                let rx = serial.rx();
                self.gui.recv(rx).context("failed to recv serial data")?;
                // Forward data to emulator
                let tx = serial.tx();
                self.gui.send(tx).context("failed to send serial data")?;
            }

            // Draw next frame
            let video = self.emu.video();
            if video.ready() {
                // Borrow frame
                let frame = video.frame();
                // Redraw screen
                self.gui.draw(frame);
            }

            // Redraw debug windows
            #[cfg(feature = "win")]
            if count.delta() == 0 {
                if let Some(dbg) = self.gui.win.as_mut().map(|gui| &mut gui.dbg) {
                    // Gather debug info
                    let info = dmg::dbg::ppu(&mut self.emu);
                    // Extract PPU state
                    let tile = info.tile.map(|col| self.gui.cfg.pal[col as usize].into());
                    let map1 = info.map1.map(|col| self.gui.cfg.pal[col as usize].into());
                    let map2 = info.map2.map(|col| self.gui.cfg.pal[col as usize].into());
                    // Display PPU state
                    dbg.get_mut(Region::Tile)
                        .map(|win| win.redraw(&tile))
                        .transpose()
                        .context("failed to redraw tile data")?;
                    dbg.get_mut(Region::Map1)
                        .map(|win| win.redraw(&map1))
                        .transpose()
                        .context("failed to redraw tile map 1")?;
                    dbg.get_mut(Region::Map2)
                        .map(|win| win.redraw(&map2))
                        .transpose()
                        .context("failed to redraw tile map 2")?;
                }
            }

            // Log doctor entries
            #[cfg(feature = "doc")]
            if let Some(out) = &mut self.dbg.doc {
                // Gather debug info
                let info = dmg::dbg::doc(&mut self.emu);
                // Format, writing if non-empty
                let note = format!("{info}");
                if !note.is_empty() {
                    writeln!(out, "{note}").context("failed to write doctor log")?;
                }
            }

            // Reset timing stats
            let elapsed = timer.elapsed();
            if elapsed.as_secs() > 0 {
                // Calculate stats
                let stats = count.stats(elapsed);
                debug!("{stats}");
                // Update GUI title
                if let Some(gui) = self.gui.win.as_mut() {
                    gui.lcd.set_title(&format!(
                        "{title} ({frame:.1} FPS)",
                        title = self.emu.cart().map_or(NAME, Cartridge::title),
                        frame = stats.rate()
                    ));
                }
                // Flush count, reset timer
                count.flush();
                timer = Instant::now();
            } else {
                // Clock another cycle
                count.tick();
            }
        }

        Ok(())
    }
}
