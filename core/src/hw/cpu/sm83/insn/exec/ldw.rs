use super::*;

pub const fn default() -> Operation {
    Operation::Ldw(Ldw::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Ldw {
    #[default]
    Fetch,
    Delay(u16),
    Execute(u16),
    DelayA0x08(u16),
    Execute0x08(u16),
    DelayB0x08,
    Done0x08,
    Execute0xF8(u8),
    Done0xF8,
}

impl Execute for Ldw {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Delay(op2) => delay(code, cpu, op2),
            Self::Execute(op2) => execute(code, cpu, op2),
            Self::DelayA0x08(a16) => delay_a_0x08(code, cpu, a16),
            Self::Execute0x08(a16) => execute_0x08(code, cpu, a16),
            Self::DelayB0x08 => delay_b_0x08(code, cpu),
            Self::Done0x08 => done_0x08(code, cpu),
            Self::Execute0xF8(e8) => execute_0xf8(code, cpu, e8),
            Self::Done0xF8 => done_0xf8(code, cpu),
        }
    }
}

impl From<Ldw> for Operation {
    fn from(value: Ldw) -> Self {
        Self::Ldw(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        // LD r16, n16
        0x01 | 0x11 | 0x21 | 0x31 => {
            // Fetch n16
            let n16 = cpu.fetchword();
            // Proceed
            Ok(Some(Ldw::Delay(n16).into()))
        }
        // LD [a16], SP
        0x08 => {
            // Fetch a16
            let a16 = cpu.fetchword();
            // Proceed
            Ok(Some(Ldw::DelayA0x08(a16).into()))
        }
        // LD HL, SP + e8
        0xf8 => {
            // Fetch e8
            let e8 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Ldw::Execute0xF8(e8).into()))
        }
        // LD SP, HL
        0xf9 => {
            // Load HL
            let r16 = (cpu.file.hl.load)(&cpu.file);
            // Proceed
            Ok(Some(Ldw::Execute(r16).into()))
        }
        code => Err(Error::Opcode(code)),
    }
}

fn delay(_: u8, _: &mut Cpu, op2: u16) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Ldw::Execute(op2).into()))
}

fn execute(code: u8, cpu: &mut Cpu, op2: u16) -> Return {
    // Execute LDW
    match code {
        0x01 => (cpu.file.bc.store)(&mut cpu.file, op2),
        0x11 => (cpu.file.de.store)(&mut cpu.file, op2),
        0x21 => (cpu.file.hl.store)(&mut cpu.file, op2),
        0x31 | 0xf9 => *cpu.file.sp = op2,
        code => return Err(Error::Opcode(code)),
    }

    // Finish
    Ok(None)
}

fn delay_a_0x08(_: u8, _: &mut Cpu, a16: u16) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Ldw::Execute0x08(a16).into()))
}

fn execute_0x08(_: u8, cpu: &mut Cpu, mut a16: u16) -> Return {
    // Read SP
    let sp = cpu.file.sp.load().to_le_bytes();
    // Write a16
    cpu.write(a16, sp[0]);
    a16 = a16.wrapping_add(1);
    cpu.write(a16, sp[1]);

    // Proceed
    Ok(Some(Ldw::DelayB0x08.into()))
}

fn delay_b_0x08(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Ldw::Done0x08.into()))
}

fn done_0x08(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}

fn execute_0xf8(_: u8, cpu: &mut Cpu, e8: u8) -> Return {
    // LD HL, SP + e8
    let sp = *cpu.file.sp;
    let e16 = e8 as i8 as u16;
    let res = sp.wrapping_add(e16);
    let hl = cpu.file.hl;
    hl.store(&mut cpu.file, res);

    // Set flags
    let flags = &mut cpu.file.f.load();
    Flag::Z.set(flags, false);
    Flag::N.set(flags, false);
    Flag::H.set(flags, 0x000f < (sp & 0x000f) + (e16 & 0x000f));
    Flag::C.set(flags, 0x00ff < (sp & 0x00ff) + (e16 & 0x00ff));
    cpu.file.f.store(*flags);

    // Proceed
    Ok(Some(Ldw::Done0xF8.into()))
}

fn done_0xf8(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
