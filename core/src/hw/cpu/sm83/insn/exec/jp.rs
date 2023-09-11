use enuf::Enuf;
use remus::Cell;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Jp(Jp::Fetch0)
}

#[derive(Clone, Debug, Default)]
pub enum Jp {
    #[default]
    Fetch0,
    Fetch1(u8),
    Check(u16),
    Jump(u16),
}

impl Execute for Jp {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch0     => fetch0(code, cpu),
            Self::Fetch1(a8) => fetch1(code, cpu, a8),
            Self::Check(a16) => check(code, cpu, a16),
            Self::Jump(a16)  => jump(code, cpu, a16),
        }
    }
}

impl From<Jp> for Operation {
    fn from(value: Jp) -> Self {
        Self::Jp(value)
    }
}

fn fetch0(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0xc2 | 0xc3 | 0xca | 0xd2 | 0xda => {
            // Fetch lower(a16) <- [PC]
            let a8 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Jp::Fetch1(a8).into()))
        }
        0xe9 => {
            // Load HL
            let a16 = cpu.file.hl.load(&cpu.file);
            // Continue
            jump(code, cpu, a16)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn fetch1(_: u8, cpu: &mut Cpu, a8: u8) -> Return {
    // Fetch upper(a16) <- [PC + 1]
    let b8 = cpu.fetchbyte();
    // Combine into a16
    let a16 = u16::from_le_bytes([a8, b8]);

    // Proceed
    Ok(Some(Jp::Check(a16).into()))
}

fn check(code: u8, cpu: &mut Cpu, a16: u16) -> Return {
    // Evaluate condition
    let flags = &cpu.file.f.load();
    #[rustfmt::skip]
    let cond = match code {
        0xc2 => !Flag::Z.get(flags),
        0xca =>  Flag::Z.get(flags),
        0xd2 => !Flag::C.get(flags),
        0xda =>  Flag::C.get(flags),
        0xc3 => true,
        code => return Err(Error::Opcode(code)),
    };

    // Check condition
    if cond {
        // Proceed
        Ok(Some(Jp::Jump(a16).into()))
    } else {
        // Finish
        Ok(None)
    }
}

fn jump(_: u8, cpu: &mut Cpu, a16: u16) -> Return {
    // Perform jump
    cpu.file.pc.store(a16);

    // Finish
    Ok(None)
}
