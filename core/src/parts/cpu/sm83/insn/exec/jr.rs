use remus::reg::Register;
use remus::Byte;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Jr(Jr::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Jr {
    #[default]
    Fetch,
    Check(Byte),
    Jump(Byte),
}

impl Execute for Jr {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch     => fetch(code, cpu),
            Self::Check(e8) => check(code, cpu, e8),
            Self::Jump(e8)  => jump(code, cpu, e8),
        }
    }
}

impl From<Jr> for Operation {
    fn from(value: Jr) -> Self {
        Self::Jr(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
            // Fetch e8 <- [PC++]
            let e8 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Jr::Check(e8).into()))
        }
        code => Err(Error::Opcode(code)),
    }
}

fn check(code: Byte, cpu: &mut Cpu, e8: Byte) -> Return {
    // Evaluate condition
    let flags = &cpu.reg.f.load();
    #[rustfmt::skip]
    let cond = match code {
        0x20 => !Flag::Z.get(flags),
        0x28 =>  Flag::Z.get(flags),
        0x30 => !Flag::C.get(flags),
        0x38 =>  Flag::C.get(flags),
        0x18 => true,
        code => return Err(Error::Opcode(code)),
    };

    // Check condition
    if cond {
        // Proceed
        Ok(Some(Jr::Jump(e8).into()))
    } else {
        // Finish
        Ok(None)
    }
}

fn jump(_: Byte, cpu: &mut Cpu, e8: Byte) -> Return {
    // Perform jump
    let e8 = e8 as i8 as i16;
    let pc = cpu.reg.pc.load() as i16;
    let pc = pc.wrapping_add(e8) as u16;
    cpu.reg.pc.store(pc);

    // Finish
    Ok(None)
}
