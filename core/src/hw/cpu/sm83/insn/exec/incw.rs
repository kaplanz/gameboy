use remus::Cell;

use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Incw(Incw::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Incw {
    #[default]
    Fetch,
    Execute(u16),
}

impl Execute for Incw {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Execute(op1) => execute(code, cpu, op1),
        }
    }
}

impl From<Incw> for Operation {
    fn from(value: Incw) -> Self {
        Self::Incw(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    let op1 = match code {
        0x03 => cpu.file.bc.load(&cpu.file),
        0x13 => cpu.file.de.load(&cpu.file),
        0x23 => cpu.file.hl.load(&cpu.file),
        0x33 => cpu.file.sp.load(),
        code => return Err(Error::Opcode(code)),
    };

    // Proceed
    Ok(Some(Incw::Execute(op1).into()))
}

fn execute(code: u8, cpu: &mut Cpu, op1: u16) -> Return {
    // Execute INCW
    let res = op1.wrapping_add(1);
    match code {
        0x03 => {
            (cpu.file.bc.store)(&mut cpu.file, res);
        }
        0x13 => {
            (cpu.file.de.store)(&mut cpu.file, res);
        }
        0x23 => {
            (cpu.file.hl.store)(&mut cpu.file, res);
        }
        0x33 => cpu.file.sp.store(res),
        code => return Err(Error::Opcode(code)),
    }

    // Finish
    Ok(None)
}
