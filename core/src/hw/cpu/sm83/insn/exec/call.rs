use enuf::Enuf;
use remus::Cell;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Call(Call::Fetch0)
}

#[derive(Clone, Debug, Default)]
pub enum Call {
    #[default]
    Fetch0,
    Fetch1(u8),
    Check(u16),
    Push0(u16),
    Push1(u16),
    Jump(u16),
}

impl Execute for Call {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch0      => fetch0(code, cpu),
            Self::Fetch1(lsb) => fetch1(code, cpu, lsb),
            Self::Check(a16)  => check(code, cpu, a16),
            Self::Push0(a16)  => push0(code, cpu, a16),
            Self::Push1(a16)  => push1(code, cpu, a16),
            Self::Jump(a16)   => jump(code, cpu, a16),
        }
    }
}

impl From<Call> for Operation {
    fn from(value: Call) -> Self {
        Self::Call(value)
    }
}

fn fetch0(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0xc4 | 0xcc | 0xcd | 0xd4 | 0xdc => (),
        code => return Err(Error::Opcode(code)),
    }

    // Fetch lower(a16) <- [PC++]
    let lsb = cpu.fetchbyte();
    // Proceed
    Ok(Some(Call::Fetch1(lsb).into()))
}

fn fetch1(_: u8, cpu: &mut Cpu, lsb: u8) -> Return {
    // Fetch upper(a16) <- [PC++]
    let msb = cpu.fetchbyte();
    // Combine into a16
    let a16 = u16::from_le_bytes([lsb, msb]);

    // Proceed
    Ok(Some(Call::Check(a16).into()))
}

fn check(code: u8, cpu: &mut Cpu, a16: u16) -> Return {
    // Evaluate condition
    let flags = &cpu.file.f.load();
    #[rustfmt::skip]
    let cond = match code {
        0xc4 => !Flag::Z.get(flags),
        0xcc =>  Flag::Z.get(flags),
        0xd4 => !Flag::C.get(flags),
        0xdc =>  Flag::C.get(flags),
        0xcd => true,
        code => return Err(Error::Opcode(code)),
    };

    // Check condition
    if cond {
        // Proceed
        Ok(Some(Call::Push0(a16).into()))
    } else {
        // Finish
        Ok(None)
    }
}

fn push0(_: u8, cpu: &mut Cpu, a16: u16) -> Return {
    // Load PC
    let pc = cpu.file.pc.load().to_le_bytes();
    // Push [--SP] <- upper(PC++)
    cpu.pushbyte(pc[1]);

    // Proceed
    Ok(Some(Call::Push1(a16).into()))
}

fn push1(_: u8, cpu: &mut Cpu, a16: u16) -> Return {
    // Load PC
    let pc = cpu.file.pc.load().to_le_bytes();
    // Push [--SP] <- lower(PC++)
    cpu.pushbyte(pc[0]);

    // Proceed
    Ok(Some(Call::Jump(a16).into()))
}

fn jump(_: u8, cpu: &mut Cpu, a16: u16) -> Return {
    // Perform jump
    cpu.file.pc.store(a16);

    // Finish
    Ok(None)
}
