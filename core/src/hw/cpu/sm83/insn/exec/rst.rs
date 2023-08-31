use remus::Cell;

use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Rst(Rst::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Rst {
    #[default]
    Fetch,
    Push,
    Delay,
    Jump,
}

impl Execute for Rst {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Push => push(code, cpu),
            Self::Delay => delay(code, cpu),
            Self::Jump => jump(code, cpu),
        }
    }
}

impl From<Rst> for Operation {
    fn from(value: Rst) -> Self {
        Self::Rst(value)
    }
}

fn fetch(code: u8, _: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => (),
        code => return Err(Error::Opcode(code)),
    };

    // Delay by 1 cycle

    // Proceed
    Ok(Some(Rst::Push.into()))
}

fn push(_: u8, cpu: &mut Cpu) -> Return {
    // Push [SP] <- PC
    let pc = cpu.file.pc.load();
    cpu.pushword(pc);

    // Proceed
    Ok(Some(Rst::Delay.into()))
}

fn delay(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Rst::Jump.into()))
}

fn jump(code: u8, cpu: &mut Cpu) -> Return {
    // Perform jump
    let op1 = match code {
        0xc7 => 0x00,
        0xcf => 0x08,
        0xd7 => 0x10,
        0xdf => 0x18,
        0xe7 => 0x20,
        0xef => 0x28,
        0xf7 => 0x30,
        0xff => 0x38,
        code => return Err(Error::Opcode(code)),
    };
    cpu.file.pc.store(op1);

    // Finish
    Ok(None)
}
