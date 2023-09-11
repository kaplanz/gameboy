use remus::Cell;

use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Rst(Rst::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Rst {
    #[default]
    Fetch,
    Push0(u8),
    Push1(u8, u8),
    Jump(u8),
}

impl Execute for Rst {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch          => fetch(code, cpu),
            Self::Push0(v8)      => push0(code, cpu, v8),
            Self::Push1(v8, pc0) => push1(code, cpu, v8, pc0),
            Self::Jump(v8)       => jump(code, cpu, v8),
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
    let v8 = match code {
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

    // Proceed
    Ok(Some(Rst::Push0(v8).into()))
}

fn push0(_: u8, cpu: &mut Cpu, v8: u8) -> Return {
    // Load PC
    let pc = cpu.file.pc.load().to_le_bytes();
    // Push [SP] <- upper(PC + 2)
    cpu.pushbyte(pc[1]);

    // Proceed
    Ok(Some(Rst::Push1(v8, pc[0]).into()))
}

fn push1(_: u8, cpu: &mut Cpu, v8: u8, pc0: u8) -> Return {
    // Push [SP - 1] <- lower(PC + 2)
    cpu.pushbyte(pc0);

    // Proceed
    Ok(Some(Rst::Jump(v8).into()))
}

fn jump(_: u8, cpu: &mut Cpu, v8: u8) -> Return {
    // Combine into v16
    let v16 = u16::from_le_bytes([v8, 0x00]);
    // Perform jump
    cpu.file.pc.store(v16);

    // Finish
    Ok(None)
}
