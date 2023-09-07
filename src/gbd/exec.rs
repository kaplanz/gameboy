#![allow(clippy::unnecessary_wraps)]

use std::cmp::Ordering;
use std::io::Read;
use std::ops::Range;

use gameboy::dmg::cpu::Processor;
use remus::{Block, Location as _};

use super::lang::{Keyword, Location, Value};
use super::{Debugger, Error, GameBoy, Mode, Result};
use crate::gbd::Breakpoint;

pub fn r#break(gbd: &mut Debugger, addr: u16) -> Result<()> {
    // Check if the breakpoint already exists
    if let Some((point, _, Some(_))) = gbd.bpts.get_full_mut(&addr) {
        // Inform of existing breakpoint
        tell::warn!("breakpoint {point} already exists at {addr:#06x}");
    } else {
        // Create a new breakpoint
        let (point, _) = gbd.bpts.insert_full(addr, Some(Breakpoint::default()));
        tell::info!("breakpoint {point} created");
    }

    Ok(())
}

pub fn r#continue(gbd: &mut Debugger) -> Result<()> {
    gbd.step = None; // reset step count
    gbd.resume(); // resume console

    Ok(())
}

pub fn delete(gbd: &mut Debugger, point: usize) -> Result<()> {
    // Find the specified breakpoint
    let Some((&addr, bpt @ Some(_))) = gbd.bpts.get_index_mut(point) else {
        return Err(Error::PointNotFound);
    };
    // Mark it as deleted
    *bpt = None;
    tell::info!("breakpoint {point} @ {addr:#06x} deleted");

    Ok(())
}

pub fn disable(gbd: &mut Debugger, point: usize) -> Result<()> {
    // Find the specified breakpoint
    let Some((&addr, Some(bpt))) = gbd.bpts.get_index_mut(point) else {
        return Err(Error::PointNotFound);
    };
    // Disable it
    bpt.disable = true;
    tell::info!("breakpoint {point} @ {addr:#06x} disabled");

    Ok(())
}

pub fn enable(gbd: &mut Debugger, point: usize) -> Result<()> {
    // Find the specified breakpoint
    let Some((&addr, Some(bpt))) = gbd.bpts.get_index_mut(point) else {
        return Err(Error::PointNotFound);
    };
    // Enable it
    bpt.disable = false;
    tell::info!("breakpoint {point} @ {addr:#06x} enabled");

    Ok(())
}

pub fn freq(gbd: &mut Debugger, mode: Mode) -> Result<()> {
    // Change the current frequency
    gbd.freq = mode;
    tell::info!("frequency set to {mode}");

    Ok(())
}

pub fn goto(emu: &mut GameBoy, addr: u16) -> Result<()> {
    // Jump to specified address
    emu.cpu_mut().goto(addr);

    Ok(())
}

pub fn help(what: Option<Keyword>) -> Result<()> {
    // Extract the keyword
    let what = what.unwrap_or(Keyword::All);
    // Print help info
    let help = format!("{what}");
    for line in help.split('\n') {
        tell::info!("{line}");
    }

    Ok(())
}

pub fn jump(gbd: &mut Debugger, emu: &mut GameBoy, addr: u16) -> Result<()> {
    // Jump to specified address
    emu.cpu_mut().goto(addr);
    // Continue execution
    r#continue(gbd)?;

    Ok(())
}

pub fn ignore(gbd: &mut Debugger, point: usize, many: usize) -> Result<()> {
    // Find the specified breakpoint
    let Some((&addr, Some(bpt))) = gbd.bpts.get_index_mut(point) else {
        return Err(Error::PointNotFound);
    };
    // Update ignore count
    bpt.ignore = many;
    tell::info!("{}", bpt.display(point, addr));

    Ok(())
}

