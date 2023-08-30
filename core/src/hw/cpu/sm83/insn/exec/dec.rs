use super::*;

pub const fn default() -> Operation {
    Operation::Dec(Dec::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Dec {
    #[default]
    Fetch,
    Execute(u8),
    Delay,
}

impl Execute for Dec {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Execute(op1) => execute(code, cpu, op1),
            Self::Delay => delay(code, cpu),
        }
    }
}

impl From<Dec> for Operation {
    fn from(value: Dec) -> Self {
        Self::Dec(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x35 => {
            // Read [HL]
            let op1 = cpu.readbyte();
            // Proceed
            Ok(Some(Dec::Execute(op1).into()))
        }
        0x05 => {
            // Load B
            let op1 = cpu.file.b.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x0d => {
            // Load C
            let op1 = cpu.file.c.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x15 => {
            // Load D
            let op1 = cpu.file.d.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x1d => {
            // Load E
            let op1 = cpu.file.e.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x25 => {
            // Load H
            let op1 = cpu.file.h.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x2d => {
            // Load L
            let op1 = cpu.file.l.load();
            // Continue
            execute(code, cpu, op1)
        }
        0x3d => {
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
    // Execute DEC
    let res = op1.wrapping_sub(1);

    // Set flags
    let flags = &mut cpu.file.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, true);
    Flag::H.set(flags, op1 & 0x0f == 0);
    cpu.file.f.store(*flags);

    // Check opcode
    match code {
        0x35 => {
            // Write [HL]
            cpu.writebyte(res);
            // Proceed
            Ok(Some(Dec::Delay.into()))
        }
        0x05 => {
            // Store B
            cpu.file.b.store(res);
            // Finish
            Ok(None)
        }
        0x0d => {
            // Store C
            cpu.file.c.store(res);
            // Finish
            Ok(None)
        }
        0x15 => {
            // Store D
            cpu.file.d.store(res);
            // Finish
            Ok(None)
        }
        0x1d => {
            // Store E
            cpu.file.e.store(res);
            // Finish
            Ok(None)
        }
        0x25 => {
            // Store H
            cpu.file.h.store(res);
            // Finish
            Ok(None)
        }
        0x2d => {
            // Store L
            cpu.file.l.store(res);
            // Finish
            Ok(None)
        }
        0x3d => {
            // Store A
            cpu.file.a.store(res);
            // Finish
            Ok(None)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn delay(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
