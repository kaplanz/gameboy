use enuf::Enuf;
use remus::reg::Register;
use remus::Byte;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Inc(Inc::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Inc {
    #[default]
    Fetch,
    Execute(Byte),
    Done,
}

impl Execute for Inc {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op1) => execute(code, cpu, op1),
            Self::Done         => done(code, cpu),
        }
    }
}

impl From<Inc> for Operation {
    fn from(value: Inc) -> Self {
        Self::Inc(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x34 => {
            // Read [HL]
            let op1 = cpu.readbyte();
            // Proceed
            Ok(Some(Inc::Execute(op1).into()))
        }
        0x04 => {
            // Load B
            let op1 = cpu.reg.b.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x0c => {
            // Load C
            let op1 = cpu.reg.c.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x14 => {
            // Load D
            let op1 = cpu.reg.d.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x1c => {
            // Load E
            let op1 = cpu.reg.e.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x24 => {
            // Load H
            let op1 = cpu.reg.h.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x2c => {
            // Load L
            let op1 = cpu.reg.l.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x3c => {
            // Load A
            let op1 = cpu.reg.a.load();
            // Continue
            execute(code, cpu, op1)
        }
        code => Err(Error::Opcode(code)),
    }
}

#[allow(clippy::verbose_bit_mask)]
fn execute(code: Byte, cpu: &mut Cpu, op1: Byte) -> Return {
    // Execute INC
    let res = op1.wrapping_add(1);

    // Set flags
    let flags = &mut cpu.reg.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, res & 0x0f == 0);
    cpu.reg.f.store(*flags);

    // Check opcode
    match code {
        0x34 => {
            // Write [HL]
            cpu.writebyte(res);
            // Proceed
            Ok(Some(Inc::Done.into()))
        }
        0x04 => {
            // Store B
            cpu.reg.b.store(res);
            // Finish
            Ok(None)
        }
        0x0c => {
            // Store C
            cpu.reg.c.store(res);
            // Finish
            Ok(None)
        }
        0x14 => {
            // Store D
            cpu.reg.d.store(res);
            // Finish
            Ok(None)
        }
        0x1c => {
            // Store E
            cpu.reg.e.store(res);
            // Finish
            Ok(None)
        }
        0x24 => {
            // Store H
            cpu.reg.h.store(res);
            // Finish
            Ok(None)
        }
        0x2c => {
            // Store L
            cpu.reg.l.store(res);
            // Finish
            Ok(None)
        }
        0x3c => {
            // Store A
            cpu.reg.a.store(res);
            // Finish
            Ok(None)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn done(_: Byte, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
