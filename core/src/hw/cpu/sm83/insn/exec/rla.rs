use enuf::Enuf;
use remus::Cell;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Rla(Rla::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Rla {
    #[default]
    Execute,
}

impl Execute for Rla {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Rla> for Operation {
    fn from(value: Rla) -> Self {
        Self::Rla(value)
    }
}

fn execute(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x17 {
        return Err(Error::Opcode(code));
    }

    // Execute RLA
    let flags = &mut cpu.file.f.load();
    let acc = cpu.file.a.load();
    let cin = Flag::C.get(flags);
    let carry = acc & 0x80 != 0;
    let res = acc << 1 | (cin as u8);
    cpu.file.a.store(res);

    // Set flags
    Flag::Z.set(flags, false);
    Flag::N.set(flags, false);
    Flag::H.set(flags, false);
    Flag::C.set(flags, carry);
    cpu.file.f.store(*flags);

    // Finish
    Ok(None)
}
