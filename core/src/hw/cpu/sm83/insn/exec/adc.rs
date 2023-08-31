use super::*;

pub const fn default() -> Operation {
    Operation::Adc(Adc::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Adc {
    #[default]
    Fetch,
    Execute(u8),
}

impl Execute for Adc {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Execute(op2) => execute(code, cpu, op2),
        }
    }
}

impl From<Adc> for Operation {
    fn from(op: Adc) -> Self {
        Self::Adc(op)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x8e => {
            // Read [HL]
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(Adc::Execute(op2).into()))
        }
        0xce => {
            // Fetch n8
            let op2 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Adc::Execute(op2).into()))
        }
        0x88..=0x8f => {
            // Prepare op2
            let op2 = helpers::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op2)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(_: u8, cpu: &mut Cpu, op2: u8) -> Return {
    // Execute ADC
    let acc = cpu.file.a.load();
    let flags = &cpu.file.f.load();
    let cin = Flag::C.get(flags) as u8;
    let (res, carry0) = acc.overflowing_add(op2);
    let (res, carry1) = res.overflowing_add(cin);
    cpu.file.a.store(res);

    // Set flags
    let flags = &mut cpu.file.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, 0x0f < (acc & 0x0f) + (op2 & 0x0f) + cin);
    Flag::C.set(flags, carry0 | carry1);
    cpu.file.f.store(*flags);

    // Finish
    Ok(None)
}
