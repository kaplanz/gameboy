#![allow(clippy::result_large_err)]

use std::cmp::Ordering;
use std::fmt::Display;
use std::io::{self, Write};
use std::ops::Range;
use std::str::FromStr;

use gameboy::core::cpu::sm83::{self, State};
use gameboy::core::cpu::Processor;
use gameboy::dmg::GameBoy;
use indexmap::IndexMap;
use log::{error, info, trace, warn};
use remus::{Block, Machine};
use thiserror::Error;

mod parser;

type Result<T> = std::result::Result<T, Error>;

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

#[derive(Clone, Debug)]
pub enum Command {
    Break(u16),
    Continue,
    Delete(usize),
    Freq(Cycle),
    Help(Option<String>),
    Info(Option<String>),
    List,
    Quit,
    Read(u16),
    ReadRange(Range<u16>),
    Skip(usize, usize),
    Step,
    Write(u16, u8),
    WriteRange(Range<u16>, u8),
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        parser::parse(s)
            .map_err(Error::Parser)?
            .ok_or(Error::NoInput)
    }
}

#[derive(Debug, Default)]
pub struct Debugger {
    // Application state
    cycle: usize,
    // Console state
    pc: u16,
    state: State,
    // Internal state
    play: bool,
    freq: Cycle,
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
            self.list(emu).unwrap();
        }
    }

    pub fn prompt(&mut self) -> Result<Option<Command>> {
        // Present the prompt
        print!("(#{} @ {:#06x})> ", self.cycle, self.pc);
        io::stdout().flush().map_err(Error::Io)?;

        // Read input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Parse the command
        let cmd = match input.parse() {
            Err(Error::NoInput) => return Ok(self.prev.clone()), // re-use the previous command
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
        self.prev = Some(cmd.clone()); // recall previous command

        // Perform the command
        match cmd {
            Break(addr)       => self.r#break(addr),
            Continue          => self.r#continue(),
            Delete(point)     => self.delete(point),
            Freq(cycle)       => self.freq(cycle),
            Help(what)        => self.help(what),
            Info(what)        => self.info(what),
            List              => self.list(emu),
            Quit              => self.quit(),
            Read(addr)        => self.read(emu, addr),
            ReadRange(range)  => self.read_range(emu, range),
            Skip(point, many) => self.skip(point, many),
            Step              => self.step(),
            Write(addr, byte) => self.write(emu, addr, byte),
            WriteRange(range, byte) => self.write_range(emu, range, byte),
        }
    }

    fn r#break(&mut self, addr: u16) -> Result<()> {
        // Check if the breakpoint already exists
        if let Some((point, _, skip)) = self.bpts.get_full_mut(&addr) {
            // Reset an existing breakpoint
            *skip = 0;
            println!("breakpoint {point} exists; resetting skip count");
        } else {
            // Create a new breakpoint
            self.bpts.insert(addr, 0);
            println!("breakpoint {point} created", point = self.bpts.len() - 1);
        }

        Ok(())
    }

    fn r#continue(&mut self) -> Result<()> {
        self.skip = None; // reset skipped cycles
        self.resume(); // resume console

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

    fn freq(&mut self, cycle: Cycle) -> Result<()> {
        // Change the current frequency
        self.freq = cycle;
        println!("frequency set to {cycle}");

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn help(&self, what: Option<String>) -> Result<()> {
        if let Some(what) = what {
            info!("help: `{what}`");
        }
        error!("help is not yet available");

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn info(&self, what: Option<String>) -> Result<()> {
        if let Some(what) = what {
            info!("info: `{what}`");
        }
        error!("info is not yet available");

        Ok(())
    }

    fn list(&self, emu: &GameBoy) -> Result<()> {
        let insn = match &emu.cpu().state() {
            State::Execute(insn) => insn.clone(),
            _ => emu.cpu().insn(),
        };
        println!(
            "{addr:#06x}: {opcode:02X} ; {insn}",
            addr = self.pc,
            opcode = insn.opcode()
        );

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn quit(&self) -> Result<()> {
        Err(Error::Quit)
    }

    #[allow(clippy::unused_self)]
    fn read(&self, emu: &mut GameBoy, addr: u16) -> Result<()> {
        // Perform a read
        let byte = emu.cpu().read(addr);
        println!("{addr:#06x}: {byte:02x}");

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn read_range(&self, emu: &mut GameBoy, range: Range<u16>) -> Result<()> {
        // Allow range to wrap
        let Range { start, end } = range;
        let iter: Box<dyn Iterator<Item = u16>> = match start.cmp(&end) {
            Ordering::Less => Box::new(start..end),
            Ordering::Equal => return Ok(()),
            Ordering::Greater => {
                warn!("wrapping range for `read`");
                Box::new((start..u16::MAX).chain(u16::MIN..end))
            }
        };
        // Load all reads
        let data: Vec<_> = iter.map(|addr| emu.cpu().read(addr)).collect();
        // Display results
        println!(
            "{}",
            crate::hex::Printer::<u8>::new(start.into(), &data).display()
        );

        Ok(())
    }

    fn skip(&mut self, point: usize, many: usize) -> Result<()> {
        // Find the specified breakpoint
        let (addr, skips) = self.bpts.get_index_mut(point).ok_or(Error::PointNotFound)?;
        // Update the amount of skips
        *skips = many;
        println!("breakpoint {point} @ {addr:#06x}: will ignore next {many} crossings");

        Ok(())
    }

    fn step(&mut self) -> Result<()> {
        self.skip = Some(0); // set no skipped cycles
        self.resume(); // resume console

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn write(&self, emu: &mut GameBoy, addr: u16, byte: u8) -> Result<()> {
        // Perform the write
        emu.cpu().write(addr, byte);
        let read = emu.cpu().read(addr);
        if read != byte {
            warn!("ignored write {addr:#06x} <- {byte:02x} (retained: {read:02x})");
        }
        // Read the written value
        self.read(emu, addr)?;

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn write_range(&self, emu: &mut GameBoy, range: Range<u16>, byte: u8) -> Result<()> {
        // Allow range to wrap
        let Range { start, end } = range;
        let iter: Box<dyn Iterator<Item = u16>> = match start.cmp(&end) {
            Ordering::Less => Box::new(start..end),
            Ordering::Equal => return Ok(()),
            Ordering::Greater => {
                warn!("wrapping range for `write`");
                Box::new((start..u16::MAX).chain(u16::MIN..end))
            }
        };
        // Store all writes
        let worked = iter
            .map(|addr| {
                // Perform the write
                emu.cpu().write(addr, byte);
                // Read the written value
                emu.cpu().read(addr)
            })
            .all(|read| read == byte);
        if !worked {
            warn!("ignored some writes in range");
        }
        // Read the written values
        self.read_range(emu, range)?;

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
        let mcycle = self.cycle % 4 == 0;
        let step = match self.freq {
            Cycle::Dot => true,
            Cycle::Mach => mcycle,
            Cycle::Insn => mcycle && matches!(self.state, State::Done),
        };
        let skip = !matches!(self.skip, Some(0));
        let bkpt = self.bpts.get(&self.pc).map_or(false, |&skips| skips == 0);
        step && (!skip || bkpt)
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
        if let Some(skip) = self.bpts.get_mut(&self.pc) {
            // Decrement skip count
            *skip = skip.saturating_sub(1);
        }
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
    NoInput,
    #[error("requested quit")]
    Quit,
}
