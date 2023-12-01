use enuf::Enuf;
use remus::Cell;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Addw(Addw::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Addw {
    #[default]
    Fetch,
    Execute(u16, u16),
    Fetch0xE8,
    Delay0xE8(u8),
    Execute0xE8(u8),
}

impl Execute for Addw {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch             => fetch(code, cpu),
            Self::Execute(op1, op2) => execute(code, cpu, op1, op2),
            Self::Fetch0xE8         => fetch_0xe8(code, cpu),
            Self::Delay0xE8(e8)     => delay_0xe8(code, cpu, e8),
            Self::Execute0xE8(e8)   => execute_0xe8(code, cpu, e8),
        }
    }
}

impl From<Addw> for Operation {
    fn from(op: Addw) -> Self {
        Self::Addw(op)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x09 => {
            let op1 = cpu.file.hl.load(&cpu.file);
            let op2 = cpu.file.bc.load(&cpu.file);
            // Proceed
            Ok(Some(Addw::Execute(op1, op2).into()))
        }
        0x19 => {
            let op1 = cpu.file.hl.load(&cpu.file);
            let op2 = cpu.file.de.load(&cpu.file);
            // Proceed
            Ok(Some(Addw::Execute(op1, op2).into()))
        }
        0x29 => {
            let op1 = cpu.file.hl.load(&cpu.file);
            let op2 = cpu.file.hl.load(&cpu.file);
            // Proceed
            Ok(Some(Addw::Execute(op1, op2).into()))
        }
        0x39 => {
            let op1 = cpu.file.hl.load(&cpu.file);
            let op2 = cpu.file.sp.load();
            // Proceed
            Ok(Some(Addw::Execute(op1, op2).into()))
        }
        0xe8 => {
            // Proceed
            Ok(Some(Addw::Fetch0xE8.into()))
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(_: u8, cpu: &mut Cpu, op1: u16, op2: u16) -> Return {
    // Execute ADDW
    let (res, carry) = op1.overflowing_add(op2);
    (cpu.file.hl.store)(&mut cpu.file, res);

    // Set flags
    let flags = &mut cpu.file.f.load();
    Flag::N.set(flags, false);
    Flag::H.set(flags, 0x0fff < (op1 & 0x0fff) + (op2 & 0x0fff));
    Flag::C.set(flags, carry);
    cpu.file.f.store(*flags);

    // Finish
    Ok(None)
}

fn fetch_0xe8(_: u8, cpu: &mut Cpu) -> Return {
    // Fetch e8
    let e8 = cpu.fetchbyte();

    // Proceed
    Ok(Some(Addw::Delay0xE8(e8).into()))
}

fn delay_0xe8(_: u8, _: &mut Cpu, e8: u8) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Addw::Execute0xE8(e8).into()))
}

fn execute_0xe8(_: u8, cpu: &mut Cpu, e8: u8) -> Return {
    // Execute ADDW
    let op1 = cpu.file.sp.load();
    let op2 = e8 as i8 as u16;
    let res = op1.wrapping_add(op2);
    cpu.file.sp.store(res);

    // Set flags
    let flags = &mut cpu.file.f.load();
    Flag::Z.set(flags, false);
    Flag::N.set(flags, false);
    Flag::H.set(flags, 0x000f < (op1 & 0x000f) + (op2 & 0x000f));
    Flag::C.set(flags, 0x00ff < (op1 & 0x00ff) + (op2 & 0x00ff));
    cpu.file.f.store(*flags);

    // Finish
    Ok(None)
}
