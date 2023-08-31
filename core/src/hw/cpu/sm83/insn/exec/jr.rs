use enuf::Enuf;
use remus::Cell;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Jr(Jr::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Jr {
    #[default]
    Fetch,
    Check(u8, bool),
    Jump(u8),
}

impl Execute for Jr {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Check(e8, cond) => check(code, cpu, e8, cond),
            Self::Jump(e8) => jump(code, cpu, e8),
        }
    }
}

impl From<Jr> for Operation {
    fn from(value: Jr) -> Self {
        Self::Jr(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
            // Fetch e8
            let e8 = cpu.fetchbyte();
            // Continue
            evaluate(code, cpu, e8)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn evaluate(code: u8, cpu: &mut Cpu, e8: u8) -> Return {
    // Evaluate condition
    let flags = &cpu.file.f.load();
    let cond = match code {
        0x18 => true,
        0x20 => !Flag::Z.get(flags),
        0x28 => Flag::Z.get(flags),
        0x30 => !Flag::C.get(flags),
        0x38 => Flag::C.get(flags),
        code => return Err(Error::Opcode(code)),
    };

    // Proceed
    Ok(Some(Jr::Check(e8, cond).into()))
}

fn check(_: u8, _: &mut Cpu, e8: u8, cond: bool) -> Return {
    // Execute JR
    if cond {
        // Proceed
        Ok(Some(Jr::Jump(e8).into()))
    } else {
        // Finish
        Ok(None)
    }
}

fn jump(_: u8, cpu: &mut Cpu, e8: u8) -> Return {
    // Perform jump
    let pc = cpu.file.pc.load() as i16;
    let e8 = e8 as i8 as i16;
    let res = pc.wrapping_add(e8) as u16;
    cpu.file.pc.store(res);

    // Finish
    Ok(None)
}
