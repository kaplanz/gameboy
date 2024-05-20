use remus::reg::Register;
use remus::Byte;

use super::{help, Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Ld(Ld::Fetch)
}

#[allow(unused)]
#[derive(Clone, Debug, Default)]
pub enum Ld {
    #[default]
    Fetch,
    Update,
    Write(Byte),
    Byte(Byte),
    Fetch0,
    Fetch1(Byte),
    Word(u16),
    Done,
}

impl Execute for Ld {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch       => fetch(code, cpu),
            Self::Update      => update(code, cpu),
            Self::Write(op2)  => write(code, cpu, op2),
            Self::Byte(op2)   => byte(code, cpu, op2),
            Self::Fetch0      => fetch0(code, cpu),
            Self::Fetch1(lsb) => fetch1(code, cpu, lsb),
            Self::Word(a16)   => word(code, cpu, a16),
            Self::Done        => done(code, cpu),
        }
    }
}

impl From<Ld> for Operation {
    fn from(value: Ld) -> Self {
        Self::Ld(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        // LD [r16], A
        0x02 | 0x12 | 0x22 | 0x32 => {
            // Load r16
            let r16 = match code {
                0x02 => cpu.reg.bc(),
                0x12 => cpu.reg.de(),
                0x22 | 0x32 => cpu.reg.hl(),
                code => return Err(Error::Opcode(code)),
            }
            .load();
            // Load A
            let op2 = cpu.reg.a.load();
            // Write A
            cpu.write(r16, op2);
            // Proceed
            match code {
                0x02 | 0x12 => Ok(Some(Ld::Done.into())),
                0x22 | 0x32 => Ok(Some(Ld::Update.into())),
                code => Err(Error::Opcode(code)),
            }
        }
        // LD A, [r16]
        0x0a | 0x1a | 0x2a | 0x3a => {
            // Load r16
            let r16 = match code {
                0x0a => cpu.reg.bc(),
                0x1a => cpu.reg.de(),
                0x2a | 0x3a => cpu.reg.hl(),
                code => return Err(Error::Opcode(code)),
            }
            .load();
            // Read r16
            let op2 = cpu.read(r16);
            // Proceed
            Ok(Some(Ld::Byte(op2).into()))
        }
        // LD r8, n8
        0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x3e => {
            // Fetch n8 <- [PC++]
            let op2 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Ld::Byte(op2).into()))
        }
        // LD [HL], n8
        0x36 => {
            // Fetch n8 <- [PC++]
            let op2 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Ld::Write(op2).into()))
        }
        // LD r8, [HL]
        0x46 | 0x4e | 0x56 | 0x5e | 0x66 | 0x6e | 0x7e => {
            // Load [HL]
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(Ld::Byte(op2).into()))
        }
        // HALT (unexpected opcode)
        0x76 => Err(Error::Opcode(code)),
        // LD [HL], r8
        0x70..=0x77 => {
            // Load HL
            let addr = cpu.reg.hl().load();
            // Prepare op2
            let op2 = help::get_op8(cpu, code & 0x07);
            // Write op2
            cpu.write(addr, op2);
            // Proceed
            Ok(Some(Ld::Done.into()))
        }
        // LD r8, r8
        0x40..=0x7f => {
            // Prepare op2
            let op2 = help::get_op8(cpu, code & 0x07);
            // Continue
            byte(code, cpu, op2)
        }
        // LD [a16], A
        // LD A, [a16]
        0xea | 0xfa => {
            // Continue
            fetch0(code, cpu)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn update(code: Byte, cpu: &mut Cpu) -> Return {
    // Load HL
    let mut hl = cpu.reg.hl_mut();
    let r16 = hl.load();
    // Update HL
    let res = match code {
        // INC HL
        0x22 | 0x2a => r16.wrapping_add(1),
        // DEC HL
        0x32 | 0x3a => r16.wrapping_sub(1),
        code => return Err(Error::Opcode(code)),
    };
    // Store HL <- HL{+, -}
    hl.store(res);

    // Continue
    done(code, cpu)
}

fn write(code: Byte, cpu: &mut Cpu, op2: Byte) -> Return {
    // Check opcode
    if code != 0x36 {
        return Err(Error::Opcode(code));
    }

    // Write [HL]
    cpu.writebyte(op2);

    // Proceed
    Ok(Some(Ld::Done.into()))
}

fn byte(code: Byte, cpu: &mut Cpu, op2: Byte) -> Return {
    // Store r8 <- {r8, n8, [HL]}
    let op1 = match code {
        0x06 | 0x40..=0x47 => &mut cpu.reg.b,
        0x0e | 0x48..=0x4f => &mut cpu.reg.c,
        0x16 | 0x50..=0x57 => &mut cpu.reg.d,
        0x1e | 0x58..=0x5f => &mut cpu.reg.e,
        0x26 | 0x60..=0x67 => &mut cpu.reg.h,
        0x2e | 0x68..=0x6f => &mut cpu.reg.l,
        0x0a | 0x1a | 0x2a | 0x3a | 0x3e | 0x78..=0x7f | 0xf2 => &mut cpu.reg.a,
        code => return Err(Error::Opcode(code)),
    };
    op1.store(op2);

    // Continue
    match code {
        0x2a | 0x3a => update(code, cpu),
        _ => done(code, cpu),
    }
}

fn fetch0(_: Byte, cpu: &mut Cpu) -> Return {
    // Fetch LSB
    let lsb = cpu.fetchbyte();
    // Proceed
    Ok(Some(Ld::Fetch1(lsb).into()))
}

fn fetch1(_: Byte, cpu: &mut Cpu, lsb: Byte) -> Return {
    // Fetch LSB
    let msb = cpu.fetchbyte();
    // Combine bytes
    let a16 = u16::from_le_bytes([lsb, msb]);

    // Proceed
    Ok(Some(Ld::Word(a16).into()))
}

fn word(code: Byte, cpu: &mut Cpu, word: u16) -> Return {
    match code {
        0xea => {
            // Execute LD [a16], A
            let op2 = cpu.reg.a.load();
            cpu.write(word, op2);
        }
        0xfa => {
            // Execute LD A, [a16]
            let op2 = cpu.read(word);
            cpu.reg.a.store(op2);
        }
        code => return Err(Error::Opcode(code)),
    }

    // Proceed
    Ok(Some(Ld::Done.into()))
}

fn done(_: Byte, _: &mut Cpu) -> Return {
    // Finish
    Ok(None)
}
