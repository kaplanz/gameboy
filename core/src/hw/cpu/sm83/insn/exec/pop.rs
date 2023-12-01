use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Pop(Pop::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Pop {
    #[default]
    Fetch,
    Delay(u16),
    Execute(u16),
}

impl Execute for Pop {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch         => fetch(code, cpu),
            Self::Delay(word)   => delay(code, cpu, word),
            Self::Execute(word) => execute(code, cpu, word),
        }
    }
}

impl From<Pop> for Operation {
    fn from(value: Pop) -> Self {
        Self::Pop(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0xc1 | 0xd1 | 0xe1 | 0xf1 => (),
        code => return Err(Error::Opcode(code)),
    }

    // Pop r16 <- [SP]
    let mut word = cpu.popword();
    if code == 0xf1 {
        word &= 0xfff0; // lower 4 bits of F cannot be changed
    }

    // Proceed
    Ok(Some(Pop::Delay(word).into()))
}

fn delay(_: u8, _: &mut Cpu, word: u16) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Pop::Execute(word).into()))
}

fn execute(code: u8, cpu: &mut Cpu, word: u16) -> Return {
    // Perform pop
    match code {
        0xc1 => cpu.file.bc,
        0xd1 => cpu.file.de,
        0xe1 => cpu.file.hl,
        0xf1 => cpu.file.af,
        code => return Err(Error::Opcode(code)),
    }
    .store(&mut cpu.file, word);

    // Finish
    Ok(None)
}
