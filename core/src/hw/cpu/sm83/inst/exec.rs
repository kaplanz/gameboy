use std::ops::{BitAnd, BitOr, BitXor};

use enumflag::Enumflag;
use remus::Device;

use super::{helpers, Cpu, Flag, Ime, Instruction, Status};

pub mod adc {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x8e => {
                // Read (HL)
                let op2 = cpu.readbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xce => {
                // Fetch d8
                let op2 = cpu.fetchbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0x88..=0x8f => {
                // Prepare op2
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op2);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute ADC
        let acc = *cpu.regs.a;
        let op2 = inst.stack.pop().unwrap();
        let cin = Flag::C.get(&*cpu.regs.f) as u8;
        let (res, carry0) = acc.overflowing_add(op2);
        let (res, carry1) = res.overflowing_add(cin);
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, 0x0f < (acc & 0x0f) + (op2 & 0x0f) + cin);
        Flag::C.set(flags, carry0 | carry1);

        // Finish
        None
    }
}

pub mod add {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x86 => {
                // Read (HL)
                let op2 = cpu.readbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xc6 => {
                // Fetch d8
                let op2 = cpu.fetchbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0x80..=0x87 => {
                // Prepare op2
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op2);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute ADD
        let acc = *cpu.regs.a;
        let op2 = inst.stack.pop().unwrap();
        let (res, carry) = acc.overflowing_add(op2);
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, 0x0f < (acc & 0x0f) + (op2 & 0x0f));
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

pub mod addw {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x09 | 0x19 | 0x29 | 0x39 => {
                let op1 = cpu.regs.hl.get(&cpu.regs);
                let op2 = match inst.opcode {
                    0x09 => cpu.regs.bc.get(&cpu.regs),
                    0x19 => cpu.regs.de.get(&cpu.regs),
                    0x29 => cpu.regs.hl.get(&cpu.regs),
                    0x39 => *cpu.regs.sp,
                    _ => panic!("Illegal instruction."),
                };
                inst.stack.extend(op1.to_le_bytes());
                inst.stack.extend(op2.to_le_bytes());
                inst.exec = done;
                Some(inst)
            }
            0xe8 => {
                // Fetch r8
                let r8 = cpu.fetchbyte();
                inst.stack.push(r8);
                // Proceed
                inst.exec = exec_0xe8_1;
                Some(inst)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute ADDW
        let op1 = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        let op2 = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        let (res, carry) = op1.overflowing_add(op2);
        let hl = cpu.regs.hl;
        hl.set(&mut cpu.regs, res);

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::N.set(flags, false);
        Flag::H.set(flags, 0x0fff < (op1 & 0x0fff) + (op2 & 0x0fff));
        Flag::C.set(flags, carry);

        // Finish
        None
    }

    pub fn exec_0xe8_1(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        inst.exec = exec_0xe8_2;
        Some(inst)
    }

    pub fn exec_0xe8_2(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        inst.exec = done_0xe8;
        Some(inst)
    }

    pub fn done_0xe8(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute ADDW
        let op1 = *cpu.regs.sp;
        let op2 = inst.stack.pop().unwrap() as i8 as u16;
        let res = op1.wrapping_add(op2);
        *cpu.regs.sp = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, false);
        Flag::N.set(flags, false);
        Flag::H.set(flags, 0x000f < (op1 & 0x000f) + (op2 & 0x000f));
        Flag::C.set(flags, 0x00ff < (op1 & 0x00ff) + (op2 & 0x00ff));

        // Finish
        None
    }
}

pub mod and {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0xa6 => {
                // Read (HL)
                let op2 = cpu.readbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xe6 => {
                // Fetch d8
                let op2 = cpu.fetchbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xa0..=0xa7 => {
                // Prepare op2
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op2);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute AND
        let acc = *cpu.regs.a;
        let op2 = inst.stack.pop().unwrap();
        let res = acc.bitand(op2);
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, true);
        Flag::C.set(flags, false);

        // Finish
        None
    }
}

pub mod bit {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x46 | 0x4e | 0x56 | 0x5e | 0x66 | 0x6e | 0x76 | 0x7e => {
                // Read (HL)
                let op2 = cpu.readbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0x40..=0x7f => {
                // Prepare op2
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op2);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute BIT
        let op1 = (inst.opcode & 0x38) >> 3;
        let op2 = inst.stack.pop().unwrap();
        let res = (0b1 << op1) & op2;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, true);

        // Finish
        None
    }
}

pub mod call {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0xc4 | 0xcc | 0xcd | 0xd4 | 0xdc => {
                // Fetch u16
                let op1 = cpu.fetchword();
                inst.stack.extend(op1.to_le_bytes());
                // Proceed
                inst.exec = evaluate;
                Some(inst)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn evaluate(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to fetch a u16.

        // Evaluate condition
        let flags = &mut cpu.regs.f;
        let cond = match inst.opcode {
            0xc4 => !Flag::Z.get(flags),
            0xcc => Flag::Z.get(flags),
            0xcd => true,
            0xd4 => !Flag::C.get(flags),
            0xdc => Flag::C.get(flags),
            _ => panic!("Illegal instruction."),
        };
        inst.stack.push(cond as u8);

        // Proceed
        inst.exec = check;
        Some(inst)
    }

