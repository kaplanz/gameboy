use enuf::Enuf;
use remus::Cell;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Inc(Inc::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Inc {
    #[default]
    Fetch,
    Execute(u8),
    Done,
}

impl Execute for Inc {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
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

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
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
            let op1 = cpu.file.b.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x0c => {
            // Load C
            let op1 = cpu.file.c.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x14 => {
            // Load D
            let op1 = cpu.file.d.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x1c => {
            // Load E
            let op1 = cpu.file.e.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x24 => {
            // Load H
            let op1 = cpu.file.h.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x2c => {
            // Load L
            let op1 = cpu.file.l.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x3c => {
            // Load A
            let op1 = cpu.file.a.load();
            // Continue
            execute(code, cpu, op1)
        }
        code => Err(Error::Opcode(code)),
    }
}

#[allow(clippy::verbose_bit_mask)]
fn execute(code: u8, cpu: &mut Cpu, op1: u8) -> Return {
    // Execute INC
    let res = op1.wrapping_add(1);

    // Set flags
    let flags = &mut cpu.file.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, res & 0x0f == 0);
    cpu.file.f.store(*flags);

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
            cpu.file.b.store(res);
            // Finish
            Ok(None)
        }
        0x0c => {
            // Store C
            cpu.file.c.store(res);
            // Finish
            Ok(None)
        }
        0x14 => {
            // Store D
            cpu.file.d.store(res);
            // Finish
            Ok(None)
        }
        0x1c => {
            // Store E
            cpu.file.e.store(res);
            // Finish
            Ok(None)
        }
        0x24 => {
            // Store H
            cpu.file.h.store(res);
            // Finish
            Ok(None)
        }
        0x2c => {
            // Store L
            cpu.file.l.store(res);
            // Finish
            Ok(None)
        }
        0x3c => {
            // Store A
            cpu.file.a.store(res);
            // Finish
            Ok(None)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn done(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
