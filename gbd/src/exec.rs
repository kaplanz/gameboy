#![allow(clippy::unnecessary_wraps)]

use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::ops::Not;
use std::path::Path;

use derange::Derange;
use rugby::arch::reg::Port;
use rugby::arch::Block;
use rugby::core::dmg::LCD;
use rugby::prelude::*;

use super::lang::{Keyword, Select, Serial, Value};
use super::{Debugger, Error, GameBoy, Result, Tick};
use crate::Breakpoint;

pub fn r#break(gbd: &mut Debugger, addr: u16) -> Result<()> {
    // Check if the breakpoint already exists
    if let Some((point, _, Some(_))) = gbd.bpts.get_full_mut(&addr) {
        // Inform of existing breakpoint
        advise::warn!("breakpoint {point} already exists at {addr:#06x}");
    } else {
        // Create a new breakpoint
        let (point, _) = gbd.bpts.insert_full(addr, Some(Breakpoint::default()));
        advise::info!("breakpoint {point} created");
    }

    Ok(())
}

pub fn capture(emu: &mut GameBoy, path: &Path, force: bool) -> Result<()> {
    // Process screen data
    let lcd = emu
        // extract frame buffer
        .video()
        .screen()
        // convert pixels to 2-bit value
        .map(|pix| pix as u8)
        // combine every 4 pixels (2bpp) into a byte
        .chunks_exact(4)
        .map(|cols| cols.iter().fold(0, |acc, &pix| (acc << 2) | pix))
        // invert data (`Color::C0` is usually lightest)
        .map(Not::not)
        // collect as a fixed-size array
        .collect::<Box<_>>();
    // Create image file
    let mut file = if force {
        // Forcefully overwrite existing files
        File::create
    } else {
        // Error if the file exists
        File::create_new
    }(path)?;
    // Declare image properties
    let mut encoder = png::Encoder::new(&mut file, LCD.wd.into(), LCD.ht.into());
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Two);
    // Write image to file
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&lcd)?;
    advise::info!("wrote to file: `{}`", path.display());

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
        return Err(Error::Breakpoint);
    };
    // Mark it as deleted
    *bpt = None;
    advise::info!("breakpoint {point} @ {addr:#06x} deleted");

    Ok(())
}

pub fn disable(gbd: &mut Debugger, point: usize) -> Result<()> {
    // Find the specified breakpoint
    let Some((&addr, Some(bpt))) = gbd.bpts.get_index_mut(point) else {
        return Err(Error::Breakpoint);
    };
    // Disable it
    bpt.disable = true;
    advise::info!("breakpoint {point} @ {addr:#06x} disabled");

    Ok(())
}

pub fn enable(gbd: &mut Debugger, point: usize) -> Result<()> {
    // Find the specified breakpoint
    let Some((&addr, Some(bpt))) = gbd.bpts.get_index_mut(point) else {
        return Err(Error::Breakpoint);
    };
    // Enable it
    bpt.disable = false;
    advise::info!("breakpoint {point} @ {addr:#06x} enabled");

    Ok(())
}

pub fn freq(gbd: &mut Debugger, mode: Option<Tick>) -> Result<()> {
    // Change the current frequency
    if let Some(mode) = mode {
        gbd.freq = mode;
    }
    // Print the current frequency
    advise::info!("frequency set to {mode}", mode = gbd.freq);

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
        advise::info!("{line}");
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
        return Err(Error::Breakpoint);
    };
    // Update ignore count
    bpt.ignore = many;
    advise::info!("{}", bpt.display(point, addr));

    Ok(())
}

pub fn info(gbd: &Debugger, what: Option<Keyword>) -> Result<()> {
    // Extract keyword
    let Some(kword) = what else {
        // Print help message when no keyword supplied
        advise::error!("missing keyword");
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
                advise::info!("no breakpoints set");
            } else {
                // Print each breakpoint
                for (point, addr, bpt) in bpts {
                    advise::info!("{}", bpt.display(point, addr));
                }
            }
        }
        _ => return Err(Error::Unsupported),
    }

    Ok(())
}

