use remus::Cell;

use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Ldh(Ldh::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Ldh {
    #[default]
    Fetch,
    Read(u8),
    Write(u8),
    Delay,
}

impl Execute for Ldh {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch     => fetch(code, cpu),
            Self::Read(a8)  => read(code, cpu, a8),
            Self::Write(a8) => write(code, cpu, a8),
            Self::Delay     => delay(code, cpu),
        }
    }
}

impl From<Ldh> for Operation {
    fn from(value: Ldh) -> Self {
        Self::Ldh(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0xe0 => {
            // Fetch a8 <- [PC++]
            let a8 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Ldh::Write(a8).into()))
        }
        0xf0 => {
            // Fetch a8 <- [PC++]
            let a8 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Ldh::Read(a8).into()))
        }
        0xe2 => {
            // Load C
            let a8 = cpu.file.c.load();
            // Continue
            write(code, cpu, a8)
        }
        0xf2 => {
            // Load C
            let a8 = cpu.file.c.load();
            // Continue
            read(code, cpu, a8)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn read(_: u8, cpu: &mut Cpu, a8: u8) -> Return {
    // Calculate absolute address
    let addr = u16::from_be_bytes([0xff, a8]);

    // Execute LDH B, {a8, C}
    let op2 = cpu.read(addr);
    cpu.file.a.store(op2);

    // Proceed
    Ok(Some(Ldh::Delay.into()))
}

fn write(_: u8, cpu: &mut Cpu, a8: u8) -> Return {
    // Calculate absolute address
    let addr = u16::from_be_bytes([0xff, a8]);

    // Execute LDH {a8, C}, A
    let op2 = cpu.file.a.load();
    cpu.write(addr, op2);

    // Proceed
    Ok(Some(Ldh::Delay.into()))
}

fn delay(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