    pub fn check(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Execute CALL
        let cond = inst.stack.pop().unwrap() != 0;
        if cond {
            // Proceed
            inst.exec = push;
            Some(inst)
        } else {
            // Finish
            None
        }
    }

    pub fn push(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Push SP
        cpu.pushword(*cpu.regs.pc);

        // Proceed
        inst.exec = delay;
        Some(inst)
    }

    pub fn delay(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to push a u16.

        // Proceed
        inst.exec = jump;
        Some(inst)
    }

    pub fn jump(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let op1 = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        *cpu.regs.pc = op1;

        // Finish
        None
    }
}

pub mod ccf {
    use super::*;

    pub fn start(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0x3f {
            panic!("Illegal instruction.");
        }

        // Execute CCF
        let flags = &mut cpu.regs.f;
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        let carry = Flag::C.get(flags);
        Flag::C.set(flags, !carry);

        // Finish
        None
    }
}

pub mod cp {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0xbe => {
                let op2 = cpu.readbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xfe => {
                let op2 = cpu.fetchbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xb8..=0xbf => {
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op2);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute CP
        let acc = *cpu.regs.a;
        let op2 = inst.stack.pop().unwrap();
        let (res, carry) = acc.overflowing_sub(op2);

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, true);
        Flag::H.set(flags, (op2 & 0x0f) > (acc & 0x0f));
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

pub mod cpl {
    use super::*;

    pub fn start(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0x2f {
            panic!("Illegal instruction.");
        }

        // Execute CPL
        let acc = *cpu.regs.a;
        let res = !acc;
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::N.set(flags, true);
        Flag::H.set(flags, true);

        // Finish
        None
    }
}

pub mod daa {
    use super::*;

