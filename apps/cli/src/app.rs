use std::io::{self, Read, Write};
use std::net::UdpSocket;
#[cfg(feature = "gbd")]
use std::sync::mpsc;
use std::time::Instant;

use anyhow::Context;
use log::{debug, trace};
use minifb::Key;
use remus::{Clock, Machine};
#[cfg(any(feature = "doctor", feature = "view"))]
use rugby::core::dmg;
use rugby::core::dmg::{Button, GameBoy, Screen};
use rugby::core::Emulator;
#[cfg(feature = "gbd")]
use rugby::gbd::{self, Debugger};
use rugby::pal::Palette;

#[cfg(feature = "doctor")]
use crate::dbg::doc::Doctor;
use crate::gui::Gui;
use crate::FREQ;

// Clock divider for more efficient synchronization.
const DIVIDER: u32 = 0x100;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Application options.
    pub cfg: Options,
    /// Debug options.
    #[cfg(feature = "debug")]
    pub dbg: Debug,
    /// Connected peripherals.
    pub dev: Devices,
    /// Emulator instance.
    pub emu: GameBoy,
    /// User-interface handle.
    pub gui: Option<Gui>,
}

/// Application options.
#[derive(Debug)]
pub struct Options {
    /// Cartridge title.
    pub title: String,
    /// Color palette.
    pub pal: Palette,
    /// Clock frequency.
    pub spd: Option<u32>,
}

/// Application debug.
#[cfg(feature = "debug")]
#[derive(Debug)]
pub struct Debug {
    /// Introspective logging.
    #[cfg(feature = "doctor")]
    pub doc: Option<Doctor>,
    /// Interactive debugger.
    #[cfg(feature = "gbd")]
    pub gbd: Option<Debugger>,
}

/// Application devices.
#[derive(Debug)]
pub struct Devices {
    /// Link cable.
    pub link: Option<UdpSocket>,
}