pub fn list(gbd: &Debugger, emu: &GameBoy) -> Result<()> {
    let insn = emu.cpu().insn();
    advise::info!(
        "{addr:#06x}: {opcode:02X} ; {insn}",
        addr = gbd.pc,
        opcode = insn.opcode()
    );

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn log(gbd: &mut Debugger, filter: Option<String>) -> Result<()> {
    // Extract the logger handle
    let log = gbd.log.as_mut().ok_or(Error::CfgLogger)?;

    // Change the tracing filter
    if let Some(filter) = filter {
        (log.set)(filter);
    }

    // Print the current filter
    advise::info!("filter: {}", (log.get)());

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn loads(emu: &GameBoy, locs: Vec<Select>) -> Result<()> {
    locs.into_iter().try_for_each(|loc| load(emu, loc))
}

#[allow(clippy::needless_pass_by_value)]
pub fn load(emu: &GameBoy, loc: Select) -> Result<()> {
    // Perform the load
    match loc {
        Select::Byte(reg) => {
            let byte: u8 = emu.cpu().load(reg);
            advise::info!("{reg:?}: {byte:#04x}");
        }
        Select::Word(reg) => {
            let word: u16 = emu.cpu().load(reg);
            advise::info!("{reg:?}: {word:#06x}");
        }
        Select::Pic(reg) => {
            let byte: u8 = emu.pic().load(reg);
            advise::info!("{reg:?}: {byte:#04x}");
        }
        Select::Ppu(reg) => {
            let byte: u8 = emu.video().load(reg);
            advise::info!("{reg:?}: {byte:#04x}");
        }
        Select::Serial(reg) => {
            let byte: u8 = emu.serial().load(reg);
            advise::info!("{reg:?}: {byte:#04x}");
        }
        Select::Timer(reg) => {
            let byte: u8 = emu.timer().load(reg);
            advise::info!("{reg:?}: {byte:#04x}");
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
    advise::info!("{addr:#06x}: {byte:02x}");

    Ok(())
}

pub fn read_range(emu: &mut GameBoy, range: Derange<u16>) -> Result<()> {
    // Create iterator from range
    let Derange { start, .. } = range.clone();
    let iter = range.into_iter();
    // Load all reads
    let data: Vec<_> = iter.map(|addr| emu.cpu().read(addr)).collect();
    // Display results
    advise::info!("read {nbytes} bytes:", nbytes = data.len(),);
    let data = format!("{}", phex::Printer::<u8>::new(start.into(), &data));
    for line in data.split('\n') {
        advise::info!("{line}");
    }

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

pub fn serial(emu: &mut GameBoy, mode: Serial) -> Result<()> {
    match mode {
        Serial::Peek | Serial::Recv => {
            // Receive serial data
            let mut data = Vec::new();
            let nbytes = match mode {
                // Peek without draining output buffer
                Serial::Peek => {
                    data.extend_from_slice(emu.serial_mut().rx().fill_buf()?);
                    data.len()
                }
                // Read, consuming output buffer
                Serial::Recv => emu.serial_mut().rx().read_to_end(&mut data)?,
                Serial::Send(_) => unreachable!(),
            };
            // Decode assuming ASCII representation
            let text = std::str::from_utf8(&data);
            // Display results
            advise::info!("received {nbytes} bytes");
            if nbytes > 0 {
                advise::debug!("raw: {data:?}");
                match text {
                    Ok(text) => advise::debug!("txt: {text:?}"),
                    Err(err) => advise::warn!("could not decode: {err}"),
                }
            }
        }
        Serial::Send(data) => {
            // Transmit serial data
            let nbytes = emu.serial_mut().tx().write(&data)?;
            let extra = data.len() - nbytes;
            // Display results
            advise::info!("transmitted {nbytes} bytes");
            if extra > 0 {
                advise::warn!("could not transmit {extra} bytes");
            }
        }
    }

    Ok(())
}

pub fn step(gbd: &mut Debugger, many: Option<usize>) -> Result<()> {
    gbd.step = many.or(Some(0)); // set step count
    gbd.resume(); // resume console

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn stores(emu: &mut GameBoy, locs: Vec<Select>, value: Value) -> Result<()> {
    locs.into_iter()
        .try_for_each(|loc| store(emu, loc, value.clone()))
}

#[allow(clippy::needless_pass_by_value)]
pub fn store(emu: &mut GameBoy, loc: Select, value: Value) -> Result<()> {
    match loc {
        Select::Byte(reg) => {
            // Extract the byte
            let Value::Byte(byte) = value else {
                return Err(Error::Value);
            };
            // Perform the store
            emu.cpu_mut().store(reg, byte);
        }
        Select::Word(reg) => {
            // Extract the byte
            let Value::Word(word) = value else {
                return Err(Error::Value);
            };
            // Perform the store
            emu.cpu_mut().store(reg, word);
        }
        Select::Pic(reg) => {
            // Extract the byte
            let Value::Byte(byte) = value else {
                return Err(Error::Value);
            };
            // Perform the store
            emu.pic_mut().store(reg, byte);
        }
        Select::Ppu(reg) => {
            // Extract the byte
            let Value::Byte(byte) = value else {
                return Err(Error::Value);
            };
            // Perform the store
            emu.video_mut().store(reg, byte);
        }
        Select::Serial(reg) => {
            // Extract the byte
            let Value::Byte(byte) = value else {
                return Err(Error::Value);
            };
            // Perform the store
            emu.serial_mut().store(reg, byte);
        }
        Select::Timer(reg) => {
            // Extract the byte
            let Value::Byte(byte) = value else {
                return Err(Error::Value);
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
        advise::warn!("ignored write {addr:#06x} <- {byte:02x} (retained: {data:02x})");
    }
    // Read the written value
    read(emu, addr)?;

    Ok(())
}

pub fn write_range(emu: &mut GameBoy, range: Derange<u16>, byte: u8) -> Result<()> {
    // Create iterator from range
    let Derange { start, end } = range.clone();
    let iter = range.into_iter();
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
        advise::warn!("ignored some writes in {start:#06x}..{end:04x} <- {byte:02x}");
    }
    // Display results
    advise::info!("write {nbytes} bytes:", nbytes = data.len(),);
    let data = format!("{}", phex::Printer::<u8>::new(start.into(), &data));
    for line in data.split('\n') {
        advise::info!("{line}");
    }

    Ok(())
}
