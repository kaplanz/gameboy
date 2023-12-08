use remus::Cell;

use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Push(Push::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Push {
    #[default]
    Fetch,
    Push0,
    Push1,
    Delay,
}

impl Execute for Push {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Push0 => push0(code, cpu),
            Self::Push1 => push1(code, cpu),
            Self::Delay => delay(code, cpu),
        }
    }
}

impl From<Push> for Operation {
    fn from(value: Push) -> Self {
        Self::Push(value)
    }
}

fn fetch(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Push::Push0.into()))
}

fn push0(code: u8, cpu: &mut Cpu) -> Return {
    // Load MSB
    let msb = match code {
        0xc5 => &cpu.file.b,
        0xd5 => &cpu.file.d,
        0xe5 => &cpu.file.h,
        0xf5 => &cpu.file.a,
        code => return Err(Error::Opcode(code)),
    }
    .load();

    // Push MSB
    cpu.pushbyte(msb);

    // Proceed
    Ok(Some(Push::Push1.into()))
}

fn push1(code: u8, cpu: &mut Cpu) -> Return {
    // Load LSB
    let lsb = match code {
        0xc5 => &cpu.file.c,
        0xd5 => &cpu.file.e,
        0xe5 => &cpu.file.l,
        0xf5 => &cpu.file.f,
        code => return Err(Error::Opcode(code)),
    }
    .load();

    // Push LSB
    cpu.pushbyte(lsb);

    // Proceed
    Ok(Some(Push::Delay.into()))
}

fn delay(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
