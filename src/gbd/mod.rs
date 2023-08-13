#![allow(clippy::result_large_err)]

use std::cmp::Ordering;
use std::fmt::{Display, Write};
use std::ops::Range;

use gameboy::core::cpu::sm83::{self, Register, State};
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

use self::lang::{Command, Keyword, Program};
use super::Handle;

mod lang;

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Default)]
struct Breakpoint {
    off: bool,
    skip: usize,
}

impl Breakpoint {
    fn display(&self, point: usize, addr: u16) -> impl Display {
        let &Self { off: disable, skip } = self;

        // Prepare format string
        let mut f = String::new();

        // Format the point, addr
        write!(f, "breakpoint {point} @ {addr:#06x}").unwrap();
        // Format any skips
        if skip > 0 {
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

#[allow(clippy::unnecessary_wraps)]
#[allow(clippy::unused_self)]
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
            self.list(emu).unwrap();
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
            Break(addr)       => self.r#break(addr),
            Continue          => self.r#continue(),
            Delete(point)     => self.delete(point),
            Disable(point)    => self.disable(point),
            Enable(point)     => self.benable(point),
            Freq(cycle)       => self.freq(cycle),
            Help(what)        => self.help(what),
            Info(what)        => self.info(what),
            Jump(addr)        => self.jump(emu, addr),
            List              => self.list(emu),
            Load(reg)         => self.load(emu, reg),
            Log(filter)       => self.log(filter),
            Quit              => self.quit(),
            Read(addr)        => self.read(emu, addr),
            ReadRange(range)  => self.read_range(emu, range),
            Reset             => Self::reset(self, emu),
            Skip(point, many) => self.skip(point, many),
            Step(many)        => self.step(many),
            Store(reg, word)  => self.store(emu, reg, word),
            Write(addr, byte) => self.write(emu, addr, byte),
            WriteRange(range, byte) => self.write_range(emu, range, byte),
        }
    }

    fn r#break(&mut self, addr: u16) -> Result<()> {
        // Check if the breakpoint already exists
        if let Some((point, _, Some(_))) = self.bpts.get_full_mut(&addr) {
            // Inform of existing breakpoint
            tell::warn!("breakpoint {point} already exists at {addr:#06x}");
        } else {
            // Create a new breakpoint
            let (point, _) = self.bpts.insert_full(addr, Some(Breakpoint::default()));
            tell::info!("breakpoint {point} created");
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
        let Some((&addr, bpt @ Some(_))) = self.bpts.get_index_mut(point) else {
            return Err(Error::PointNotFound);
        };
        // Mark it as deleted
        *bpt = None;
        tell::info!("breakpoint {point} @ {addr:#06x} deleted");

        Ok(())
    }

    fn disable(&mut self, point: usize) -> Result<()> {
        // Find the specified breakpoint
        let Some((&addr, Some(bpt))) = self.bpts.get_index_mut(point) else {
            return Err(Error::PointNotFound);
        };
        // Disable it
        bpt.off = true;
        tell::info!("breakpoint {point} @ {addr:#06x} disabled");

        Ok(())
    }

    fn benable(&mut self, point: usize) -> Result<()> {
        // Find the specified breakpoint
        let Some((&addr, Some(bpt))) = self.bpts.get_index_mut(point) else {
            return Err(Error::PointNotFound);
        };
        // Enable it
        bpt.off = false;
        tell::info!("breakpoint {point} @ {addr:#06x} enabled");

        Ok(())
    }

    fn freq(&mut self, cycle: Cycle) -> Result<()> {
        // Change the current frequency
        self.freq = cycle;
        tell::info!("frequency set to {cycle}");

        Ok(())
    }

    fn help(&self, what: Option<Keyword>) -> Result<()> {
        if let Some(what) = what {
            debug!("help: `{what:?}`");
        }
        tell::error!("help is not yet available");

        Ok(())
    }

    fn jump(&mut self, emu: &mut GameBoy, addr: u16) -> Result<()> {
        // Jump to specified address
        emu.cpu_mut().goto(addr);
        // Continue execution
        self.r#continue()?;

        Ok(())
    }

    fn info(&self, what: Option<Keyword>) -> Result<()> {
        // Extract keyword
        let Some(kword) = what else {
            // Print help message when no keyword supplied
            tell::error!("missing keyword");
            return self.help(Some(Keyword::Info));
        };

        // Handle keyword
        match kword {
            // Print breakpoints
            Keyword::Break => {
                let bpts: Vec<_> = self
                    .bpts
                    .iter()
                    // Add breakpoint indices
                    .enumerate()
                    // Filter out deleted breakpoints
                    .filter_map(|(point, (&addr, bpt))| bpt.as_ref().map(|bpt| (point, addr, bpt)))
                    .collect();
                if bpts.is_empty() {
                    // Print empty message
                    tell::info!("no breakpoints set");
                } else {
                    // Print each breakpoint
                    for (point, addr, bpt) in bpts {
                        tell::info!("{}", bpt.display(point, addr));
                    }
                }
            }
            _ => return Err(Error::Unsupported),
        }

        Ok(())
    }

