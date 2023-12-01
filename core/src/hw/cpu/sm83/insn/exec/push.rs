use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Push(Push::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Push {
    #[default]
    Fetch,
    Push(u16),
    Delay,
    Done,
}

impl Execute for Push {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch     => fetch(code, cpu),
            Self::Push(op2) => push(code, cpu, op2),
            Self::Delay     => delay(code, cpu),
            Self::Done      => done(code, cpu),
        }
    }
}

impl From<Push> for Operation {
    fn from(value: Push) -> Self {
        Self::Push(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    let word = match code {
        0xc5 => cpu.file.bc,
        0xd5 => cpu.file.de,
        0xe5 => cpu.file.hl,
        0xf5 => cpu.file.af,
        code => return Err(Error::Opcode(code)),
    }
    .load(&cpu.file);

    // Proceed
    Ok(Some(Push::Push(word).into()))
}

fn push(_: u8, cpu: &mut Cpu, word: u16) -> Return {
    // Perform push
    cpu.pushword(word);

    // Proceed
    Ok(Some(Push::Delay.into()))
}

fn delay(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Push::Done.into()))
}

fn done(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
