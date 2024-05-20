use enuf::Enuf;
use remus::reg::Register;
use remus::Byte;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Scf(Scf::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Scf {
    #[default]
    Execute,
}

impl Execute for Scf {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Scf> for Operation {
    fn from(value: Scf) -> Self {
        Self::Scf(value)
    }
}

fn execute(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x37 {
        return Err(Error::Opcode(code));
    }

    // Execute SCF
    let flags = &mut cpu.reg.f.load();
    Flag::N.set(flags, false);
    Flag::H.set(flags, false);
    Flag::C.set(flags, true);
    cpu.reg.f.store(*flags);

    // Finish
    Ok(None)
}
