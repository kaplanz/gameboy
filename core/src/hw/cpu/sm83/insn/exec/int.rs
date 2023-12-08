use remus::Cell;

use super::{Cpu, Error, Execute, Ime, Operation, Return};
use crate::hw::pic::Interrupt;

#[derive(Clone, Debug, Default)]
pub enum Int {
    #[default]
    Fetch,
    Nop,
    Push0,
    Push1,
    Jump,
}

impl Execute for Int {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Nop   => nop(code, cpu),
            Self::Push0 => push0(code, cpu),
            Self::Push1 => push1(code, cpu),
            Self::Jump  => jump(code, cpu),
        }
    }
}

impl From<Int> for Operation {
    fn from(value: Int) -> Self {
        Self::Int(value)
    }
}

fn fetch(_: u8, cpu: &mut Cpu) -> Return {
    // Disable interrupts
    cpu.ime = Ime::Disabled;

    // Proceed
    Ok(Some(Int::Nop.into()))
}

fn nop(_: u8, _: &mut Cpu) -> Return {
    // Execute NOP

    // Proceed
    Ok(Some(Int::Push0.into()))
}

fn push0(_: u8, cpu: &mut Cpu) -> Return {
    // Load MSB
    let msb = cpu.file.pc.load().to_le_bytes()[1];

    // Push MSB -> [--SP]
    cpu.pushbyte(msb);

    // Proceed
    Ok(Some(Int::Push1.into()))
}

fn push1(_: u8, cpu: &mut Cpu) -> Return {
    // Load LSB
    let lsb = cpu.file.pc.load().to_le_bytes()[0];

    // Push LSB -> [--SP]
    cpu.pushbyte(lsb);

    // Proceed
    Ok(Some(Int::Jump.into()))
}

fn jump(code: u8, cpu: &mut Cpu) -> Return {
    // Perform jump
    let int = Interrupt::try_from(code).map_err(|_| Error::Opcode(code))?;
    cpu.file.pc.store(int.handler() as u16);

    // Finish
    Ok(None)
}
