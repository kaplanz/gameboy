use enuf::Enuf;
use remus::Cell;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Rra(Rra::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Rra {
    #[default]
    Execute,
}

impl Execute for Rra {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Rra> for Operation {
    fn from(value: Rra) -> Self {
        Self::Rra(value)
    }
}

fn execute(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x1f {
        return Err(Error::Opcode(code));
    }

    // Execute RRA
    let flags = &cpu.file.f.load();
    let acc = cpu.file.a.load();
    let cin = Flag::C.get(flags);
    let carry = acc & 0x01 != 0;
    let res = ((cin as u8) << 7) | acc >> 1;
    cpu.file.a.store(res);

    // Set flags
    let flags = &mut cpu.file.f.load();
    Flag::Z.set(flags, false);
    Flag::N.set(flags, false);
    Flag::H.set(flags, false);
    Flag::C.set(flags, carry);
    cpu.file.f.store(*flags);

    // Finish
    Ok(None)
}
