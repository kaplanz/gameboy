use super::*;

pub const fn default() -> Operation {
    Operation::Rlca(Rlca::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Rlca {
    #[default]
    Execute,
}

impl Execute for Rlca {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Rlca> for Operation {
    fn from(value: Rlca) -> Self {
        Self::Rlca(value)
    }
}

fn execute(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x07 {
        return Err(Error::Opcode(code));
    }

    // Execute RLCA
    let acc = cpu.file.a.load();
    let carry = acc & 0x80 != 0;
    let res = acc << 1 | (carry as u8);
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
