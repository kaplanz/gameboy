use remus::Cell;

use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Pop(Pop::Pop1)
}

#[derive(Clone, Debug, Default)]
pub enum Pop {
    #[default]
    Pop1,
    Pop0,
    Delay,
}

impl Execute for Pop {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Pop1  => pop1(code, cpu),
            Self::Pop0  => pop0(code, cpu),
            Self::Delay => delay(code, cpu),
        }
    }
}

impl From<Pop> for Operation {
    fn from(value: Pop) -> Self {
        Self::Pop(value)
    }
}

fn pop1(code: u8, cpu: &mut Cpu) -> Return {
    // Pop LSB <- [SP++]
    let mut lsb = cpu.popbyte();
    if code == 0xf1 {
        lsb &= 0xf0; // pop1 4 bits of F cannot be changed
    }

    // Store LSB
    match code {
        0xc1 => &mut cpu.file.c,
        0xd1 => &mut cpu.file.e,
        0xe1 => &mut cpu.file.l,
        0xf1 => &mut cpu.file.f,
        code => return Err(Error::Opcode(code)),
    }
    .store(lsb);

    // Proceed
    Ok(Some(Pop::Pop0.into()))
}

fn pop0(code: u8, cpu: &mut Cpu) -> Return {
    // Pop MSB <- [SP++]
    let msb = cpu.popbyte();

    // Store MSB
    match code {
        0xc1 => &mut cpu.file.b,
        0xd1 => &mut cpu.file.d,
        0xe1 => &mut cpu.file.h,
        0xf1 => &mut cpu.file.a,
        code => return Err(Error::Opcode(code)),
    }
    .store(msb);

    // Finish
    Ok(Some(Pop::Delay.into()))
}

fn delay(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
