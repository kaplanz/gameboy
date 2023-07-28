use std::io::{self, Write};
use std::str::FromStr;

use gameboy::core::cpu::{sm83, Processor};
use gameboy::dmg::GameBoy;
use indexmap::IndexMap;
use log::{trace, warn};
use remus::{Block, Machine};
use thiserror::Error;

#[derive(Debug, Default)]
pub struct Debugger {
    bpts: IndexMap<u16, usize>,
    skip: Option<usize>,
    pc: u16,
}

impl Debugger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self) {
        self.skip = Some(0);
    }

    pub fn prompt(&self) -> crate::Result<Command> {
        // Present the prompt
        print!("({:04x}) > ", self.pc);
        io::stdout().flush().map_err(Error::Io)?;

        // Read input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Parse the command
        let cmd = input.parse()?;
        trace!("parsed command: `{cmd:?}`");

        Ok(cmd)
    }

    #[rustfmt::skip]
    pub fn act(&mut self, emu: &mut GameBoy, cmd: Command) {
        use Command::{Break, Continue, List, Read, Skip, Step, Write};
        // Update internal bookkeeping data
        self.pc = emu.cpu().get(sm83::Register::PC);

        // Perform the command
        match cmd {
            Break(addr)       => self.r#break(addr),
            Continue          => self.r#continue(),
            List              => self.list(),
            Read(addr)        => self.read(emu, addr),
            Skip(point, many) => self.skip(point, many),
            Step              => self.step(),
            Write(addr, byte) => self.write(emu, addr, byte),
        }
    }

    fn r#break(&mut self, addr: u16) {
        self.bpts.insert(addr, 0);
    }

    fn r#continue(&mut self) {
        self.skip = None;
    }

    fn list(&self) {
        todo!()
    }

    #[allow(clippy::unused_self)]
    fn read(&self, emu: &mut GameBoy, addr: u16) {
        let byte = emu.cpu().read(addr);
        println!("{addr:04x}: {byte:02x}");
    }

    fn skip(&self, _point: usize, _many: usize) {
        todo!()
    }

    #[allow(clippy::unused_self)]
    fn step(&self) {}

    #[allow(clippy::unused_self)]
    fn write(&self, emu: &mut GameBoy, addr: u16, byte: u8) {
        emu.cpu().write(addr, byte);
        let read = emu.cpu().read(addr);
        if read != byte {
            warn!("{addr:04x}: {read:02x} (ignored write: {byte:02x})");
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
        matches!(self.skip, Some(0))
    }

    fn cycle(&mut self) {
        // Handle skipped cycles
        if let Some(skip) = &mut self.skip {
            // Decrement skip count
            *skip = skip.saturating_sub(1);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Command {
    Break(u16),
    Continue,
    List,
    Read(u16),
    Write(u16, u8),
    Skip(usize, usize),
    Step,
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "b" | "break" => Ok(Self::Break(0)),
            "c" | "continue" => Ok(Self::Continue),
            "l" | "list" => Ok(Self::List),
            "r" | "read" => Ok(Self::Read(0x0000)),
            "w" | "write" => Ok(Self::Write(0x0000, 0x00)),
            "skip" => Ok(Self::Skip(0, 0)),
            "s" | "step" => Ok(Self::Step),
            _ => Err(Error::Unknown),
        }
    }
}

/// A type specifying categories of [`Debugger`] parse errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("unknown command")]
    Unknown,
}