    pub fn start(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0x27 {
            panic!("Illegal instruction.");
        }

        // Execute DAA
        let didsub = Flag::N.get(&cpu.regs.f);
        let hcarry = Flag::H.get(&cpu.regs.f);
        let mut carry = Flag::C.get(&cpu.regs.f);
        let mut adj = 0i8;
        let acc = *cpu.regs.a;
        if hcarry || (!didsub && (acc & 0x0f) > 0x09) {
            adj |= 0x06;
        }
        if carry || (!didsub && acc > 0x99) {
            adj |= 0x60;
            carry = true;
        }
        adj = if !didsub { adj } else { -adj };
        let res = (acc as i8).wrapping_add(adj) as u8;
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

pub mod dec {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x35 => {
                let op1 = cpu.readbyte();
                inst.stack.push(op1);
                inst.exec = done;
                Some(inst)
            }
            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x3d => {
                let op1 = match inst.opcode {
                    0x05 => *cpu.regs.b,
                    0x0d => *cpu.regs.c,
                    0x15 => *cpu.regs.d,
                    0x1d => *cpu.regs.e,
                    0x25 => *cpu.regs.h,
                    0x2d => *cpu.regs.l,
                    0x3d => *cpu.regs.a,
                    _ => panic!("Illegal instruction."),
                };
                inst.stack.push(op1);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute DEC
        let op1 = inst.stack.pop().unwrap();
        let res = op1.wrapping_sub(1);

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, true);
        Flag::H.set(flags, op1 & 0x0f == 0);

        // Check opcode
        match inst.opcode {
            0x35 => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x3d => {
                // Write X
                *match inst.opcode {
                    0x05 => &mut *cpu.regs.b,
                    0x0d => &mut *cpu.regs.c,
                    0x15 => &mut *cpu.regs.d,
                    0x1d => &mut *cpu.regs.e,
                    0x25 => &mut *cpu.regs.h,
                    0x2d => &mut *cpu.regs.l,
                    0x3d => &mut *cpu.regs.a,
                    _ => panic!("Illegal instruction."),
                } = res;
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

pub mod decw {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        let op1 = match inst.opcode {
            0x0b => cpu.regs.bc.get(&cpu.regs),
            0x1b => cpu.regs.de.get(&cpu.regs),
            0x2b => cpu.regs.hl.get(&cpu.regs),
            0x3b => *cpu.regs.sp,
            _ => panic!("Illegal instruction."),
        };
        inst.stack.extend(op1.to_le_bytes());
        inst.exec = done;

        Some(inst)
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute DECW
        let op1 = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        let res = op1.wrapping_sub(1);
        match inst.opcode {
            0x0b => {
                let bc = cpu.regs.bc;
                bc.set(&mut cpu.regs, res);
            }
            0x1b => {
                let de = cpu.regs.de;
                de.set(&mut cpu.regs, res);
            }
            0x2b => {
                let hl = cpu.regs.hl;
                hl.set(&mut cpu.regs, res);
            }
            0x3b => *cpu.regs.sp = res,
            _ => panic!("Illegal instruction."),
        }

        // Finish
        None
    }
}

pub mod di {
    use super::*;

    pub fn start(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0xf3 {
            panic!("Illegal instruction.");
        }

        // Execute DI
        cpu.ime = Ime::Disabled;

        // Finish
        None
    }
}

pub mod ei {
    use super::*;

    pub fn start(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0xfb {
            panic!("Illegal instruction.");
        }

        // Execute EI
        cpu.ime = Ime::WillEnable;

        // Finish
        None
    }
}

pub mod halt {
    use super::*;

    pub fn start(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0x76 {
            panic!("Illegal instruction.");
        }

        // Perform HALT bug
        if !cpu.ime.enabled() && cpu.pic.borrow().int().is_some() {
            cpu.halt_bug = true;
        } else {
            // Execute HALT
            cpu.status = Status::Halted;
        }

        // Finish
        None
    }
}

pub mod inc {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x34 => {
                let op1 = cpu.readbyte();
                inst.stack.push(op1);
                inst.exec = done;
                Some(inst)
            }
            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x3c => {
                let op1 = match inst.opcode {
                    0x04 => *cpu.regs.b,
                    0x0c => *cpu.regs.c,
                    0x14 => *cpu.regs.d,
                    0x1c => *cpu.regs.e,
                    0x24 => *cpu.regs.h,
                    0x2c => *cpu.regs.l,
                    0x3c => *cpu.regs.a,
                    _ => panic!("Illegal instruction."),
                };
                inst.stack.push(op1);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute INC
        let op1 = inst.stack.pop().unwrap();
        let res = op1.wrapping_add(1);

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, res & 0x0f == 0);

        // Check opcode
        match inst.opcode {
            0x34 => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x3c => {
                // Write X
                *match inst.opcode {
                    0x04 => &mut *cpu.regs.b,
                    0x0c => &mut *cpu.regs.c,
                    0x14 => &mut *cpu.regs.d,
                    0x1c => &mut *cpu.regs.e,
                    0x24 => &mut *cpu.regs.h,
                    0x2c => &mut *cpu.regs.l,
                    0x3c => &mut *cpu.regs.a,
                    _ => panic!("Illegal instruction."),
                } = res;
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

pub mod incw {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        let op1 = match inst.opcode {
            0x03 => cpu.regs.bc.get(&cpu.regs),
            0x13 => cpu.regs.de.get(&cpu.regs),
            0x23 => cpu.regs.hl.get(&cpu.regs),
            0x33 => *cpu.regs.sp,
            _ => panic!("Illegal instruction."),
        };
        inst.stack.extend(op1.to_le_bytes());
        inst.exec = done;

        Some(inst)
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute INCW
        let op1 = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        let res = op1.wrapping_add(1);
        match inst.opcode {
            0x03 => {
                let bc = cpu.regs.bc;
                bc.set(&mut cpu.regs, res);
            }
            0x13 => {
                let de = cpu.regs.de;
                de.set(&mut cpu.regs, res);
            }
            0x23 => {
                let hl = cpu.regs.hl;
                hl.set(&mut cpu.regs, res);
            }
            0x33 => *cpu.regs.sp = res,
            _ => panic!("Illegal instruction."),
        }

        // Finish
        None
    }
}

pub mod int {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Disable interrupts
        cpu.ime = Ime::Disabled;

        inst.exec = nop;
        Some(inst)
    }

    pub fn nop(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Execute NOP
        inst.exec = push;
        Some(inst)
    }

    pub fn push(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Push SP
        cpu.pushword(*cpu.regs.pc);

        // Proceed
        inst.exec = delay;
        Some(inst)
    }

    pub fn delay(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to push a u16.

        // Proceed
        inst.exec = jump;
        Some(inst)
    }

    pub fn jump(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let addr = inst.stack.pop().unwrap() as u16;
        *cpu.regs.pc = addr;

        // Finish
        None
    }
}

pub mod jp {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0xc2 | 0xc3 | 0xca | 0xd2 | 0xda => {
                // Fetch u16
                let op1 = cpu.fetchword();
                inst.stack.extend(op1.to_le_bytes());
                // Proceed
                inst.exec = evaluate;
                Some(inst)
            }
            0xe9 => {
                // Read HL
                let op1 = cpu.regs.hl.get(&cpu.regs);
                inst.stack.extend(op1.to_le_bytes());
                // Continue
                jump(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn evaluate(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to fetch a u16.

        // Evaluate condition
        let flags = &mut cpu.regs.f;
        let cond = match inst.opcode {
            0xc2 => !Flag::Z.get(flags),
            0xc3 => true,
            0xca => Flag::Z.get(flags),
            0xd2 => !Flag::C.get(flags),
            0xda => Flag::C.get(flags),
            _ => panic!("Illegal instruction."),
        };
        inst.stack.push(cond as u8);

        // Proceed
        inst.exec = check;
        Some(inst)
    }

    pub fn check(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Execute JP
        let cond = inst.stack.pop().unwrap() != 0;
        if cond {
            // Proceed
            inst.exec = jump;
            Some(inst)
        } else {
            // Finish
            None
        }
    }

    pub fn jump(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let op1 = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        *cpu.regs.pc = op1;

        // Finish
        None
    }
}

pub mod jr {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
                // Fetch u8
                let op1 = cpu.fetchbyte();
                inst.stack.push(op1);
                // Continue
                evaluate(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn evaluate(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Evaluate condition
        let flags = &mut cpu.regs.f;
        let cond = match inst.opcode {
            0x18 => true,
            0x20 => !Flag::Z.get(flags),
            0x28 => Flag::Z.get(flags),
            0x30 => !Flag::C.get(flags),
            0x38 => Flag::C.get(flags),
            _ => panic!("Illegal instruction."),
        };
        inst.stack.push(cond as u8);

        // Proceed
        inst.exec = check;
        Some(inst)
    }

    pub fn check(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Execute JR
        let cond = inst.stack.pop().unwrap() != 0;
        if cond {
            // Proceed
            inst.exec = jump;
            Some(inst)
        } else {
            // Finish
            None
        }
    }

    pub fn jump(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let pc = *cpu.regs.pc as i16;
        let op1 = inst.stack.pop().unwrap() as i8 as i16;
        let res = pc.wrapping_add(op1) as u16;
        *cpu.regs.pc = res;

        // Finish
        None
    }
}

pub mod ld {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x02 | 0x12 | 0x22 | 0x32 => {
                // Execute LD (XY), A
                let addr = match inst.opcode {
                    0x02 => cpu.regs.bc,
                    0x12 => cpu.regs.de,
                    0x22 | 0x32 => cpu.regs.hl,
                    _ => panic!("Illegal instruction."),
                }
                .get(&cpu.regs);
                inst.stack.extend(addr.to_le_bytes());
                let op2 = *cpu.regs.a;
                cpu.bus.borrow_mut().write(addr as usize, op2);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x0a | 0x1a | 0x2a | 0x3a => {
                // Load (XY)
                let addr = match inst.opcode {
                    0x0a => cpu.regs.bc,
                    0x1a => cpu.regs.de,
                    0x2a | 0x3a => cpu.regs.hl,
                    _ => panic!("Illegal instruction."),
                }
                .get(&cpu.regs);
                inst.stack.extend(addr.to_le_bytes());
                let op2 = cpu.bus.borrow().read(addr as usize);
                inst.stack.push(op2);
                // Proceed
                inst.exec = execute;
                Some(inst)
            }
            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x3e => {
                // Fetch d8
                let op2 = cpu.fetchbyte();
                inst.stack.push(op2);
                // Proceed
                inst.exec = execute;
                Some(inst)
            }
            0x36 => {
                // Fetch d8
                let op2 = cpu.fetchbyte();
                inst.stack.push(op2);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x46 | 0x4e | 0x56 | 0x5e | 0x66 | 0x6e | 0x7e => {
                // Load (HL)
                let op2 = cpu.readbyte();
                inst.stack.push(op2);
                // Proceed
                inst.exec = execute;
                Some(inst)
            }
            0x76 => panic!("Illegal instruction."),
            0x70..=0x77 => {
                // Execute LD (HL), X
                let addr = cpu.regs.hl.get(&cpu.regs);
                inst.stack.extend(addr.to_le_bytes());
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                cpu.bus.borrow_mut().write(addr as usize, op2);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x40..=0x7f => {
                // Execute LD X, Y
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op2);
                // Continue
                execute(inst, cpu)
            }
            0xea | 0xfa => {
                // Fetch a16
                let addr = cpu.fetchword();
                inst.stack.extend(addr.to_le_bytes());
                // Proceed
                inst.exec = delay_a16;
                Some(inst)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: Since the memory bus is busy with the write, we must fetch the
        //       next instruction this cycle instead.

        // Perform inc/dec on HL
        match inst.opcode {
            0x02 | 0x12 => {
                // Continue
                done(inst, cpu)
            }
            0x22 | 0x32 => {
                // Continue
                inc_dec_hl(inst, cpu)
            }
            0x36 => {
                // Execute (HL), d8
                let op2 = inst.stack.pop().unwrap();
                cpu.writebyte(op2);
                // Proceed
                inst.exec = done;
                Some(inst)
            }
            0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x77 => {
                // Continue
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn execute(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute LD X, {Y, d8, (HL)}
        let op1 = match inst.opcode {
            0x06 | 0x40..=0x47 => &mut *cpu.regs.b,
            0x0e | 0x48..=0x4f => &mut *cpu.regs.c,
            0x16 | 0x50..=0x57 => &mut *cpu.regs.d,
            0x1e | 0x58..=0x5f => &mut *cpu.regs.e,
            0x26 | 0x60..=0x67 => &mut *cpu.regs.h,
            0x2e | 0x68..=0x6f => &mut *cpu.regs.l,
            0x0a | 0x1a | 0x2a | 0x3a | 0x3e | 0x78..=0x7f | 0xf2 => &mut *cpu.regs.a,
            _ => panic!("Illegal instruction."),
        };
        let op2 = inst.stack.pop().unwrap();
        *op1 = op2;

        // Continue
        match inst.opcode {
            0x2a | 0x3a => inc_dec_hl(inst, cpu),
            _ => done(inst, cpu),
        }
    }

    pub fn done(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Finish
        None
    }

    pub fn inc_dec_hl(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform inc/dec on HL
        let addr = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        let hl = cpu.regs.hl;
        match inst.opcode {
            0x22 | 0x2a => hl.set(&mut cpu.regs, addr.wrapping_add(1)),
            0x32 | 0x3a => hl.set(&mut cpu.regs, addr.wrapping_sub(1)),
            _ => panic!("Illegal instruction."),
        }

        // Continue
        done(inst, cpu)
    }

    pub fn delay_a16(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to read an a16.

        // Proceed
        inst.exec = delay_rw;
        Some(inst)
    }

    pub fn delay_rw(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        let addr = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        match inst.opcode {
            0xea => {
                // Execute LD (a16), A
                let op2 = *cpu.regs.a;
                cpu.bus.borrow_mut().write(addr as usize, op2);
            }
            0xfa => {
                // Execute LD A, (a16)
                let op2 = cpu.bus.borrow_mut().read(addr as usize);
                *cpu.regs.a = op2;
            }
            _ => panic!("Illegal instruction."),
        }

        // Proceed
        inst.exec = done;
        Some(inst)
    }
}

pub mod ldw {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x01 | 0x11 | 0x21 | 0x31 => {
                // Fetch d16
                let op2 = cpu.fetchword();
                inst.stack.extend(op2.to_le_bytes());
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x08 => {
                // Fetch a16
                let addr = cpu.fetchword();
                inst.stack.extend(addr.to_le_bytes());
                // Proceed
                inst.exec = delay_0x08_1;
                Some(inst)
            }
            0xf8 => {
                // Fetch r8
                let r8 = cpu.fetchbyte();
                inst.stack.push(r8);
                // Proceed
                inst.exec = delay_0xf8;
                Some(inst)
            }
            0xf9 => {
                // Read HL
                let op2 = (cpu.regs.hl.get)(&cpu.regs);
                inst.stack.extend(op2.to_le_bytes());
                // Proceed
                inst.exec = done;
                Some(inst)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to read a d16.

        // Proceed
        inst.exec = done;
        Some(inst)
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute LDW
        let op2 = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        match inst.opcode {
            0x01 => (cpu.regs.bc.set)(&mut cpu.regs, op2),
            0x11 => (cpu.regs.de.set)(&mut cpu.regs, op2),
            0x21 => (cpu.regs.hl.set)(&mut cpu.regs, op2),
            0x31 | 0xf9 => *cpu.regs.sp = op2,
            _ => panic!("Illegal instruction."),
        }

        // Finish
        None
    }

    pub fn delay_0x08_1(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to read an a16.

        // Proceed
        inst.exec = delay_0x08_2;
        Some(inst)
    }

    pub fn delay_0x08_2(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Write the SP into the a16
        let addr = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        let sp = &cpu.regs.sp;
        cpu.bus.borrow_mut().write(addr as usize, sp.read(0));
        let addr = addr.wrapping_add(1);
        cpu.bus.borrow_mut().write(addr as usize, sp.read(1));

        // Proceed
        inst.exec = delay_0x08_3;
        Some(inst)
    }

    pub fn delay_0x08_3(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to read an a16.

        // Proceed
        inst.exec = done_0x08;
        Some(inst)
    }

    pub fn done_0x08(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the delay needed to fetch the next instruction.

        // Finish
        None
    }

    pub fn delay_0xf8(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute LD HL, SP + r8
        let sp = *cpu.regs.sp;
        let r8 = inst.stack.pop().unwrap() as i8 as u16;
        let res = sp.wrapping_add(r8);
        let hl = cpu.regs.hl;
        hl.set(&mut cpu.regs, res);

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, false);
        Flag::N.set(flags, false);
        Flag::H.set(flags, 0x000f < (sp & 0x000f) + (r8 & 0x000f));
        Flag::C.set(flags, 0x00ff < (sp & 0x00ff) + (r8 & 0x00ff));

        // Proceed
        inst.exec = done_0xf8;
        Some(inst)
    }

    pub fn done_0xf8(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the delay needed to fetch the next instruction.

        // Finish
        None
    }
}

pub mod ldh {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0xe0 | 0xf0 => {
                // Fetch a8
                let a8 = cpu.fetchbyte();
                inst.stack.push(a8);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0xe2 | 0xf2 => {
                // Read C
                let a8 = *cpu.regs.c;
                inst.stack.push(a8);
                // Proceed
                delay(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Calculate absolute address from relative
        let addr = 0xff00 | inst.stack.pop().unwrap() as u16;

        // Perform a read/write to the address
        match inst.opcode {
            0xe0 | 0xe2 => {
                // Execute LD (a8|C), A
                let op2 = *cpu.regs.a;
                cpu.bus.borrow_mut().write(addr as usize, op2);
            }
            0xf0 | 0xf2 => {
                // Execute LD A, (a8|C)
                *cpu.regs.a = cpu.bus.borrow().read(addr as usize);
            }
            _ => panic!("Illegal instruction."),
        }

        // Proceed
        inst.exec = done;
        Some(inst)
    }

    pub fn done(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This either represents the cycle needed to perform the load, or
        //       a delay cycle due to a previous write (as we fetch the next
        //       instruction).

        // Finish
        None
    }
}

pub mod nop {
    use super::*;

    pub fn start(inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0x00 {
            panic!("Illegal instruction.");
        }

        // Finish
        None
    }
}

pub mod or {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0xb6 => {
                let op2 = cpu.readbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xf6 => {
                let op2 = cpu.fetchbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xb0..=0xb7 => {
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op2);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute OR
        let acc = *cpu.regs.a;
        let op2 = inst.stack.pop().unwrap();
        let res = acc.bitor(op2);
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, false);

        // Finish
        None
    }
}

pub mod pop {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0xc1 | 0xd1 | 0xe1 | 0xf1 => (),
            _ => panic!("Illegal instruction."),
        }

        // Pop u16
        let mut word = cpu.popword();
        if inst.opcode == 0xf1 {
            word &= 0xfff0; // lower 4 bits of F cannot be changed
        }
        inst.stack.extend(word.to_le_bytes());

        // Proceed
        inst.exec = delay;
        Some(inst)
    }

    pub fn delay(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to pop a u16.

        // Proceed
        inst.exec = done;
        Some(inst)
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform pop
        let word = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        match inst.opcode {
            0xc1 => cpu.regs.bc,
            0xd1 => cpu.regs.de,
            0xe1 => cpu.regs.hl,
            0xf1 => cpu.regs.af,
            _ => panic!("Illegal instruction."),
        }
        .set(&mut cpu.regs, word);

        // Finish
        None
    }
}

pub mod prefix {
    use super::*;

    pub fn start(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0xcb {
            panic!("Illegal instruction.");
        }

        // Fetch prefix instruction
        let opcode = cpu.fetchbyte();
        let inst = Instruction::prefix(opcode);
        Some(inst)
    }
}

pub mod push {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        let word = match inst.opcode {
            0xc5 => cpu.regs.bc,
            0xd5 => cpu.regs.de,
            0xe5 => cpu.regs.hl,
            0xf5 => cpu.regs.af,
            _ => panic!("Illegal instruction."),
        }
        .get(&cpu.regs);
        inst.stack.extend(word.to_le_bytes());

        // Proceed
        inst.exec = push;
        Some(inst)
    }

    pub fn push(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform push
        let word = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        cpu.pushword(word);

        // Proceed
        inst.exec = delay;
        Some(inst)
    }

    pub fn delay(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to push a u16.

        // Proceed
        inst.exec = done;
        Some(inst)
    }

    pub fn done(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the delay needed to fetch the next instruction.

        // Finish
        None
    }
}

pub mod res {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x86 | 0x8e | 0x96 | 0x9e | 0xa6 | 0xae | 0xb6 | 0xbe => {
                let op2 = cpu.readbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0x80..=0xbf => {
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op2);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute RES
        let op1 = (inst.opcode & 0x38) >> 3;
        let op2 = inst.stack.pop().unwrap();
        let mask = !(0b1 << op1);
        let res = mask & op2;

        // Check opcode
        match inst.opcode {
            0x86 | 0x8e | 0x96 | 0x9e | 0xa6 | 0xae | 0xb6 | 0xbe => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x80..=0xbf => {
                // Write X
                helpers::set_op8(cpu, inst.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

pub mod ret {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Evaluate condition
        let flags = &mut cpu.regs.f;
        let cond = match inst.opcode {
            0xc0 => !Flag::Z.get(flags),
            0xc8 => Flag::Z.get(flags),
            0xc9 => true,
            0xd0 => !Flag::C.get(flags),
            0xd8 => Flag::C.get(flags),
            _ => panic!("Illegal instruction."),
        };
        inst.stack.push(cond as u8);

        if inst.opcode == 0xc9 {
            // Continue
            check(inst, cpu)
        } else {
            // Proceed
            inst.exec = check;
            Some(inst)
        }
    }

    pub fn check(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Execute RET
        let cond = inst.stack.pop().unwrap() != 0;
        if cond {
            // Proceed
            inst.exec = pop;
            Some(inst)
        } else {
            // Finish
            None
        }
    }

    pub fn pop(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Pop PC
        let pc = cpu.popword();
        inst.stack.extend(pc.to_le_bytes());

        // Proceed
        inst.exec = delay;
        Some(inst)
    }

    pub fn delay(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to pop a u16.

        // Proceed
        inst.exec = jump;
        Some(inst)
    }

    pub fn jump(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let pc = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        *cpu.regs.pc = pc;

        // Finish
        None
    }
}

pub mod reti {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0xd9 {
            panic!("Illegal instruction.");
        }

        // Pop PC
        let pc = cpu.popword();
        inst.stack.extend(pc.to_le_bytes());

        // Proceed
        inst.exec = delay;
        Some(inst)
    }

    pub fn delay(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to pop a u16.

        // Proceed
        inst.exec = jump;
        Some(inst)
    }

    pub fn jump(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let pc = u16::from_le_bytes(
            inst.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        *cpu.regs.pc = pc;

        // Proceed
        inst.exec = done;
        Some(inst)
    }

    pub fn done(_: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Enable interrupts
        cpu.ime = Ime::WillEnable;

        // Finish
        None
    }
}

pub mod rl {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x16 => {
                let op1 = cpu.readbyte();
                inst.stack.push(op1);
                inst.exec = done;
                Some(inst)
            }
            0x10..=0x17 => {
                let op1 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op1);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute RL
        let op1 = inst.stack.pop().unwrap();
        let flags = &mut cpu.regs.f;
        let cin = Flag::C.get(flags);
        let carry = op1 & 0x80 != 0;
        let res = op1 << 1 | (cin as u8);

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match inst.opcode {
            0x16 => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x10..=0x17 => {
                // Write X
                helpers::set_op8(cpu, inst.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

pub mod rla {
    use super::*;

    pub fn start(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0x17 {
            panic!("Illegal instruction.");
        }

        // Execute RLA
        let flags = &mut cpu.regs.f;
        let cin = Flag::C.get(flags);
        let carry = *cpu.regs.a & 0x80 != 0;
        let res = *cpu.regs.a << 1 | (cin as u8);
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, false);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

pub mod rlc {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x06 => {
                let op1 = cpu.readbyte();
                inst.stack.push(op1);
                inst.exec = done;
                Some(inst)
            }
            0x00..=0x07 => {
                let op1 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op1);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute RLC
        let op1 = inst.stack.pop().unwrap();
        let carry = op1 & 0x80 != 0;
        let res = op1 << 1 | (carry as u8);

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match inst.opcode {
            0x06 => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x00..=0x07 => {
                // Write X
                helpers::set_op8(cpu, inst.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

pub mod rlca {
    use super::*;

    pub fn start(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0x07 {
            panic!("Illegal instruction.");
        }

        // Execute RLCA
        let carry = *cpu.regs.a & 0x80 != 0;
        let res = *cpu.regs.a << 1 | (carry as u8);
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, false);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

pub mod rr {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x1e => {
                let op1 = cpu.readbyte();
                inst.stack.push(op1);
                inst.exec = done;
                Some(inst)
            }
            0x18..=0x1f => {
                let op1 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op1);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute RR
        let op1 = inst.stack.pop().unwrap();
        let flags = &mut cpu.regs.f;
        let cin = Flag::C.get(flags);
        let carry = op1 & 0x01 != 0;
        let res = ((cin as u8) << 7) | op1 >> 1;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match inst.opcode {
            0x1e => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x18..=0x1f => {
                // Write X
                helpers::set_op8(cpu, inst.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

pub mod rra {
    use super::*;

    pub fn start(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0x1f {
            panic!("Illegal instruction.");
        }

        // Execute RRA
        let flags = &mut cpu.regs.f;
        let cin = Flag::C.get(flags);
        let carry = *cpu.regs.a & 0x01 != 0;
        let res = ((cin as u8) << 7) | *cpu.regs.a >> 1;
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, false);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

pub mod rrc {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x0e => {
                let op1 = cpu.readbyte();
                inst.stack.push(op1);
                inst.exec = done;
                Some(inst)
            }
            0x08..=0x0f => {
                let op1 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op1);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute RRC
        let op1 = inst.stack.pop().unwrap();
        let carry = op1 & 0x01 != 0;
        let res = ((carry as u8) << 7) | op1 >> 1;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match inst.opcode {
            0x0e => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x08..=0x0f => {
                // Write X
                helpers::set_op8(cpu, inst.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

pub mod rrca {
    use super::*;

    pub fn start(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0x0f {
            panic!("Illegal instruction.");
        }

        // Execute RRCA
        let carry = *cpu.regs.a & 0x01 != 0;
        let res = ((carry as u8) << 7) | *cpu.regs.a >> 1;
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, false);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

pub mod rst {
    use super::*;

    pub fn start(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => (),
            _ => panic!("Illegal instruction."),
        };

        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 1 cycle to decrement SP.

        // Proceed
        inst.exec = push;
        Some(inst)
    }

    pub fn push(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Push SP
        cpu.pushword(*cpu.regs.pc);

        // Proceed
        inst.exec = delay;
        Some(inst)
    }

    pub fn delay(mut inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to push a u16.

        // Proceed
        inst.exec = jump;
        Some(inst)
    }

    pub fn jump(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let op1 = match inst.opcode {
            0xc7 => 0x00,
            0xcf => 0x08,
            0xd7 => 0x10,
            0xdf => 0x18,
            0xe7 => 0x20,
            0xef => 0x28,
            0xf7 => 0x30,
            0xff => 0x38,
            _ => panic!("Illegal instruction."),
        };
        *cpu.regs.pc = op1;

        // Finish
        None
    }
}

pub mod sbc {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x9e => {
                let op2 = cpu.readbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xde => {
                let op2 = cpu.fetchbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0x98..=0x9f => {
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op2);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SUB
        let acc = *cpu.regs.a;
        let op2 = inst.stack.pop().unwrap();
        let cin = Flag::C.get(&*cpu.regs.f) as u8;
        let (res, carry0) = acc.overflowing_sub(op2);
        let (res, carry1) = res.overflowing_sub(cin);
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, true);
        Flag::H.set(flags, (op2 & 0x0f) + cin > (acc & 0x0f));
        Flag::C.set(flags, carry0 | carry1);

        // Finish
        None
    }
}

pub mod scf {
    use super::*;

    pub fn start(inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0x37 {
            panic!("Illegal instruction.");
        }

        // Execute SCF
        let flags = &mut cpu.regs.f;
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, true);

        // Finish
        None
    }
}

pub mod set {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0xc6 | 0xce | 0xd6 | 0xde | 0xe6 | 0xee | 0xf6 | 0xfe => {
                let op2 = cpu.readbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xc0..=0xff => {
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op2);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SET
        let op1 = (inst.opcode & 0x38) >> 3;
        let op2 = inst.stack.pop().unwrap();
        let mask = !(0b1 << op1);
        let res = (mask & op2) | !mask;

        // Check opcode
        match inst.opcode {
            0xc6 | 0xce | 0xd6 | 0xde | 0xe6 | 0xee | 0xf6 | 0xfe => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0xc0..=0xff => {
                // Write X
                helpers::set_op8(cpu, inst.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

pub mod sla {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x26 => {
                let op1 = cpu.readbyte();
                inst.stack.push(op1);
                inst.exec = done;
                Some(inst)
            }
            0x20..=0x27 => {
                let op1 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op1);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SLA
        let op1 = inst.stack.pop().unwrap();
        let carry = op1 & 0x80 != 0;
        let res = op1 << 1;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match inst.opcode {
            0x26 => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x20..=0x27 => {
                // Write X
                helpers::set_op8(cpu, inst.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

pub mod sra {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x2e => {
                let op1 = cpu.readbyte();
                inst.stack.push(op1);
                inst.exec = done;
                Some(inst)
            }
            0x28..=0x2f => {
                let op1 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op1);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SRA
        let op1 = inst.stack.pop().unwrap();
        let sign = op1 & 0x80;
        let carry = op1 & 0x01 != 0;
        let res = sign | (op1 >> 1);

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match inst.opcode {
            0x2e => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x28..=0x2f => {
                // Write X
                helpers::set_op8(cpu, inst.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

pub mod srl {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x3e => {
                let op1 = cpu.readbyte();
                inst.stack.push(op1);
                inst.exec = done;
                Some(inst)
            }
            0x38..=0x3f => {
                let op1 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op1);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SRL
        let op1 = inst.stack.pop().unwrap();
        let carry = op1 & 0x01 != 0;
        let res = 0x7f & (op1 >> 1);

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match inst.opcode {
            0x3e => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x38..=0x3f => {
                // Write X
                helpers::set_op8(cpu, inst.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

pub mod stop {
    use super::*;

    #[allow(unreachable_code)]
    pub fn start(inst: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        if inst.opcode != 0x10 {
            panic!("Illegal instruction.");
        }

        // Execute STOP
        todo!("implement this mess of an instruction");

        // Finish
        None
    }
}

pub mod sub {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x96 => {
                let op2 = cpu.readbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xd6 => {
                let op2 = cpu.fetchbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0x90..=0x97 => {
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op2);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SUB
        let acc = *cpu.regs.a;
        let op2 = inst.stack.pop().unwrap();
        let (res, carry) = acc.overflowing_sub(op2);
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, true);
        Flag::H.set(flags, (op2 & 0x0f) > (acc & 0x0f));
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

pub mod swap {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0x36 => {
                let op1 = cpu.readbyte();
                inst.stack.push(op1);
                inst.exec = done;
                Some(inst)
            }
            0x30..=0x37 => {
                let op1 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op1);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SWAP
        let op1 = inst.stack.pop().unwrap();
        let res = ((op1 & 0xf0) >> 4) | ((op1 & 0x0f) << 4);

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, false);

        // Check opcode
        match inst.opcode {
            0x36 => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                inst.exec = delay;
                Some(inst)
            }
            0x30..=0x37 => {
                // Write X
                helpers::set_op8(cpu, inst.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

pub mod unused {
    use super::*;

    pub fn start(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        panic!("Illegal instruction.");
    }
}

pub mod xor {
    use super::*;

    pub fn start(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match inst.opcode {
            0xae => {
                let op2 = cpu.readbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xee => {
                let op2 = cpu.fetchbyte();
                inst.stack.push(op2);
                inst.exec = done;
                Some(inst)
            }
            0xa8..=0xaf => {
                let op2 = helpers::get_op8(cpu, inst.opcode & 0x07);
                inst.stack.push(op2);
                done(inst, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut inst: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute XOR
        let acc = *cpu.regs.a;
        let op2 = inst.stack.pop().unwrap();
        let res = acc.bitxor(op2);
        *cpu.regs.a = res;

        // Set flags
        let flags = &mut cpu.regs.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, false);

        // Finish
        None
    }
}
