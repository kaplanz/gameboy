use super::*;

pub const fn default() -> Operation {
    Operation::Ld(Ld::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Ld {
    #[default]
    Fetch,
    Delay,
    Write(u8),
    ExecuteByte(u8),
    DelayWord(u16),
    ExecuteWord(u16),
    Done,
}

impl Execute for Ld {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Delay => delay(code, cpu),
            Self::Write(op2) => write(code, cpu, op2),
            Self::ExecuteByte(op2) => execute_byte(code, cpu, op2),
            Self::DelayWord(word) => delay_word(code, cpu, word),
            Self::ExecuteWord(word) => execute_word(code, cpu, word),
            Self::Done => done(code, cpu),
        }
    }
}

impl From<Ld> for Operation {
    fn from(value: Ld) -> Self {
        Self::Ld(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        // LD [r16], A
        0x02 | 0x12 | 0x22 | 0x32 => {
            // Load r16
            let r16 = match code {
                0x02 => cpu.file.bc,
                0x12 => cpu.file.de,
                0x22 | 0x32 => cpu.file.hl,
                code => return Err(Error::Opcode(code)),
            }
            .load(&cpu.file);
            // Load A
            let op2 = cpu.file.a.load();
            // Write A
            cpu.write(r16, op2);
            // Proceed
            Ok(Some(Ld::Delay.into()))
        }
        // LD A, [r16]
        0x0a | 0x1a | 0x2a | 0x3a => {
            // Load r16
            let r16 = match code {
                0x0a => cpu.file.bc,
                0x1a => cpu.file.de,
                0x2a | 0x3a => cpu.file.hl,
                code => return Err(Error::Opcode(code)),
            }
            .load(&cpu.file);
            // Read r16
            let op2 = cpu.read(r16);
            // Proceed
            Ok(Some(Ld::ExecuteByte(op2).into()))
        }
        // LD r8, n8
        0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x3e => {
            // Fetch n8
            let op2 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Ld::ExecuteByte(op2).into()))
        }
        // LD [HL], n8
        0x36 => {
            // Fetch n8
            let op2 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Ld::Write(op2).into()))
        }
        // LD r8, [HL]
        0x46 | 0x4e | 0x56 | 0x5e | 0x66 | 0x6e | 0x7e => {
            // Load [HL]
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(Ld::ExecuteByte(op2).into()))
        }
        // HALT (unexpected opcode)
        0x76 => Err(Error::Opcode(code)),
        // LD [HL], r8
        0x70..=0x77 => {
            // Load HL
            let addr = cpu.file.hl.load(&cpu.file);
            // Prepare op2
            let op2 = helpers::get_op8(cpu, code & 0x07);
            // Write op2
            cpu.write(addr, op2);
            // Proceed
            Ok(Some(Ld::Delay.into()))
        }
        // LD r8, r8
        0x40..=0x7f => {
            // Prepare op2
            let op2 = helpers::get_op8(cpu, code & 0x07);
            // Continue
            execute_byte(code, cpu, op2)
        }
        // LD [a16], A
        // LD A, [a16]
        0xea | 0xfa => {
            // Fetch a16
            let word = cpu.fetchword();
            // Proceed
            Ok(Some(Ld::DelayWord(word).into()))
        }
        code => Err(Error::Opcode(code)),
    }
}

fn delay(code: u8, cpu: &mut Cpu) -> Return {
    // Delay by 1 cycle

    #[allow(clippy::match_same_arms)]
    match code {
        0x02 | 0x12 => {
            // Continue
            done(code, cpu)
        }
        0x22 | 0x32 => {
            // Continue
            update(code, cpu)
        }
        0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x77 => {
            // Continue
            done(code, cpu)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn update(code: u8, cpu: &mut Cpu) -> Return {
    // Load HL
    let hl = cpu.file.hl;
    let r16 = hl.load(&cpu.file);
    // Update HL
    let res = match code {
        // INC HL
        0x22 | 0x2a => r16.wrapping_add(1),
        // DEC HL
        0x32 | 0x3a => r16.wrapping_sub(1),
        code => return Err(Error::Opcode(code)),
    };
    // Store HL <- HL{+, -}
    hl.store(&mut cpu.file, res);

    // Continue
    done(code, cpu)
}

fn write(code: u8, cpu: &mut Cpu, op2: u8) -> Return {
    // Check opcode
    if code != 0x36 {
        return Err(Error::Opcode(code));
    }

    // Execute [HL], n8
    cpu.writebyte(op2);
    // Proceed
    Ok(Some(Ld::Done.into()))
}

fn execute_byte(code: u8, cpu: &mut Cpu, op2: u8) -> Return {
    // Store r8 <- {r8, n8, [HL]}
    let op1 = match code {
        0x06 | 0x40..=0x47 => &mut cpu.file.b,
        0x0e | 0x48..=0x4f => &mut cpu.file.c,
        0x16 | 0x50..=0x57 => &mut cpu.file.d,
        0x1e | 0x58..=0x5f => &mut cpu.file.e,
        0x26 | 0x60..=0x67 => &mut cpu.file.h,
        0x2e | 0x68..=0x6f => &mut cpu.file.l,
        0x0a | 0x1a | 0x2a | 0x3a | 0x3e | 0x78..=0x7f | 0xf2 => &mut cpu.file.a,
        code => return Err(Error::Opcode(code)),
    };
    op1.store(op2);

    // Continue
    match code {
        0x2a | 0x3a => update(code, cpu),
        _ => done(code, cpu),
    }
}

fn delay_word(_: u8, _: &mut Cpu, word: u16) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Ld::ExecuteWord(word).into()))
}

fn execute_word(code: u8, cpu: &mut Cpu, word: u16) -> Return {
    match code {
        0xea => {
            // Execute LD [a16], A
            let op2 = cpu.file.a.load();
            cpu.write(word, op2);
        }
        0xfa => {
            // Execute LD A, [a16]
            let op2 = cpu.read(word);
            cpu.file.a.store(op2);
        }
        code => return Err(Error::Opcode(code)),
    }

    // Proceed
    Ok(Some(Ld::Done.into()))
}

fn done(_: u8, _: &mut Cpu) -> Return {
    // Finish
    Ok(None)
}
