//! Application structures.

#[cfg(feature = "doc")]
use std::io::Write;
#[cfg(feature = "gbd")]
use std::sync::mpsc;
use std::time::Instant;

use anyhow::Context as _;
use log::debug;
use rugby::arch::{Block, Clock};
#[cfg(any(feature = "doc", feature = "win"))]
use rugby::core::dmg;
#[cfg(feature = "doc")]
use rugby::core::dmg::cpu::Stage;
use rugby::core::dmg::{Cartridge, GameBoy};
use rugby::prelude::*;
#[cfg(feature = "gbd")]
use rugby_gbd::Debugger;

use self::ctx::Counter;
#[cfg(feature = "win")]
use self::gui::dbg::Region;
#[cfg(feature = "doc")]
use crate::dbg::doc::Doctor;
use crate::NAME;

mod ctx;

pub mod gui;

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
    /// Emulator instance.
    pub emu: GameBoy,
    /// Graphical frontend.
    pub gui: Frontend,
    /// Debug features.
    #[cfg(feature = "debug")]
    pub dbg: Debug,
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
    /// Graphical VRAM rendering.
    #[cfg(feature = "win")]
    pub win: bool,
}

impl App {
    /// Runs the application.
    #[allow(clippy::too_many_lines)]
    pub fn run(&mut self) -> crate::Result<()> {
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
                log::trace!("triggered interrupt");
                if let Err(err) = tx.send(()) {
                    log::error!("could not send interrupt: {err}");
                }
            })
            .context("failed to set interrupt handler")?;
            rx
        };

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
                if gbd.ready() {
                    let res = gbd.run(&mut self.emu, &mut clk);
                    // Quit if requested
                    if matches!(res, Err(rugby_gbd::Error::Quit)) {
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
                self.emu.inside_mut().joypad().recv(keys);
            }

            // Sync serial data
            let serial = self.emu.inside_mut().serial();
            if count.cycle() % 0x10000 == 0 {
                // Receive data from emulator
                let rx = serial.rx();
                self.gui.recv(rx).context("failed to recv serial data")?;
                // Forward data to emulator
                let tx = serial.tx();
                self.gui.send(tx).context("failed to send serial data")?;
            }

            // Draw next frame
            let video = self.emu.inside().video();
            if video.vsync() {
                // Borrow frame
                let frame = video.frame();
                // Redraw screen
                self.gui.draw(frame);

                // Redraw debug windows
                #[cfg(feature = "win")]
                if self.dbg.win {
                    if let Some(dbg) = self.gui.win.as_mut().map(|gui| &mut gui.dbg) {
                        // Gather debug info
                        let info = dmg::dbg::ppu(&mut self.emu);
                        // Extract PPU state
                        let recolor = |col: dmg::ppu::Color| self.gui.cfg.pal[col as usize].into();
                        let tdat = info.tdat.into_iter().map(recolor).collect::<Box<_>>();
                        let map1 = info.map1.into_iter().map(recolor).collect::<Box<_>>();
                        let map2 = info.map2.into_iter().map(recolor).collect::<Box<_>>();
                        // Display PPU state
                        dbg.get_mut(Region::Tile)
                            .map(|win| win.redraw(&tdat))
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
            }

            // Log doctor entries
            #[cfg(feature = "doc")]
            if let Some(out) = &mut self.dbg.doc {
                if matches!(self.emu.inside().proc().stage(), Stage::Done) && count.delta % 4 == 0 {
                    // Gather debug info
                    let info = dmg::dbg::cpu(&mut self.emu);
                    // Format, writing if non-empty
                    if !info.doc.is_empty() {
                        writeln!(out, "{}", info.doc).context("failed to write doctor entry")?;
                    }
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
