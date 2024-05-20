use enuf::Enuf;
use remus::reg::Register;
use remus::Byte;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Cpl(Cpl::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Cpl {
    #[default]
    Execute,
}

impl Execute for Cpl {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Cpl> for Operation {
    fn from(value: Cpl) -> Self {
        Self::Cpl(value)
    }
}

fn execute(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x2f {
        return Err(Error::Opcode(code));
    }

    // Execute CPL
    let acc = cpu.reg.a.load();
    let res = !acc;
    cpu.reg.a.store(res);

    // Set flags
    let flags = &mut cpu.reg.f.load();
    Flag::N.set(flags, true);
    Flag::H.set(flags, true);
    cpu.reg.f.store(*flags);

    // Finish
    Ok(None)
}
