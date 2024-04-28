//! Game Boy Debugger (GBD).

#![allow(clippy::result_large_err)]

use std::fmt::{Debug, Display, Write};

use indexmap::IndexMap;
use log::debug;
use remus::{Block, Clock, Location, Machine};
use thiserror::Error;

use self::lang::Program;
use self::prompt::Prompt;
use crate::core::dmg::cpu::{self, reg};
use crate::core::dmg::{ppu, GameBoy};
use crate::emu::proc::Support as _;

mod exec;
mod lang;

pub mod prompt;

pub use self::lang::{Command, Keyword};

type Result<T, E = Error> = std::result::Result<T, E>;

/// Interactive debugger object.
#[derive(Debug, Default)]
pub struct Debugger {
    // Application
    cycle: usize,
    line: Option<Box<dyn Prompt>>,
    log: Option<Portal<String>>,
    // Console
    pc: u16,
    state: State,
    // Internal
    play: bool,
    freq: Tick,
    step: Option<usize>,
    prog: Option<Program>,
    prev: Option<Program>,
    bpts: IndexMap<u16, Option<Breakpoint>>,
}

impl Debugger {
    /// Constructs a new `Debugger` instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the logging handle.
    ///
    /// Used to change the logging level filter.
    pub fn logger(&mut self, log: Portal<String>) {
        self.log = Some(log);
    }

    /// Sets the prompt handle.
    ///
    /// Used to prompt the user for commands.
    pub fn prompt(&mut self, line: Box<dyn Prompt>) {
        self.line = Some(line);
    }

    /// Enables the debugger.
    pub fn enable(&mut self) {
        self.step = Some(0);
    }

    /// Resumes console emulation.
    pub fn resume(&mut self) {
        self.play = true;
    }

    /// Pauses console emulation.
    pub fn pause(&mut self) {
        self.play = false;
    }

    /// Checks if the console is paused.
    #[must_use]
    pub fn paused(&self) -> bool {
        !self.play
    }

    /// Synchronizes the debugger with the console.
    pub fn sync(&mut self, emu: &GameBoy) {
        // Update program counter
        self.pc = emu.cpu().load(reg::Word::PC);
        self.state = State {
            cpu: emu.cpu().stage().clone(),
            dot: emu.ppu().dot(),
            ppu: emu.ppu().mode().clone(),
        };
    }

    /// Informs the user of the current emulation context.
    ///
    /// # Panics
    ///
    /// Cannot panic.
    pub fn inform(&self, emu: &GameBoy) {
        // Give context if recently paused
        if self.play {
            exec::list(self, emu).unwrap();
        }
    }

