use enuf::Enuf;
use remus::Cell;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Ldw(Ldw::Fetch)
}

#[allow(unused)]
#[derive(Clone, Debug, Default)]
pub enum Ldw {
    #[default]
    Fetch,
    ReadLsb,
    ReadMsb(u8),
    Execute(u16),
    Load(u16),
    WriteLsb(u16, u16),
    WriteMsb(u16, u16),
    Add(u8),
    Delay,
}

impl Execute for Ldw {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch             => fetch(code, cpu),
            Self::ReadLsb           => read_lsb(code, cpu),
            Self::ReadMsb(lsb)      => read_msb(code, cpu, lsb),
            Self::Execute(op2)      => execute(code, cpu, op2),
            Self::Load(a16)         => load(code, cpu, a16),
            Self::WriteLsb(a16, sp) => write_lsb(code, cpu, a16, sp),
            Self::WriteMsb(a16, sp) => write_msb(code, cpu, a16, sp),
            Self::Add(e8)           => add(code, cpu, e8),
            Self::Delay             => delay(code, cpu),
        }
    }
}

impl From<Ldw> for Operation {
    fn from(value: Ldw) -> Self {
        Self::Ldw(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    #[allow(clippy::match_same_arms)]
    match code {
        // LD r16, n16
        0x01 | 0x11 | 0x21 | 0x31 => {
            // Continue
            read_lsb(code, cpu)
        }
        // LD [a16], SP
        0x08 => {
            // Continue
            read_lsb(code, cpu)
        }
        // LD HL, SP + e8
        0xf8 => {
            // Fetch e8 <- [PC++]
            let e8 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Ldw::Add(e8).into()))
        }
        // LD SP, HL
        0xf9 => {
            // Load HL
            let r16 = (cpu.file.hl.load)(&cpu.file);
            // Proceed
            Ok(Some(Ldw::Execute(r16).into()))
        }
        code => Err(Error::Opcode(code)),
    }
}

fn read_lsb(_: u8, cpu: &mut Cpu) -> Return {
    // Fetch LSB <- [PC++]
    let lsb = cpu.fetchbyte();
    // Proceed
    Ok(Some(Ldw::ReadMsb(lsb).into()))
}

fn read_msb(_: u8, cpu: &mut Cpu, lsb: u8) -> Return {
    // Fetch MSB <- [PC++]
    let msb = cpu.fetchbyte();
    // Combine bytes
    let n16 = u16::from_le_bytes([lsb, msb]);

    // Proceed
    Ok(Some(Ldw::Execute(n16).into()))
}

fn execute(code: u8, cpu: &mut Cpu, op2: u16) -> Return {
    // Execute LDW
    let file = &mut cpu.file;
    match code {
        0x01 => (file.bc.store)(file, op2),
        0x08 => return load(code, cpu, op2),
        0x11 => (file.de.store)(file, op2),
        0x21 => (file.hl.store)(file, op2),
        0x31 | 0xf9 => file.sp.store(op2),
        code => return Err(Error::Opcode(code)),
    }

    // Finish
    Ok(None)
}

fn load(_: u8, cpu: &mut Cpu, a16: u16) -> Return {
    // Load SP
    let sp = cpu.file.sp.load();

    // Proceed
    Ok(Some(Ldw::WriteLsb(a16, sp).into()))
}

fn write_lsb(_: u8, cpu: &mut Cpu, a16: u16, sp: u16) -> Return {
    // Write a16 <- lower(SP)
    cpu.write(a16, sp.to_le_bytes()[0]);

    // Proceed
    Ok(Some(Ldw::WriteMsb(a16, sp).into()))
}

fn write_msb(_: u8, cpu: &mut Cpu, mut a16: u16, sp: u16) -> Return {
    // Write a16 + 1 <- upper(SP)
    a16 = a16.wrapping_add(1);
    cpu.write(a16, sp.to_le_bytes()[1]);

    // Finish
    Ok(None)
}

fn add(_: u8, cpu: &mut Cpu, e8: u8) -> Return {
    // LD HL, SP + e8
    let sp = cpu.file.sp.load();
    let e16 = e8 as i8 as u16;
    let res = sp.wrapping_add(e16);
    (cpu.file.hl.store)(&mut cpu.file, res);

    // Set flags
    let flags = &mut cpu.file.f.load();
    Flag::Z.set(flags, false);
    Flag::N.set(flags, false);
    Flag::H.set(flags, 0x000f < (sp & 0x000f) + (e16 & 0x000f));
    Flag::C.set(flags, 0x00ff < (sp & 0x00ff) + (e16 & 0x00ff));
    cpu.file.f.store(*flags);

    // Proceed
    Ok(Some(Ldw::Delay.into()))
}

fn delay(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
