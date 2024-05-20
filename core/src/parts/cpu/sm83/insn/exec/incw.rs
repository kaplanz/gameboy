use remus::reg::Register;
use remus::Byte;

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
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op1) => execute(code, cpu, op1),
        }
    }
}

impl From<Incw> for Operation {
    fn from(value: Incw) -> Self {
        Self::Incw(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    let op1 = match code {
        0x03 => cpu.reg.bc().load(),
        0x13 => cpu.reg.de().load(),
        0x23 => cpu.reg.hl().load(),
        0x33 => cpu.reg.sp.load(),
        code => return Err(Error::Opcode(code)),
    };

    // Proceed
    Ok(Some(Incw::Execute(op1).into()))
}

fn execute(code: Byte, cpu: &mut Cpu, op1: u16) -> Return {
    // Execute INCW
    let res = op1.wrapping_add(1);
    match code {
        0x03 => {
            cpu.reg.bc_mut().store(res);
        }
        0x13 => {
            cpu.reg.de_mut().store(res);
        }
        0x23 => {
            cpu.reg.hl_mut().store(res);
        }
        0x33 => cpu.reg.sp.store(res),
        code => return Err(Error::Opcode(code)),
    }

    // Finish
    Ok(None)
}
