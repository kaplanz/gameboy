use super::*;

pub const fn default() -> Operation {
    Operation::Rrca(Rrca::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Rrca {
    #[default]
    Execute,
}

impl Execute for Rrca {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Rrca> for Operation {
    fn from(value: Rrca) -> Self {
        Self::Rrca(value)
    }
}

fn execute(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x0f {
        return Err(Error::Opcode(code));
    }

    // Execute RRCA
    let acc = cpu.file.a.load();
    let carry = acc & 0x01 != 0;
    let res = ((carry as u8) << 7) | acc >> 1;
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
