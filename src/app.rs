#[cfg(feature = "doctor")]
use std::io::Write;
#[cfg(feature = "gbd")]
use std::sync::Arc;
#[cfg(feature = "gbd")]
use std::time::Duration;

use eyre::Context;
use gameboy::core::Emulator;
#[allow(unused_imports)]
use gameboy::dmg;
use gameboy::dmg::{Button, GameBoy, Screen};
use log::debug;
use minifb::Key;
#[cfg(feature = "gbd")]
use parking_lot::Mutex;
use remus::{Clock, Machine};

#[cfg(feature = "doctor")]
use crate::doc::Doctor;
#[cfg(feature = "gbd")]
use crate::gbd;
#[cfg(feature = "gbd")]
use crate::gbd::Debugger;
use crate::gui::Gui;
use crate::pal::Palette;
use crate::{Speed, FREQ};

// Clock divider for more efficient synchronization.
const DIVIDER: u32 = 0x100;

#[derive(Debug)]
pub struct App {
    pub title: String,
    pub cfg: Settings,
    pub emu: GameBoy,
    pub gui: Option<Gui>,
    #[cfg(feature = "debug")]
    pub dbg: Debug,
}

#[derive(Debug)]
pub struct Settings {
    pub pal: Palette,
    pub speed: Speed,
}

#[cfg(feature = "debug")]
#[derive(Debug)]
pub struct Debug {
    #[cfg(feature = "doctor")]
    pub doc: Option<Doctor>,
    #[cfg(feature = "gbd")]
    pub gbd: Option<Debugger>,
}

impl App {
    #[allow(clippy::too_many_lines)]
    pub fn run(self) -> crate::Result<()> {
        #[allow(unused_mut, unused_variables)]
        let Self {
            title,
            cfg,
            mut emu,
            mut gui,
            #[cfg(feature = "debug")]
            mut dbg,
        } = self;

        // Create 4 MiHz clock to sync emulator
        #[rustfmt::skip]
        let freq = match cfg.speed {
            Speed::Half         => Some(FREQ / 2),
            Speed::Full         => Some(FREQ),
            Speed::Double       => Some(2 * FREQ),
            Speed::Triple       => Some(3 * FREQ),
            Speed::Custom(freq) => Some(freq),
            Speed::Max          => None,
        };
        let mut clk = freq.map(|freq| freq / DIVIDER).map(Clock::with_freq);

        // Initialize timer, counters
        let mut now = std::time::Instant::now();
        let mut cycle = 0;
        let mut stamp = 0;
        let mut frame = 0;

        // Enable doctor when used
        #[cfg(feature = "doctor")]
        emu.set_doc(dbg.doc.is_some());

        // Prepare debugger when used
        #[cfg(feature = "gbd")]
        let mut gbd = dbg.gbd.map(Mutex::new).map(Arc::new);
        #[cfg(feature = "gbd")]
        if let Some(gbd) = gbd.as_ref().map(Arc::clone) {
            {
                // Unlock the debugger
                let mut gbd = gbd.lock();
                // Enable debugger
                gbd.enable();
                // Sync initial console state
                gbd.sync(&emu);
            }
            // Install SIGINT handler
            ctrlc::try_set_handler(move || {
                // Attempt to acquire the debugger (with timeout)
                if let Some(mut gbd) = gbd.try_lock_for(Duration::from_millis(10)) {
                    // Pause the console and present the user with the debugger
                    // prompt.
                    gbd.enable();
                } else {
                    // If unable to pause (likely due to prompt already
                    // present), exit the program.
                    std::process::exit(0);
                }
            })
            .context("failed to install SIGINT handler")?;
        }

        // Emulation loop
        loop {
            // Break when GUI is closed
            if let Some(false) = gui.as_ref().map(Gui::alive) {
                break;
            }

            // Calculate wall-clock frequency
            if cycle % DIVIDER == 0 && now.elapsed().as_secs() > 0 {
                // Calculate stats
                let iters = cycle - stamp; // iterations since last probe
                let freq = f64::from(iters) / now.elapsed().as_secs_f64();
                let speedup = freq / f64::from(FREQ);
                let fps = 60. * speedup;
                // Log stats
                debug!(
                    "frequency: {freq:>7.4} MHz, speedup: {speedup:>5.1}%, frame rate: {fps:>6.2} Hz",
                    freq = freq / 1e6, speedup = 100. * speedup,
                );
                // Display frequency in GUI
                if let Some(gui) = gui.as_mut() {
                    gui.main.set_title(&format!("{title} ({fps:.1} Hz)"));
                }
                // Reset timer, counters
                now = std::time::Instant::now();
                stamp = cycle;
                frame = 0;
            }

            // Run debugger when enabled
            #[cfg(feature = "gbd")]
            if let Some(mut gbd) = gbd.as_mut().map(|gbd| gbd.lock()) {
                // Sync with console
                gbd.sync(&emu);

                // Perform debugger actions
                if gbd.enabled() {
                    // Provide information to user before prompting for command
                    gbd.inform(&emu);
                    // Prompt and execute commands until emulation resumed
                    gbd.pause();
                    'gbd: while gbd.paused() {
                        let res = 'res: {
                            // Attempt to fetch command
                            let cmd = {
                                // Attempt to fetch the next command
                                if let cmd @ Some(_) = gbd.fetch() {
                                    // It worked; use it
                                    cmd
                                } else {
                                    // Couldn't fetch; get program from user
                                    match {
                                        // Pause clock while awaiting user input
                                        clk.as_mut().map(Clock::pause);
                                        // Present the prompt
                                        gbd.prompt()
                                    } {
                                        // Program input; fetch next iteration
                                        Ok(_) => continue 'gbd,
                                        // No input; repeat previous command
                                        Err(gbd::Error::NoInput) => gbd.prev().cloned(),
                                        // Prompt error; handle upstream
                                        err @ Err(_) => {
                                            break 'res err;
                                        }
                                    }
                                }
                            };
                            // Extract fetched command
                            let Some(cmd) = cmd else {
                                // Command still not found; this case should
                                // only occur when no input has been provided,
                                // as otherwise the previously executed command
                                // should be repeated.
                                continue 'gbd;
                            };
                            // Execute fetched command
                            gbd.exec(&mut emu, cmd)
                        };
                        match res {
                            Ok(_) => (),
                            Err(gbd::Error::Quit) => return Ok(()),
                            Err(err) => tell::error!("{err}"),
                        }
                    }

                    // Unconditionally resume the clock
                    clk.as_mut().map(Clock::resume);
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
                    if frame == 0 {
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
                let keys = gui.main.get_keys().into_iter().filter_map(|key| match key {
                    Key::Z     => Some(Button::A),
                    Key::X     => Some(Button::B),
                    Key::Space => Some(Button::Select),
                    Key::Enter => Some(Button::Start),
                    Key::Right => Some(Button::Right),
                    Key::Left  => Some(Button::Left),
                    Key::Up    => Some(Button::Up),
                    Key::Down  => Some(Button::Down),
                    _ => None
                }).collect::<Vec<_>>();
                    emu.send(&keys);
                }
            }

            // Clock another cycle
            cycle += 1;
        }

        Ok(())
    }
}
