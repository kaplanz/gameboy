#![allow(clippy::result_large_err)]

use std::fmt::{Display, Write};

use gameboy::dmg::cpu::{reg, Stage};
use gameboy::dmg::GameBoy;
use indexmap::IndexMap;
use log::debug;
use remus::{Block, Clock, Location, Machine};
use rustyline::error::ReadlineError;
use rustyline::history::History;
use rustyline::DefaultEditor as Readline;
use thiserror::Error;
use tracing_subscriber::reload;

use self::lang::{Command, Program};
use super::Handle;

mod exec;
mod lang;

type Result<T> = std::result::Result<T, Error>;

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

#[derive(Clone, Copy, Debug, Default)]
pub enum Mode {
    Dot,
    #[default]
    Mach,
    Insn,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Dot => "dot",
                Self::Mach => "machine",
                Self::Insn => "instruction",
            }
        )
    }
}

#[derive(Debug, Default)]
pub struct Debugger {
    // Application
    cycle: usize,
    log: Option<Handle>,
    // Console
    pc: u16,
    state: Stage,
    // Internal
    play: bool,
    freq: Mode,
    step: Option<usize>,
    line: Option<Readline>,
    prog: Option<Program>,
    prev: Option<Program>,
    bpts: IndexMap<u16, Option<Breakpoint>>,
}

impl Debugger {
    /// Constructs a new `Debugger` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs interactive debugger.
    pub fn run(&mut self, emu: &mut GameBoy, clk: &mut Option<Clock>) -> Result<()> {
        // Provide information to user before prompting for command
        self.inform(emu);
        // Prompt and execute commands until emulation resumed
        self.pause();
        'gbd: while self.paused() {
            let res = 'res: {
                // Attempt to fetch command
                let cmd =
                    {
                        // Attempt to fetch the next command
                        if let cmd @ Some(_) = self.fetch() {
                            // It worked; use it
                            cmd
                        } else {
                            // Couldn't fetch; get program from user
                            match {
                                // Pause clock while awaiting user input
                                clk.as_mut().map(Clock::pause);
                                // Present the prompt
                                self.prompt()
                            } {
                                // Program input; fetch next iteration
                                Ok(()) => continue 'gbd,
                                // No input; repeat previous program
                                Err(Error::NoInput) => {
                                    // Re-use previous program
                                    self.prog = self.prev.clone();
                                    debug!("repeat program: `{:?}`", self.prog);
                                    // Fetch command from repeated program
                                    self.fetch()
                                },
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
                Err(err) => tell::error!("{err}"),
            }
        }

        // Unconditionally resume the clock
        clk.as_mut().map(Clock::resume);

        Ok(())
    }

    /// Sets the logging handle.
    ///
    /// Used to change the logging level filter.
    pub fn set_log(mut self, handle: Handle) -> Self {
        self.log = Some(handle);
        self
    }

    /// Enables the debugger.
    pub fn enable(&mut self) {
        self.step = Some(0);
    }

    /// Resumes console emulation.
    pub fn resume(&mut self) {
        self.play = true;
    }

    /// Pauses console emuulation.
    pub fn pause(&mut self) {
        self.play = false;
    }

    /// Checks if the console is paused.
    pub fn paused(&self) -> bool {
        !self.play
    }

    /// Synchronizes the debugger with the console.
    pub fn sync(&mut self, emu: &GameBoy) {
        // Update program counter
        self.pc = emu.cpu().load(reg::Word::PC);
        self.state = emu.cpu().stage().clone();
    }

    /// Informs the user of the current emulation context.
    pub fn inform(&self, emu: &GameBoy) {
        // Give context if recently paused
        if self.play {
            exec::list(self, emu).unwrap();
        }
    }

    /// Prompts the user for a debugger program.
    pub fn prompt(&mut self) -> Result<()> {
        // Lazily initialize prompt
        let line = if let Some(line) = &mut self.line {
            line
        } else {
            self.line = Some(Readline::new()?);
            self.line.as_mut().unwrap()
        };

        // Present the prompt; get input
        let fmt = format!("(#{} @ {:#06x})> ", self.cycle, self.pc);
        let input =
            match line.readline(&fmt) {
                Err(ReadlineError::Interrupted) => return Ok(()),
                Err(ReadlineError::Eof) => return Err(Error::Quit),
                res => res?,
            };
        line.history_mut().add(&input)?; // add input to history

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
            Load(loc)               => exec::load(emu, loc),
            Log(filter)             => exec::log(self, filter),
            Quit                    => exec::quit(),
            Read(addr)              => exec::read(emu, addr),
            ReadRange(range)        => exec::read_range(emu, range),
            Reset                   => exec::reset(self, emu),
            Serial                  => exec::serial(emu),
            Step(many)              => exec::step(self, many),
            Store(loc, value)       => exec::store(emu, loc, value),
            Write(addr, byte)       => exec::write(emu, addr, byte),
            WriteRange(range, byte) => exec::write_range(emu, range, byte),
        }
    }

    /// Returns whether the current cycle is an active edge cycle.
    ///
    /// Depending on the [`Debugger`]'s frequency setting, the definition of an
    /// edge may differ.
    fn edge(&self) -> bool {
        // Pre-calculate machine cycle
        let mcycle = self.cycle % 4 == 0;
        // Check if this is an edge cycle
        match self.freq {
            Mode::Dot => true,
            Mode::Mach => mcycle,
            Mode::Insn => mcycle && matches!(self.state, Stage::Done),
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

/// A type specifying categories of [`Debugger`] errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Readline(#[from] ReadlineError),
    #[error("no input provided")]
    NoInput,
    #[error(transparent)]
    Language(#[from] lang::Error),
    #[error("value mismatch")]
    ValueMismatch,
    #[error("missing reload handle")]
    MissingReloadHandle,
    #[error("breakpoint not found")]
    PointNotFound,
    #[error("quit requested by user")]
    Quit,
    #[error(transparent)]
    Tracing(#[from] reload::Error),
    #[error("unsupported keyword")]
    Unsupported,
    #[error("serial I/O failed")]
    Serial(#[from] std::io::Error),
}