    /// Runs interactive debugger.
    ///
    /// # Errors
    ///
    /// Errors if the debugger failed to fetch, parse, or execute a command.
    pub fn run(&mut self, emu: &mut GameBoy, clk: &mut Option<Clock>) -> Result<()> {
        // Provide information to user before prompting for command
        self.inform(emu);
        // Prompt and execute commands until emulation resumed
        self.pause();
        'gbd: while self.paused() {
            let res = 'res: {
                // Attempt to fetch command
                let cmd = {
                    // Attempt to fetch the next command
                    if let cmd @ Some(_) = self.fetch() {
                        // It worked; use it
                        cmd
                    } else {
                        // Pause clock while awaiting user input
                        clk.as_mut().map(Clock::pause);
                        // Couldn't fetch; get program from user
                        match self.readline() {
                            // Program input; fetch next iteration
                            Ok(()) => continue 'gbd,
                            // No input; repeat previous program
                            Err(Error::NoInput) => {
                                // Re-use previous program
                                self.prog = self.prev.clone();
                                debug!("repeat program: `{:?}`", self.prog);
                                // Fetch command from repeated program
                                self.fetch()
                            }
                            // Prompt error; handle upstream
                            err @ Err(_) => {
                                // Clear previous program
                                self.prev = None;
                                // Raise prompt error upwards
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
                self.exec(emu, cmd)
            };
            match res {
                Ok(()) => (),
                err @ Err(Error::Quit) => return err,
                Err(err) => advise::error!("{err}"),
            }
        }

        // Unconditionally resume the clock
        clk.as_mut().map(Clock::resume);

        Ok(())
    }

    /// Prompts the user for a debugger program.
    ///
    /// # Errors
    ///
    /// Errors if the prompt failed.
    pub fn readline(&mut self) -> Result<()> {
        // Extract the prompt handle
        let line = self.line.as_mut().ok_or(Error::ConfigurePrompt)?;

        // Present the prompt; get input
        let fmt = format!("(#{} @ {:#06x})> ", self.cycle, self.pc);
        let input = match line.prompt(&fmt) {
            Err(prompt::Error::Quit) => return Err(Error::Quit),
            res => res?,
        };

        // Parse input
        let prog: Program = input.trim().parse()?;
        debug!("parsed program: `{prog:?}`");

        // Determine outcome
        if prog.is_empty() {
            // Remove stored program
            self.prog = None;
            // Report no input
            Err(Error::NoInput)
        } else {
            // Store program; update previous
            self.prog = Some(prog);
            self.prev = self.prog.clone();
            // Return successfully
            Ok(())
        }
    }

    /// Fetches the next command of the debugger program.
    pub fn fetch(&mut self) -> Option<Command> {
        self.prog.as_mut()?.pop_front()
    }

    /// Executes a debugger command.
    ///
    /// # Errors
    ///
    /// Errors if the command execution failed.
    ///
    /// # Panics
    ///
    /// Cannot panic.
    #[rustfmt::skip]
    #[allow(clippy::enum_glob_use)]
    pub fn exec(&mut self, emu: &mut GameBoy, cmd: Command) -> Result<()> {
        use Command::*;

        // Perform the command
        match cmd {
            Break(addr)             => exec::r#break(self, addr),
            Continue                => exec::r#continue(self, ),
            Delete(point)           => exec::delete(self, point),
            Disable(point)          => exec::disable(self, point),
            Enable(point)           => exec::enable(self, point),
            Freq(mode)              => exec::freq(self, mode),
            Goto(addr)              => exec::goto(emu, addr),
            Help(what)              => exec::help(what),
            Ignore(point, many)     => exec::ignore(self, point, many),
            Info(what)              => exec::info(self, what),
            Jump(addr)              => exec::jump(self, emu, addr),
            List                    => exec::list(self, emu),
            Load(loc)               => exec::loads(emu, loc),
            Log(filter)             => exec::log(self, filter),
            Print(path, force)      => exec::print(emu, &path, force),
            Quit                    => exec::quit(),
            Read(addr)              => exec::read(emu, addr),
            ReadRange(range)        => exec::read_range(emu, range),
            Reset                   => exec::reset(self, emu),
            Serial(mode)            => exec::serial(emu, mode),
            Step(many)              => exec::step(self, many),
            Store(loc, value)       => exec::stores(emu, loc, value),
            Write(addr, byte)       => exec::write(emu, addr, byte),
            WriteRange(range, byte) => exec::write_range(emu, range, byte),
        }
    }

    /// Returns whether the current cycle is an active edge cycle.
    ///
    /// Depending on the [`Debugger`]'s frequency setting, the definition of an
    /// edge may differ.
    #[rustfmt::skip]
    fn edge(&self) -> bool {
        // Pre-calculate machine cycle
        let mcycle = self.cycle % 4 == 0;
        let ppudot = self.state.dot == 0;
        // Check if this is an edge cycle
        match self.freq {
            Tick::Dot   => true,
            Tick::Mach  => mcycle,
            Tick::Insn  => mcycle && matches!(self.state.cpu, cpu::Stage::Done),
            Tick::Line  => ppudot,
            Tick::Frame => ppudot && matches!(self.state.ppu, ppu::Mode::Scan(_)),
        }
    }
}

impl Block for Debugger {
    fn reset(&mut self) {
        // Application
        std::mem::take(&mut self.cycle);
    }
}

impl Machine for Debugger {
    fn enabled(&self) -> bool {
        // Is this an edge cycle?
        let edge = self.edge();
        // Is this cycle being stepped over?
        let step = self.step != Some(0);
        // Are we at a breakpoint?
        let bpt = self
            .bpts
            .get(&self.pc)
            .and_then(Option::as_ref)
            .map_or(false, |bpt| !bpt.disable && bpt.ignore == 0);
        // Should we enable the debugger?
        edge && (!step || bpt)
    }

    fn cycle(&mut self) {
        // Update application cycle count
        self.cycle += 1;
        // Yield on non-edge cycles
        if !self.edge() {
            return;
        }
        // Handle stepped over cycles
        if let Some(step) = &mut self.step {
            // Decrement step count
            *step = step.saturating_sub(1);
        }
        // Handle ignored breakpoints
        if let Some(bpt) = self.bpts.get_mut(&self.pc).and_then(Option::as_mut) {
            // Decrement ignore count
            bpt.ignore = bpt.ignore.saturating_sub(1);
        }
    }
}

/// Debugging breakpoint metadata.
#[derive(Clone, Debug, Default)]
struct Breakpoint {
    disable: bool,
    ignore: usize,
}

impl Breakpoint {
    fn display(&self, point: usize, addr: u16) -> impl Display {
        let &Self { disable, ignore } = self;

        // Prepare format string
        let mut f = String::new();

        // Format the point, addr
        write!(f, "breakpoint {point} @ {addr:#06x}").unwrap();
        // Format characteristics
        if disable {
            write!(f, ": disabled").unwrap();
        } else if ignore > 0 {
            write!(f, ": will ignore next {ignore} crossings").unwrap();
        }

        f
    }
}

/// Debugger progress unit.
#[derive(Clone, Copy, Debug, Default)]
pub enum Tick {
    /// T-stage.
    ///
    /// True system clock notably used by the PPU; occurs at a frequency of 4
    /// MiHz.
    Dot,
    /// M-cycle.
    ///
    /// Basic clock for the CPU; always exactly 4 [dots][`Tick::Dot`].
    #[default]
    Mach,
    /// Instruction.
    ///
    /// Equal to the duration of the current instruction; always 1-6
    /// [M-cycles][`Tick::Mach`].
    Insn,
    /// Scanline.
    ///
    /// Duration to draw one line of the LCD display; always exactly 456
    /// [dots][`Tick::Dot`].
    Line,
    /// Frame.
    ///
    /// Duration to draw an entire frame of the LCD display; always exactly 154
    /// [scanlines][`Tick::Line`] (due to 10 extra v-blank scanlines).
    Frame,
}

#[rustfmt::skip]
impl Display for Tick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Dot   => "dot",
                Self::Mach  => "mach",
                Self::Insn  => "instruction",
                Self::Line  => "scanline",
                Self::Frame => "frame",
            }
        )
    }
}

