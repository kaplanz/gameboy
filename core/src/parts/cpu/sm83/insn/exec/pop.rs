use rugby_arch::reg::Register;
use rugby_arch::Byte;

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
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
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

fn pop1(code: Byte, cpu: &mut Cpu) -> Return {
    // Pop LSB <- [SP++]
    let mut lsb = cpu.popbyte();
    if code == 0xf1 {
        lsb &= 0xf0; // pop1 4 bits of F cannot be changed
    }

    // Store LSB
    match code {
        0xc1 => &mut cpu.reg.c,
        0xd1 => &mut cpu.reg.e,
        0xe1 => &mut cpu.reg.l,
        0xf1 => &mut cpu.reg.f,
        code => return Err(Error::Opcode(code)),
    }
    .store(lsb);

    // Proceed
    Ok(Some(Pop::Pop0.into()))
}

fn pop0(code: Byte, cpu: &mut Cpu) -> Return {
    // Pop MSB <- [SP++]
    let msb = cpu.popbyte();

    // Store MSB
    match code {
        0xc1 => &mut cpu.reg.b,
        0xd1 => &mut cpu.reg.d,
        0xe1 => &mut cpu.reg.h,
        0xf1 => &mut cpu.reg.a,
        code => return Err(Error::Opcode(code)),
    }
    .store(msb);

    // Finish
    Ok(Some(Pop::Delay.into()))
}

fn delay(_: Byte, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
