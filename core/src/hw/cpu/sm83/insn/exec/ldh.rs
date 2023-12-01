use remus::Cell;

use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Ldh(Ldh::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Ldh {
    #[default]
    Fetch,
    Execute(u8),
    Delay,
}

impl Execute for Ldh {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op2) => execute(code, cpu, op2),
            Self::Delay        => delay(code, cpu),
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
        0xe0 | 0xf0 => {
            // Fetch a8
            let a8 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Ldh::Execute(a8).into()))
        }
        0xe2 | 0xf2 => {
            // Load C
            let a8 = cpu.file.c.load();
            // Proceed
            execute(code, cpu, a8)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(code: u8, cpu: &mut Cpu, a8: u8) -> Return {
    // Calculate absolute address from relative
    let addr = 0xff00 | a8 as u16;

    // Perform a read/write to the address
    match code {
        0xe0 | 0xe2 => {
            // Execute LDH {a8, C}, A
            let op2 = cpu.file.a.load();
            cpu.write(addr, op2);
        }
        0xf0 | 0xf2 => {
            // Execute LDH A, {a8, C}
            let op2 = cpu.read(addr);
            cpu.file.a.store(op2);
        }
        code => return Err(Error::Opcode(code)),
    }

    // Proceed
    Ok(Some(Ldh::Delay.into()))
}

fn delay(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