pub fn info(gbd: &Debugger, what: Option<Keyword>) -> Result<()> {
    // Extract keyword
    let Some(kword) = what else {
        // Print help message when no keyword supplied
        tell::error!("missing keyword");
        return help(Some(Keyword::Info));
    };

    // Handle keyword
    match kword {
        // Print breakpoints
        Keyword::Break => {
            let bpts: Vec<_> = gbd
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

pub fn list(gbd: &Debugger, emu: &GameBoy) -> Result<()> {
    let insn = emu.cpu().insn();
    tell::info!(
        "{addr:#06x}: {opcode:02X} ; {insn}",
        addr = gbd.pc,
        opcode = insn.opcode()
    );

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn log(gbd: &mut Debugger, filter: Option<String>) -> Result<()> {
    // Extract the reload handle
    let handle = gbd.log.as_mut().ok_or(Error::MissingReloadHandle)?;

    // Change the tracing filter
    if let Some(filter) = filter {
        handle.reload(filter)?;
    }

    // Print the current filter
    handle.with_current(|filter| tell::info!("filter: {filter}"))?;

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn load(emu: &GameBoy, loc: Location) -> Result<()> {
    // Perform the load
    match loc {
        Location::Byte(reg) => {
            let byte: u8 = emu.cpu().load(reg);
            tell::info!("{reg:?}: {byte:#04x}");
        }
        Location::Word(reg) => {
            let word: u16 = emu.cpu().load(reg);
            tell::info!("{reg:?}: {word:#06x}");
        }
        Location::Ppu(reg) => {
            let byte: u8 = emu.ppu().load(reg);
            tell::info!("{reg:?}: {byte:#04x}");
        }
        Location::Timer(reg) => {
            let byte: u8 = emu.timer().load(reg);
            tell::info!("{reg:?}: {byte:#04x}");
        }
    };

    Ok(())
}

pub fn quit() -> Result<()> {
    Err(Error::Quit)
}

pub fn read(emu: &mut GameBoy, addr: u16) -> Result<()> {
    // Perform the read
    let byte = emu.cpu().read(addr);
    tell::info!("{addr:#06x}: {byte:02x}");

    Ok(())
}

pub fn read_range(emu: &mut GameBoy, range: Range<u16>) -> Result<()> {
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
    tell::info!(
        "read {nbytes} bytes:\n{data}",
        nbytes = data.len(),
        data = phex::Printer::<u8>::new(start.into(), &data)
    );

    Ok(())
}

pub fn reset(gbd: &mut Debugger, emu: &mut GameBoy) -> Result<()> {
    // Reset the console
    emu.reset();
    // Reset and sync the debugger
    gbd.reset();
    gbd.sync(emu);

    Ok(())
}

pub fn serial(emu: &mut GameBoy) -> Result<()> {
    // Receive serial data
    let mut tx = Vec::new();
    let nbytes = emu.serial_mut().read_to_end(&mut tx)?;
    // Get string representation of data
    let repr = String::from_utf8_lossy(&tx);
    // Display results
    tell::info!("received {nbytes} bytes");
    if nbytes > 0 {
        tell::debug!("raw: {tx:?}");
        tell::debug!("repr: {repr:?}");
    }

    Ok(())
}

pub fn step(gbd: &mut Debugger, many: Option<usize>) -> Result<()> {
    gbd.step = many.or(Some(0)); // set step count
    gbd.resume(); // resume console

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn store(emu: &mut GameBoy, loc: Location, value: Value) -> Result<()> {
    match loc {
        Location::Byte(reg) => {
            // Extract the byte
            let Value::Byte(byte) = value else {
                return Err(Error::ValueMismatch);
            };
            // Perform the store
            emu.cpu_mut().store(reg, byte);
        }
        Location::Word(reg) => {
            // Extract the byte
            let Value::Word(word) = value else {
                return Err(Error::ValueMismatch);
            };
            // Perform the store
            emu.cpu_mut().store(reg, word);
        }
        Location::Ppu(reg) => {
            // Extract the byte
            let Value::Byte(byte) = value else {
                return Err(Error::ValueMismatch);
            };
            // Perform the store
            emu.ppu_mut().store(reg, byte);
        }
        Location::Timer(reg) => {
            // Extract the byte
            let Value::Byte(byte) = value else {
                return Err(Error::ValueMismatch);
            };
            // Perform the store
            emu.timer_mut().store(reg, byte);
        }
    }
    // Read the stored value
    load(emu, loc)?;

    Ok(())
}

pub fn write(emu: &mut GameBoy, addr: u16, byte: u8) -> Result<()> {
    // Perform the write
    emu.cpu_mut().write(addr, byte);
    let data = emu.cpu().read(addr);
    if data != byte {
        tell::warn!("ignored write {addr:#06x} <- {byte:02x} (retained: {data:02x})");
    }
    // Read the written value
    read(emu, addr)?;

    Ok(())
}

pub fn write_range(emu: &mut GameBoy, range: Range<u16>, byte: u8) -> Result<()> {
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
            emu.cpu_mut().write(addr, byte);
            // Check the written value
            emu.cpu().read(addr)
        })
        .collect();
    // Check if it worked
    let nbytes = bytecount::count(&data, byte);
    if nbytes < data.len() {
        tell::warn!("ignored some writes in {start:#06x}..{end:04x} <- {byte:02x}");
    }
    // Display results
    tell::info!(
        "wrote {nbytes} bytes:\n{data}",
        data = phex::Printer::<u8>::new(start.into(), &data)
    );

    Ok(())
}