    fn list(&self, emu: &GameBoy) -> Result<()> {
        let insn = match &emu.cpu().state() {
            State::Execute(insn) => insn.clone(),
            _ => emu.cpu().insn(),
        };
        tell::info!(
            "{addr:#06x}: {opcode:02X} ; {insn}",
            addr = self.pc,
            opcode = insn.opcode()
        );

        Ok(())
    }

    #[allow(clippy::needless_pass_by_value)]
    fn log(&mut self, filter: Option<String>) -> Result<()> {
        // Extract the reload handle
        let handle = self.reload.as_mut().ok_or(Error::MissingReloadHandle)?;

        // Change the tracing filter
        if let Some(filter) = filter {
            handle.reload(filter)?;
        }

        // Print the current filter
        handle.with_current(|filter| tell::info!("filter: {filter}"))?;

        Ok(())
    }

    fn load(&self, emu: &GameBoy, reg: Register) -> Result<()> {
        // Perform the load
        let word = emu.cpu().get(reg);
        tell::info!("{reg:?}: {word:#04x}");

        Ok(())
    }

    fn quit(&self) -> Result<()> {
        Err(Error::Quit)
    }

    fn read(&self, emu: &mut GameBoy, addr: u16) -> Result<()> {
        // Perform the read
        let byte = emu.cpu().read(addr);
        tell::info!("{addr:#06x}: {byte:02x}");

        Ok(())
    }

    fn read_range(&self, emu: &mut GameBoy, range: Range<u16>) -> Result<()> {
        // Allow range to wrap
        let Range { start, end } = range;
        let iter: Box<dyn Iterator<Item = u16>> = match start.cmp(&end) {
            Ordering::Less => Box::new(start..end),
            Ordering::Equal => return Ok(()),
            Ordering::Greater => {
                tell::warn!("wrapping range for `read`");
                Box::new((start..u16::MAX).chain(u16::MIN..end))
            }
        };
        // Load all reads
        let data: Vec<_> = iter.map(|addr| emu.cpu().read(addr)).collect();
        // Display results
        tell::info!("{}", phex::Printer::<u8>::new(start.into(), &data));

        Ok(())
    }

    fn reset(&self, emu: &mut GameBoy) -> Result<()> {
        // Reset the console
        emu.reset();

        Ok(())
    }

    fn skip(&mut self, point: usize, many: usize) -> Result<()> {
        // Find the specified breakpoint
        let Some((&addr, Some(bpt))) = self.bpts.get_index_mut(point) else {
            return Err(Error::PointNotFound);
        };
        // Update the amount of skips
        bpt.skip = many;
        tell::info!("{}", bpt.display(point, addr));

        Ok(())
    }

    fn step(&mut self, many: Option<usize>) -> Result<()> {
        self.skip = many.or(Some(0)); // set skipped cycles
        self.resume(); // resume console

        Ok(())
    }

    fn store(&self, emu: &mut GameBoy, reg: Register, word: u16) -> Result<()> {
        // Perform the store
        emu.cpu_mut().set(reg, word);
        // Read the stored value
        self.load(emu, reg)?;

        Ok(())
    }

    fn write(&self, emu: &mut GameBoy, addr: u16, byte: u8) -> Result<()> {
        // Perform the write
        emu.cpu().write(addr, byte);
        let read = emu.cpu().read(addr);
        if read != byte {
            tell::warn!("ignored write {addr:#06x} <- {byte:02x} (retained: {read:02x})");
        }
        // Read the written value
        self.read(emu, addr)?;

        Ok(())
    }

    fn write_range(&self, emu: &mut GameBoy, range: Range<u16>, byte: u8) -> Result<()> {
        // Allow range to wrap
        let Range { start, end } = range;
        let iter: Box<dyn Iterator<Item = u16>> = match start.cmp(&end) {
            Ordering::Less => Box::new(start..end),
            Ordering::Equal => return Ok(()),
            Ordering::Greater => {
                tell::warn!("wrapping range for `write`");
                Box::new((start..u16::MAX).chain(u16::MIN..end))
            }
        };
        // Store all writes
        let data: Vec<_> = iter
            .map(|addr| {
                // Perform the write
                emu.cpu().write(addr, byte);
                // Read the written value
                emu.cpu().read(addr)
            })
            .collect();
        // See if it worked
        let worked = data.iter().all(|&read| read == byte);
        if !worked {
            tell::warn!("ignored some writes in {start:#06x}..{end:04x} <- {byte:02x}");
        }
        // Display results
        tell::info!("{}", phex::Printer::<u8>::new(start.into(), &data));

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