impl App {
    /// Runs the application.
    #[allow(clippy::too_many_lines)]
    pub fn run(self) -> crate::Result<()> {
        #[allow(unused_mut, unused_variables)]
        let Self {
            cfg,
            #[cfg(feature = "debug")]
            mut dbg,
            mut dev,
            mut emu,
            mut gui,
        } = self;

        // Construct clock for emulator sync
        let mut clk = cfg.spd.map(|freq| freq / DIVIDER).map(Clock::with_freq);

        // Initialize timer, counters
        let mut now = Instant::now();
        let mut cycle = 0;
        let mut stamp = 0;
        let mut frame = 0;

        // Install signal handlers
        #[cfg(feature = "gbd")]
        let rx = {
            let (tx, rx) = mpsc::channel();
            ctrlc::try_set_handler(move || tx.send(()).expect("failed to send signal"))
                .context("failed to install SIGINT handler")?;
            rx
        };

        // Enable doctor
        #[cfg(feature = "doctor")]
        emu.doctor(dbg.doc.is_some());

        // Prepare debugger
        #[cfg(feature = "gbd")]
        if let Some(gbd) = dbg.gbd.as_mut() {
            // Enable debugger
            gbd.enable();
            // Sync initial console state
            gbd.sync(&emu);
        }

        // Emulation loop
        loop {
            // Break when GUI is closed
            if let Some(false) = gui.as_ref().map(Gui::alive) {
                break;
            }

            // Handle signals
            #[cfg(feature = "gbd")]
            if let Ok(()) = rx.try_recv() {
                if let Some(gbd) = dbg.gbd.as_mut() {
                    gbd.enable();
                } else {
                    break;
                }
            }

            // Calculate wall-clock frequency
            if cycle % DIVIDER == 0 && now.elapsed().as_secs() > 0 {
                // Calculate stats
                let iters = cycle - stamp; // iterations since last probe
                let freq = f64::from(iters) / now.elapsed().as_secs_f64();
                let speedup = freq / f64::from(FREQ);
                let fps = 60. * speedup;
                // Log stats
                trace!(
                    "frequency: {freq:>7.4} MHz, speedup: {speedup:>5.1}%, frame rate: {fps:>6.2} FPS",
                    freq = freq / 1e6, speedup = 100. * speedup,
                );
                // Display frequency in GUI
                if let Some(gui) = gui.as_mut() {
                    gui.main.set_title(&format!("{} ({fps:.1} FPS)", cfg.title));
                }
                // Reset timer, counters
                now = Instant::now();
                stamp = cycle;
                frame = 0;
            }

            // Run debugger when enabled
            #[cfg(feature = "gbd")]
            if let Some(gbd) = dbg.gbd.as_mut() {
                // Sync with console
                gbd.sync(&emu);

                // Run debugger when enabled
                if gbd.enabled() {
                    let res = gbd.run(&mut emu, &mut clk);
                    // Quit if requested
                    if matches!(res, Err(gbd::Error::Quit)) {
                        return Ok(());
                    }
                }

                // Cycle debugger to remain synchronized with emulator
                gbd.cycle();
            }

            // Synchronize with wall-clock
            if cycle % DIVIDER == 0 {
                // Delay until clock is ready
                clk.as_mut().map(Iterator::next);
            }

            // Emulate a single cycle
            emu.cycle();

            // Redraw GUI
            if let Some(gui) = gui.as_mut() {
                // Redraw main window
                let mut res = Ok(());
                emu.redraw(|screen: &Screen| {
                    // Collect PPU screen into buffer
                    let buf = screen
                        .iter()
                        .map(|&col| cfg.pal[col as usize].into())
                        .collect::<Vec<_>>();
                    // Redraw main window
                    res = gui.main.redraw(&buf);
                    frame += 1; // update frames drawn
                });
                res.context("failed to redraw screen")?; // return early if window update failed

                // Redraw debug view
                #[cfg(feature = "view")]
                if let Some(view) = &mut gui.view {
                    // Update upon counter reset
                    if stamp == cycle {
                        // Gather debug info
                        let info = dmg::dbg::ppu(&mut emu);
                        // Extract PPU state
                        let tdat = info.tdat.map(|col| cfg.pal[col as usize].into());
                        let map1 = info.map1.map(|col| cfg.pal[col as usize].into());
                        let map2 = info.map2.map(|col| cfg.pal[col as usize].into());
                        // Display PPU state
                        view.tdat
                            .redraw(&tdat)
                            .context("failed to redraw tile data")?;
                        view.map1
                            .redraw(&map1)
                            .context("failed to redraw tile map 1")?;
                        view.map2
                            .redraw(&map2)
                            .context("failed to redraw tile map 2")?;
                    }
                }
            }

            // Log doctor entries
            #[cfg(feature = "doctor")]
            if let Some(out) = &mut dbg.doc {
                // Gather debug info
                let info = dmg::dbg::doc(&mut emu);
                // Format, writing if non-empty
                let note = format!("{info}");
                if !note.is_empty() {
                    writeln!(out.log, "{note}").context("failed to write doctor log")?;
                }
            }

            // Send joypad input
            if let Some(gui) = gui.as_mut() {
                // NOTE: Input is sampled every 64 cycles.
                if cycle % 0x40 == 0 {
                    #[rustfmt::skip]
                    let keys = gui
                        .main
                        .get_keys()
                        .into_iter()
                        .filter_map(|key| match key {
                            Key::Z     => Some(Button::A),
                            Key::X     => Some(Button::B),
                            Key::Space => Some(Button::Select),
                            Key::Enter => Some(Button::Start),
                            Key::Right => Some(Button::Right),
                            Key::Left  => Some(Button::Left),
                            Key::Up    => Some(Button::Up),
                            Key::Down  => Some(Button::Down),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    emu.send(&keys);
                }
            }

            // Stream serial data
            if let Some(link) = dev.link.as_mut() {
                // NOTE: Data is streamed at 1 Hz.
                if cycle % FREQ == 0 {
                    // emu -> link
                    'tx: {
                        // Download from emulator
                        let mut buf = Vec::new();
                        let read = emu
                            .serial_mut()
                            .read_to_end(&mut buf)
                            .context("failed to download serial data from emulator")?;
                        if read == 0 {
                            break 'tx;
                        }
                        // Transmit data to link
                        let sent = link
                            .send(&buf)
                            .context("failed to transmit serial data to network")?;
                        debug!("transmitted {sent} bytes ({read} downloaded)");
                        trace!("serial tx: {buf:?}");
                    }
                    // link -> emu
                    'rx: {
                        // Receive data from link
                        let mut buf = [0; 0x10]; // use fixed-size buffer
                        let recvd = match link.recv(&mut buf) {
                            // Explicitly ignore would block error
                            Err(err) if err.kind() == io::ErrorKind::WouldBlock => Ok(0),
                            res => res,
                        }
                        .context("failed to receive serial data from network")?;
                        let buf = &buf[..recvd]; // truncate to valid data
                        if recvd == 0 {
                            break 'rx;
                        }
                        // Upload to emulator
                        let wrote = emu
                            .serial_mut()
                            .write(buf)
                            .context("failed to upload serial data to emulator")?;
                        debug!("received {recvd} bytes ({wrote} uploaded)");
                        trace!("serial rx: {buf:?}");
                    }
                }
            }

            // Clock another cycle
            cycle += 1;
        }

        Ok(())
    }
}
