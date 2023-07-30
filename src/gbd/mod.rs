#![allow(clippy::result_large_err)]

use std::io::{self, Write};
use std::str::FromStr;

use gameboy::core::cpu::{sm83, Processor};
use gameboy::dmg::GameBoy;
use indexmap::IndexMap;
use log::{error, trace, warn};
use remus::{Block, Machine};
use thiserror::Error;

mod parser;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Default)]
pub struct Debugger {
    // Console state
    pc: u16,
    // Internal state
    play: bool,
    bpts: IndexMap<u16, usize>,
    skip: Option<usize>,
    prev: Option<Command>,
}

#[allow(clippy::unnecessary_wraps)]
impl Debugger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self) {
        self.skip = Some(0);
    }

    pub fn paused(&self) -> bool {
        !self.play
    }

    pub fn sync(&mut self, emu: &GameBoy) {
        // Update program counter
        self.pc = emu.cpu().get(sm83::Register::PC);
    }

    pub fn prompt(&mut self) -> Result<Option<Command>> {
        // Present the prompt
        print!("({:04x})> ", self.pc);
        io::stdout().flush().map_err(Error::Io)?;

        // Read input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Parse the command
        let cmd = match input.parse() {
            Err(Error::Empty) => return Ok(self.prev.clone()), // re-use the previous command
            res => res?,
        };
        trace!("parsed command: `{cmd:?}`");

        Ok(Some(cmd))
    }

    #[rustfmt::skip]
    #[allow(clippy::enum_glob_use)]
    pub fn act(&mut self, emu: &mut GameBoy, cmd: Command) -> Result<()> {
        use Command::*;
        // Update internal bookkeeping data
        self.sync(emu);                // sync with console
        self.play = false;             // pause console
        self.prev = Some(cmd.clone()); // recall previous command

        // Perform the command
        match cmd {
            Break(addr)       => self.r#break(addr),
            Continue          => self.r#continue(),
            Delete(point)     => self.delete(point),
            Help(what)        => self.help(what),
            Info(what)        => self.info(what),
            List              => self.list(),
            Read(addr)        => self.read(emu, addr),
            Skip(point, many) => self.skip(point, many),
            Step              => self.step(),
            Write(addr, byte) => self.write(emu, addr, byte),
        }
    }

    fn r#break(&mut self, addr: u16) -> Result<()> {
        // Check if the breakpoint already exists
        if let Some(point) = self.bpts.get_index_of(&addr) {
            println!("breakpoint {point} exists");
            return Ok(());
        }

        // Create a new breakpoint
        self.bpts.insert(addr, 0);
        println!("breakpoint {point} created", point = self.bpts.len() - 1);

        Ok(())
    }

    fn r#continue(&mut self) -> Result<()> {
        self.skip = None; // reset skipped cycles
        self.play = true; // resume console

        Ok(())
    }

    fn delete(&mut self, point: usize) -> Result<()> {
        // Find the specified breakpoint
        self.bpts
            .swap_remove_index(point)
            .ok_or(Error::PointNotFound)?;
        println!("breakpoint {point} deleted (indices shifted downwards)");

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn help(&self, what: Option<String>) -> Result<()> {
        if let Some(what) = what {
            trace!("help: `{what}`");
        }
        error!("help is not yet available");

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn info(&self, what: Option<String>) -> Result<()> {
        if let Some(what) = what {
            trace!("info: `{what}`");
        }
        error!("info is not yet available");

        Ok(())
    }

    fn list(&self) -> Result<()> {
        todo!()
    }

    #[allow(clippy::unused_self)]
    fn read(&self, emu: &mut GameBoy, addr: u16) -> Result<()> {
        let byte = emu.cpu().read(addr);
        println!("{addr:04x}: {byte:02x}");

        Ok(())
    }

    fn skip(&mut self, point: usize, many: usize) -> Result<()> {
        // Find the specified breakpoint
        let (addr, skips) = self.bpts.get_index_mut(point).ok_or(Error::PointNotFound)?;
        // Update the amount of skips
        *skips = many;
        trace!("breakpoint {point} @ {addr:04x}: skip {many} times");

        Ok(())
    }

    fn step(&mut self) -> Result<()> {
        self.play = true; // resume console

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn write(&self, emu: &mut GameBoy, addr: u16, byte: u8) -> Result<()> {
        emu.cpu().write(addr, byte);
        let read = emu.cpu().read(addr);
        if read != byte {
            warn!("{addr:04x}: {read:02x} (ignored write: {byte:02x})");
        }

        Ok(())
    }
}

impl Block for Debugger {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Machine for Debugger {
    fn enabled(&self) -> bool {
        matches!(self.skip, Some(0)) || self.bpts.get(&self.pc).map_or(false, |&skips| skips == 0)
    }

    fn cycle(&mut self) {
        // Handle skipped cycles
        if let Some(skip) = &mut self.skip {
            // Decrement skip count
            *skip = skip.saturating_sub(1);
        }
        // Handle skipped breakpoints
        if let Some(skip) = self.bpts.get_mut(&self.pc) {
            // Decrement skip count
            *skip = skip.saturating_sub(1);
        }
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    Break(u16),
    Continue,
    Delete(usize),
    Help(Option<String>),
    Info(Option<String>),
    List,
    Read(u16),
    Write(u16, u8),
    Skip(usize, usize),
    Step,
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        parser::parse(s).map_err(Error::Parser)?.ok_or(Error::Empty)
    }
}

/// A type specifying categories of [`Debugger`] parse errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Parser(#[from] parser::Error),
    #[error("breakpoint not found")]
    PointNotFound,
    #[error("no input provided")]
    Empty,
}