/// Emulation state.
#[derive(Debug, Default)]
struct State {
    cpu: cpu::Stage,
    dot: u16,
    ppu: ppu::Mode,
}

/// An opaque [getter](Self::get) and [setter](Self::set) for a computed value.
pub struct Portal<T> {
    /// Get the value.
    pub get: Box<dyn Fn() -> T + Send>,
    /// Set the value.
    pub set: Box<dyn FnMut(T) + Send>,
}

impl<T: Debug> Debug for Portal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Portal")
            .field(&format!("{:?}", (self.get)()))
            .finish()
    }
}

/// A type specifying categories of [`Debugger`] errors.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    /// Requested breakpoint could not be found.
    #[error("breakpoint not found")]
    Breakpoint,
    /// Prompt has not been configured.
    #[error("prompt not configured")]
    ConfigurePrompt,
    /// Logger has not been configured.
    #[error("logger not configured")]
    ConfigureLogger,
    /// Image encoding error.
    #[error(transparent)]
    Image(#[from] png::EncodingError),
    /// Command parsing error.
    #[error(transparent)]
    Language(#[from] lang::Error),
    /// Prompt returned empty string.
    #[error("no input provided")]
    NoInput,
    /// Prompt returned an error.
    #[error(transparent)]
    Prompt(#[from] prompt::Error),
    /// Quit requested by user.
    #[error("quit requested by user")]
    Quit,
    /// I/O operation failed.
    #[error("serial I/O failed")]
    Serial(#[from] std::io::Error),
    /// Attempted an unsupported operation.
    #[error("operation not supported")]
    Unsupported,
    /// Provided value does not match expectation.
    #[error("unexpected value")]
    Value,
}
