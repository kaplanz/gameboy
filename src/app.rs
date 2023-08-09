use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::time::Duration;

use gameboy::core::Emulator;
use gameboy::dmg::{Button, GameBoy, Screen, SCREEN};
use log::debug;
use minifb::{Key, Scale, Window, WindowOptions};
use parking_lot::Mutex;
use remus::{Clock, Machine};

use crate::gbd::Debugger;
use crate::pal::Palette;
use crate::{gbd, Speed, FREQ};

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
    pub debug: Option<Debug>,
    pub doctor: Option<Doctor>,
    pub gbd: Option<Debugger>,
}

impl App {
    #[allow(clippy::too_many_lines)]
    pub fn run(self) -> crate::Result<()> {
        let Self {
            opts,
            mut emu,
            mut win,
            mut debug,
            mut doctor,
            gbd,
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
        if doctor.is_some() {
            emu.doctor.enable();
        }

        // Prepare debugger when used
        let mut gbd = gbd.map(Mutex::new).map(Arc::new);
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
            })?;
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
                                match gbd.fetch() {
                                    // It worked; use it
                                    cmd @ Some(_) => cmd,
                                    // Couldn't fetch; prompt user
                                    None => match gbd.prompt() {
                                        // Program input; fetch next iteration
                                        Ok(_) => continue 'gbd,
                                        // No input; repeat previous command
                                        Err(gbd::Error::NoInput) => gbd.prev().cloned(),
                                        // Prompt error; handle upstream
                                        err @ Err(_) => {
                                            break 'res err;
                                        }
                                    },
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
                }

                // Cycle debugger to remain synchronized with emulator
                gbd.cycle();
            }

            // Synchronize with wall-clock
            // TODO: Pause when in GBD
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
            winres?; // return early if window update failed

            // Update the debug screens every second
            if let Some(debug) = &mut debug {
                if cycle == 0 {
                    // Probe for debug info
                    let info = emu.debug();

                    // Extract PPU state
                    let tdat = info.ppu.tdat.map(|col| opts.pal[col as usize].into());
                    let map1 = info.ppu.map1.map(|col| opts.pal[col as usize].into());
                    let map2 = info.ppu.map2.map(|col| opts.pal[col as usize].into());
                    // Display PPU state
                    debug.tdat.update_with_buffer(&tdat, 16 * 8, 24 * 8)?;
                    debug.map1.update_with_buffer(&map1, 32 * 8, 32 * 8)?;
                    debug.map2.update_with_buffer(&map2, 32 * 8, 32 * 8)?;
                }
            }

            // Log doctor entries
            if let Some(doctor) = &mut doctor {
                if let Some(entries) = emu.doctor.checkup() {
                    if !entries.is_empty() {
                        writeln!(doctor.log, "{entries}")?;
                    }
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

#[derive(Debug)]
pub struct Debug {
    tdat: Window,
    map1: Window,
    map2: Window,
}

impl Debug {
    pub fn new(opts: WindowOptions) -> Self {
        Self {
            tdat: Window::new("Tile Data", 16 * 8, 24 * 8, opts).unwrap(),
            map1: Window::new(
                "Tile Map 1",
                32 * 8,
                32 * 8,
                WindowOptions {
                    scale: Scale::X1,
                    ..opts
                },
            )
            .unwrap(),
            map2: Window::new(
                "Tile Map 2",
                32 * 8,
                32 * 8,
                WindowOptions {
                    scale: Scale::X1,
                    ..opts
                },
            )
            .unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct Doctor {
    log: BufWriter<File>,
}

impl Doctor {
    pub fn new(log: File) -> Self {
        Self {
            log: BufWriter::new(log),
        }
    }
}
