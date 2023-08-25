#[cfg(feature = "doc")]
use std::io::Write;
#[cfg(feature = "gbd")]
use std::sync::Arc;
#[cfg(feature = "gbd")]
use std::time::Duration;

use eyre::Context;
use gameboy::core::Emulator;
use gameboy::dmg::{self, Button, GameBoy, Screen, SCREEN};
use log::debug;
use minifb::{Key, Window};
#[cfg(feature = "gbd")]
use parking_lot::Mutex;
use remus::{Clock, Machine};

#[cfg(feature = "doc")]
use crate::doc::Doctor;
#[cfg(feature = "gbd")]
use crate::gbd;
#[cfg(feature = "gbd")]
use crate::gbd::Debugger;
use crate::pal::Palette;
#[cfg(feature = "view")]
use crate::view::View;
use crate::{Speed, FREQ};

#[derive(Debug)]
pub struct Opts {
    pub title: String,
    pub pal: Palette,
    pub speed: Speed,
}

#[derive(Debug)]
pub struct App {
    pub opts: Opts,
    pub emu: GameBoy,
    pub win: Window,
    #[cfg(feature = "doc")]
    pub doc: Option<Doctor>,
    #[cfg(feature = "gbd")]
    pub gbd: Option<Debugger>,
    #[cfg(feature = "view")]
    pub view: Option<View>,
}

impl App {
    #[allow(clippy::too_many_lines)]
    pub fn run(self) -> crate::Result<()> {
        let Self {
            opts,
            mut emu,
            mut win,
            #[cfg(feature = "doc")]
            mut doc,
            #[cfg(feature = "gbd")]
            gbd,
            #[cfg(feature = "view")]
            mut view,
        } = self;
        let title = opts.title;

        // Create 4 MiHz clock to sync emulator
        let divider = 0x100; // user a clock divider to sync
        let freq = match opts.speed {
            Speed::Half => FREQ / 2,
            Speed::Full => FREQ,
            Speed::Double => 2 * FREQ,
            Speed::Triple => 3 * FREQ,
            Speed::Max => divider, // special case
            Speed::Custom(freq) => freq,
        };
        let mut clk = Clock::with_freq(freq / divider);

        // Initialize timer, counters
        let mut now = std::time::Instant::now();
        let mut cycle = 0;
        let mut fps = 0;

        // Enable doctor when used
        #[cfg(feature = "doc")]
        emu.set_doc(doc.is_some());

        // Prepare debugger when used
        #[cfg(feature = "gbd")]
        let mut gbd = gbd.map(Mutex::new).map(Arc::new);
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
        while win.is_open() {
            // Calculate wall-clock frequency
            if now.elapsed().as_secs() > 0 {
                // Print cycle stats
                debug!(
                    "Frequency: {freq:0.4} MHz ({speedup:.1}%), FPS: {fps} Hz",
                    freq = f64::from(cycle) / 1e6,
                    speedup = 100. * f64::from(cycle) / f64::from(FREQ)
                );
                // Update the title to display the frequency
                win.set_title(&format!("{title} ({fps} Hz)"));
                // Reset timer, counters
                now = std::time::Instant::now();
                cycle = 0;
                fps = 0;
            }

            // Optionally run the debugger
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
                                        clk.pause();
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
                    clk.resume();
                }

                // Cycle debugger to remain synchronized with emulator
                gbd.cycle();
            }

            // Synchronize with wall-clock
            if cycle % divider == 0 && opts.speed != Speed::Max {
                // Delay until clock is ready
                clk.next();
            }

            // Perform a single cycle
            emu.cycle();

            // Redraw the screen (if needed)
            let mut winres = Ok(());
            emu.redraw(|screen: &Screen| {
                let buf = screen
                    .iter()
                    .map(|&col| opts.pal[col as usize].into())
                    .collect::<Vec<_>>();
                winres = win.update_with_buffer(&buf, SCREEN.width, SCREEN.height);
                fps += 1; // update frames drawn
            });
            winres.context("failed to redraw screen")?; // return early if window update failed

            // Update the debug view every second
            #[cfg(feature = "view")]
            if let Some(view) = &mut view {
                if cycle == 0 {
                    // Gather debug info
                    let info = dmg::dbg::ppu(&mut emu);
                    // Extract PPU state
                    let tdat = info.tdat.map(|col| opts.pal[col as usize].into());
                    let map1 = info.map1.map(|col| opts.pal[col as usize].into());
                    let map2 = info.map2.map(|col| opts.pal[col as usize].into());
                    // Display PPU state
                    view.tdat
                        .update_with_buffer(&tdat, 16 * 8, 24 * 8)
                        .context("failed to redraw tile data")?;
                    view.map1
                        .update_with_buffer(&map1, 32 * 8, 32 * 8)
                        .context("failed to redraw tile map 1")?;
                    view.map2
                        .update_with_buffer(&map2, 32 * 8, 32 * 8)
                        .context("failed to redraw tile map 2")?;
                }
            }

            // Log doctor entries
            #[cfg(feature = "doc")]
            if let Some(out) = &mut doc {
                // Gather debug info
                let info = dmg::dbg::doc(&mut emu);
                // Format, writing if non-empty
                let note = format!("{info}");
                if !note.is_empty() {
                    writeln!(out.log, "{note}").context("failed to write doctor log")?;
                }
            }

            // Send joypad input (sampled every 64 cycles)
            if cycle % 0x40 == 0 {
                #[rustfmt::skip]
                let keys = win.get_keys().into_iter().filter_map(|key| match key {
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

            // Clock another cycle
            cycle += 1;
        }

        Ok(())
    }
}
