#![allow(clippy::result_large_err)]

use std::fmt::{Display, Write};

use gameboy::core::cpu::sm83::{self, State};
use gameboy::core::cpu::Processor;
use gameboy::dmg::GameBoy;
use indexmap::IndexMap;
use log::debug;
use remus::{Block, Machine};
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
    off: bool,
    skip: usize,
}

impl Breakpoint {
    fn display(&self, point: usize, addr: u16) -> impl Display {
        let &Self { off, skip } = self;

        // Prepare format string
        let mut f = String::new();

        // Format the point, addr
        write!(f, "breakpoint {point} @ {addr:#06x}").unwrap();
        // Format characteristics
        if off {
            write!(f, ": disabled").unwrap();
        } else if skip > 0 {
            write!(f, ": will ignore next {skip} crossings").unwrap();
        }

        f
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Cycle {
    Dot,
    #[default]
    Mach,
    Insn,
}

impl Display for Cycle {
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
    // Application state
    cycle: usize,
    reload: Option<Handle>,
    // Console state
    pc: u16,
    state: State,
    // Internal state
    play: bool,
    prog: Option<Program>,
    prev: Option<Command>,
    freq: Cycle,
    bpts: IndexMap<u16, Option<Breakpoint>>,
    skip: Option<usize>,
    line: Option<Readline>,
}

impl Debugger {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn prog(&self) -> Option<&Program> {
        self.prog.as_ref()
    }

    pub fn prev(&self) -> Option<&Command> {
        self.prev.as_ref()
    }

    pub fn reload(mut self, handle: Handle) -> Self {
        self.reload = Some(handle);
        self
    }

    pub fn enable(&mut self) {
        self.skip = Some(0);
    }

    pub fn resume(&mut self) {
        self.play = true;
    }

    pub fn pause(&mut self) {
        self.play = false;
    }

    pub fn paused(&self) -> bool {
        !self.play
    }

    pub fn sync(&mut self, emu: &GameBoy) {
        // Update program counter
        self.pc = emu.cpu().get(sm83::Register::PC);
        self.state = emu.cpu().state().clone();
    }

    pub fn inform(&self, emu: &GameBoy) {
        // Give context if recently paused
        if self.play {
            exec::list(self, emu).unwrap();
        }
    }

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
        let input = match line.readline(&fmt) {
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

            Err(Error::NoInput)
        } else {
            // Store program
            self.prog = Some(prog);

            Ok(())
        }
    }

    pub fn fetch(&mut self) -> Option<Command> {
        self.prog.as_mut()?.pop_front()
    }

    #[rustfmt::skip]
    #[allow(clippy::enum_glob_use)]
    pub fn exec(&mut self, emu: &mut GameBoy, cmd: Command) -> Result<()> {
        use Command::*;

        // Update internal bookkeeping data
        self.prev = Some(cmd.clone()); // recall previous command

        // Perform the command
        match cmd {
            Break(addr)       => exec::r#break(self, addr),
            Continue          => exec::r#continue(self, ),
            Delete(point)     => exec::delete(self, point),
            Disable(point)    => exec::disable(self, point),
            Enable(point)     => exec::enable(self, point),
            Freq(cycle)       => exec::freq(self, cycle),
            Help(what)        => exec::help(what),
            Info(what)        => exec::info(self, what),
            Jump(addr)        => exec::jump(self, emu, addr),
            List              => exec::list(self, emu),
            Load(reg)         => exec::load(emu, reg),
            Log(filter)       => exec::log(self, filter),
            Quit              => exec::quit(),
            Read(addr)        => exec::read(emu, addr),
            ReadRange(range)  => exec::read_range(emu, range),
            Reset             => exec::reset(emu),
            Skip(point, many) => exec::skip(self, point, many),
            Step(many)        => exec::step(self, many),
            Store(reg, word)  => exec::store(emu, reg, word),
            Write(addr, byte) => exec::write(emu, addr, byte),
            WriteRange(range, byte) => exec::write_range(emu, range, byte),
        }
    }
}

impl Block for Debugger {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Machine for Debugger {
    fn enabled(&self) -> bool {
        let mcycle = self.cycle % 4 == 0;
        let step = match self.freq {
            Cycle::Dot => true,
            Cycle::Mach => mcycle,
            Cycle::Insn => mcycle && matches!(self.state, State::Done),
        };
        let skip = !matches!(self.skip, Some(0));
        let bpt = self
            .bpts
            .get(&self.pc)
            .and_then(Option::as_ref)
            .map_or(false, |bpt| !bpt.off && bpt.skip == 0);
        step && (!skip || bpt)
    }

    fn cycle(&mut self) {
        // Update application cycle count
        self.cycle += 1;
        // Handle skipped cycles
        if let Some(skip) = &mut self.skip {
            // Decrement skip count
            *skip = skip.saturating_sub(1);
        }
        // Handle skipped breakpoints
        if let Some(bpt) = self.bpts.get_mut(&self.pc).and_then(Option::as_mut) {
            // Decrement skip count
            bpt.skip = bpt.skip.saturating_sub(1);
        }
    }
}

/// A type specifying categories of [`Debugger`] parse errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Readline(#[from] ReadlineError),
    #[error("no input provided")]
    NoInput,
    #[error(transparent)]
    Language(#[from] lang::Error),
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
}
