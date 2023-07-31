use std::fs::File;
use std::io::{BufWriter, Write};

use gameboy::core::Emulator;
use gameboy::dmg::{Button, GameBoy, Screen, SCREEN};
use log::debug;
use minifb::{Key, Scale, Window, WindowOptions};
use remus::{Clock, Machine};

use crate::gbd::Debugger;
use crate::pal::Palette;
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
            mut gbd,
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
        let mut cycles = 0;
        let mut fps = 0;

        // Enable doctor when used
        if doctor.is_some() {
            emu.doctor.enable();
        }

        // Prepare debugger when used
        if let Some(gbd) = &mut gbd {
            // Enable debugger
            gbd.enable();
            // Sync initial console state
            gbd.sync(&emu);
        }

        // Emulation loop
        while win.is_open() {
            // Calculate wall-clock frequency
            if now.elapsed().as_secs() > 0 {
                // Print cycle stats
                debug!(
                    "Frequency: {freq:0.4} MHz ({speedup:.1}%), FPS: {fps} Hz",
                    freq = f64::from(cycles) / 1e6,
                    speedup = 100. * f64::from(cycles) / f64::from(FREQ)
                );
                // Update the title to display the frequency
                win.set_title(&format!("{title} ({fps} Hz)"));
                // Reset timer, counters
                now = std::time::Instant::now();
                cycles = 0;
                fps = 0;
            }

            // Optionally run the debugger
            if let Some(gbd) = &mut gbd {
                // Sync with console
                gbd.sync(&emu);
                // Perform a debugger cycle
                gbd.cycle();

                // Perform debugger actions
                if gbd.enabled() {
                    // Fetch next debugger command
                    let cmd = loop {
                        match crate::Result::from(gbd.prompt()) {
                            Ok(cmd) => break cmd,
                            Err(err) => {
                                eprintln!("{err}");
                                continue;
                            }
                        };
                    };
                    // Return to prompt when no command provided
                    let Some(cmd) = cmd else {
                        continue;
                    };
                    // Perform the command
                    let res = gbd.act(&mut emu, cmd);
                    match crate::Result::from(res) {
                        Ok(_) => (),
                        Err(err) => eprintln!("{err}"),
                    }
                }

                // Return to prompt when not playing
                if gbd.paused() {
                    continue;
                }
            }

            // Synchronize with wall-clock
            // TODO: Pause when in GBD
            if cycles % divider == 0 && opts.speed != Speed::Max {
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
                if cycles == 0 {
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
            if cycles % 0x40 == 0 {
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
            cycles += 1;
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
