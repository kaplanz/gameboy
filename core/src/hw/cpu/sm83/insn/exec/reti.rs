use remus::Cell;

use super::{Cpu, Error, Execute, Ime, Operation, Return};

pub const fn default() -> Operation {
    Operation::Reti(Reti::Pop0)
}

#[derive(Clone, Debug, Default)]
pub enum Reti {
    #[default]
    Pop0,
    Pop1(u8),
    Jump(u16),
    Done,
}

impl Execute for Reti {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Pop0      => pop0(code, cpu),
            Self::Pop1(pc0) => pop1(code, cpu, pc0),
            Self::Jump(pc)  => jump(code, cpu, pc),
            Self::Done      => done(code, cpu),
        }
    }
}

impl From<Reti> for Operation {
    fn from(value: Reti) -> Self {
        Self::Reti(value)
    }
}

fn pop0(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0xd9 {
        return Err(Error::Opcode(code));
    }

    // Pop lower(PC) <- [SP]
    let pc0 = cpu.popbyte();

    // Proceed
    Ok(Some(Reti::Pop1(pc0).into()))
}

fn pop1(_: u8, cpu: &mut Cpu, pc0: u8) -> Return {
    // Pop upper(PC) <- [SP + 1]
    let pc1 = cpu.popbyte();
    // Combine into PC
    let pc = u16::from_le_bytes([pc0, pc1]);

    // Proceed
    Ok(Some(Reti::Jump(pc).into()))
}

fn jump(_: u8, cpu: &mut Cpu, pc: u16) -> Return {
    // Perform jump
    cpu.file.pc.store(pc);

    // Proceed
    Ok(Some(Reti::Done.into()))
}

fn done(_: u8, cpu: &mut Cpu) -> Return {
    // Enable interrupts
    cpu.ime = Ime::WillEnable;

    // Finish
    Ok(None)
}
