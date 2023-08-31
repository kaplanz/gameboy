use remus::Cell;

use super::{Cpu, Error, Execute, Ime, Operation, Return};

pub const fn default() -> Operation {
    Operation::Reti(Reti::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Reti {
    #[default]
    Fetch,
    Delay(u16),
    Jump(u16),
    Execute,
}

impl Execute for Reti {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Delay(pc) => delay(code, cpu, pc),
            Self::Jump(pc) => jump(code, cpu, pc),
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Reti> for Operation {
    fn from(value: Reti) -> Self {
        Self::Reti(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0xd9 {
        return Err(Error::Opcode(code));
    }

    // Pop PC <- [SP]
    let pc = cpu.popword();

    // Proceed
    Ok(Some(Reti::Delay(pc).into()))
}

fn delay(_: u8, _: &mut Cpu, pc: u16) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Reti::Jump(pc).into()))
}

fn jump(_: u8, cpu: &mut Cpu, pc: u16) -> Return {
    // Perform jump
    cpu.file.pc.store(pc);

    // Proceed
    Ok(Some(Reti::Execute.into()))
}

fn execute(_: u8, cpu: &mut Cpu) -> Return {
    // Enable interrupts
    cpu.ime = Ime::WillEnable;

    // Finish
    Ok(None)
}
