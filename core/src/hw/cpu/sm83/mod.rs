//! SM83 core.
//!
//! Model for the CPU core present on the Sharp LR35902.

use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use enumflag::Enumflag;
use log::{debug, trace};
use remus::bus::Bus;
use remus::{reg, Block, Device, Machine};

use self::insn::Instruction;
use super::Processor;
use crate::dmg::Board;
use crate::hw::pic::Pic;

mod insn;

/// 16-bit register set.
#[allow(dead_code)]
pub enum Register {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

/// SM83 central processing unit.
#[derive(Debug, Default)]
pub struct Cpu {
    /// Memory address bus.
    bus: Rc<RefCell<Bus>>,
    /// Programmable interrupt controller.
    pic: Rc<RefCell<Pic>>,
    /// Internal register set.
    regs: Registers,
    /// Run status.
    status: Status,
    /// Execution state.
    state: State,
    /// Interrupt master enable.
    ime: Ime,
    halt_bug: bool,
}

impl Cpu {
    /// Fetch the next byte after PC.
    fn fetchbyte(&mut self) -> u8 {
        let pc = &mut *self.regs.pc;
        let byte = self.bus.borrow().read(*pc as usize);
        *pc = pc.wrapping_add(1);
        byte
    }

    /// Read the byte at HL.
    fn readbyte(&mut self) -> u8 {
        let hl = self.regs.hl.get(&self.regs);
        self.bus.borrow().read(hl as usize)
    }

    /// Write to the byte at HL
    fn writebyte(&mut self, byte: u8) {
        let hl = self.regs.hl.get(&self.regs);
        self.bus.borrow_mut().write(hl as usize, byte);
    }

    /// Fetch the next word after PC.
    fn fetchword(&mut self) -> u16 {
        let pc = &mut *self.regs.pc;
        let mut word = [0; 2];
        word[0] = self.bus.borrow().read(*pc as usize);
        *pc = pc.wrapping_add(1);
        word[1] = self.bus.borrow().read(*pc as usize);
        *pc = pc.wrapping_add(1);
        u16::from_le_bytes(word)
    }

    /// Pop the word at SP.
    fn popword(&mut self) -> u16 {
        let sp = &mut *self.regs.sp;
        let mut word = [0; 2];
        word[0] = self.bus.borrow().read(*sp as usize);
        *sp = sp.wrapping_add(1);
        word[1] = self.bus.borrow().read(*sp as usize);
        *sp = sp.wrapping_add(1);
        u16::from_le_bytes(word)
    }

    /// Push to the word at SP.
    fn pushword(&mut self, word: u16) {
        let sp = &mut *self.regs.sp;
        let word = word.to_le_bytes();
        *sp = sp.wrapping_sub(1);
        self.bus.borrow_mut().write(*sp as usize, word[1]);
        *sp = sp.wrapping_sub(1);
        self.bus.borrow_mut().write(*sp as usize, word[0]);
    }
}

impl Block for Cpu {
    fn reset(&mut self) {
        // Reset each sub-block
        self.bus.borrow_mut().reset();
        self.pic.borrow_mut().reset();
        self.regs.reset();
        // Reset to initial state
        self.status = Status::default();
        self.state = State::default();
        self.ime = Ime::default();
    }
}

impl Board for Cpu {
    fn connect(&self, _: &mut Bus) {}
}

impl Processor for Cpu {
    type Register = Register;

    fn goto(&mut self, pc: u16) {
        *self.regs.pc = pc;
    }

    fn get(&self, reg: Self::Register) -> u16 {
        let af = self.regs.af;
        let bc = self.regs.bc;
        let de = self.regs.de;
        let hl = self.regs.hl;
        let regs = &self.regs;
        match reg {
            Register::AF => af.get(regs),
            Register::BC => bc.get(regs),
            Register::DE => de.get(regs),
            Register::HL => hl.get(regs),
            Register::SP => *self.regs.sp,
            Register::PC => *self.regs.pc,
        }
    }

    fn set(&mut self, reg: Self::Register, value: u16) {
        let af = self.regs.af;
        let bc = self.regs.bc;
        let de = self.regs.de;
        let hl = self.regs.hl;
        let regs = &mut self.regs;
        match reg {
            Register::AF => af.set(regs, value),
            Register::BC => bc.set(regs, value),
            Register::DE => de.set(regs, value),
            Register::HL => hl.set(regs, value),
            Register::SP => *self.regs.sp = value,
            Register::PC => *self.regs.pc = value,
        }
    }

    fn exec(&mut self, opcode: u8) {
        // Create a new instruction...
        let mut insn = Some(Instruction::new(opcode));
        // ... then execute it until completion
        while let Some(work) = insn {
            insn = work.exec(self);
        }
    }

    fn run(&mut self, prog: &[u8]) {
        for &opcode in prog {
            self.exec(opcode);
        }
    }

    fn wake(&mut self) {
        self.status = Status::Enabled;
    }

    fn set_bus(&mut self, bus: Rc<RefCell<Bus>>) {
        self.bus = bus;
    }

    fn set_pic(&mut self, pic: Rc<RefCell<Pic>>) {
        self.pic = pic;
    }
}

impl Machine for Cpu {
    fn enabled(&self) -> bool {
        matches!(self.status, Status::Enabled)
    }

    fn cycle(&mut self) {
        self.state = std::mem::take(&mut self.state).exec(self);
    }
}

/// CPU internal register set.
#[derive(Debug)]
struct Registers {
    // ┌───────┬───────┐
    // │ A: u8 │ F: u8 │
    // ├───────┼───────┤
    // │ B: u8 │ C: u8 │
    // ├───────┼───────┤
    // │ D: u8 │ E: u8 │
    // ├───────┼───────┤
    // │ H: u8 │ L: u8 │
    // ├───────┴───────┤
    // │    SP: u16    │
    // ├───────────────┤
    // │    PC: u16    │
    // └───────────────┘
    a: reg::Register<u8>,
    f: reg::Register<u8>,
    af: WideRegister,
    b: reg::Register<u8>,
    c: reg::Register<u8>,
    bc: WideRegister,
    d: reg::Register<u8>,
    e: reg::Register<u8>,
    de: WideRegister,
    h: reg::Register<u8>,
    l: reg::Register<u8>,
    hl: WideRegister,
    sp: reg::Register<u16>,
    pc: reg::Register<u16>,
}

impl Block for Registers {
    fn reset(&mut self) {
        // NOTE: the values of internal registers other than PC are undefined
        //       after a reset.
        self.pc.reset();
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            a: reg::Register::default(),
            f: reg::Register::default(),
            af: WideRegister {
                get: |regs: &Registers| {
                    let a = *regs.a as u16;
                    let f = *regs.f as u16;
                    (a << 8) | f
                },
                set: |regs: &mut Registers, af: u16| {
                    *regs.a = ((af & 0xff00) >> 8) as u8;
                    *regs.f = (af & 0x00ff) as u8;
                },
            },
            b: reg::Register::default(),
            c: reg::Register::default(),
            bc: WideRegister {
                get: |regs: &Registers| {
                    let b = *regs.b as u16;
                    let c = *regs.c as u16;
                    (b << 8) | c
                },
                set: |regs: &mut Registers, bc: u16| {
                    *regs.b = ((bc & 0xff00) >> 8) as u8;
                    *regs.c = (bc & 0x00ff) as u8;
                },
            },
            d: reg::Register::default(),
            e: reg::Register::default(),
            de: WideRegister {
                get: |regs: &Registers| {
                    let d = *regs.d as u16;
                    let e = *regs.e as u16;
                    (d << 8) | e
                },
                set: |regs: &mut Registers, de: u16| {
                    *regs.d = ((de & 0xff00) >> 8) as u8;
                    *regs.e = (de & 0x00ff) as u8;
                },
            },
            h: reg::Register::default(),
            l: reg::Register::default(),
            hl: WideRegister {
                get: |regs: &Registers| {
                    let h = *regs.h as u16;
                    let l = *regs.l as u16;
                    (h << 8) | l
                },
                set: |regs: &mut Registers, hl: u16| {
                    *regs.h = ((hl & 0xff00) >> 8) as u8;
                    *regs.l = (hl & 0x00ff) as u8;
                },
            },
            sp: reg::Register::default(),
            pc: reg::Register::default(),
        }
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌───┬────┬───┬────┐")?;
        writeln!(f, "│ A │ {:02x} │ F │ {:02x} │", *self.a, *self.f)?;
        writeln!(f, "├───┼────┼───┼────┤")?;
        writeln!(f, "│ B │ {:02x} │ C │ {:02x} │", *self.b, *self.c)?;
        writeln!(f, "├───┼────┼───┼────┤")?;
        writeln!(f, "│ D │ {:02x} │ E │ {:02x} │", *self.d, *self.e)?;
        writeln!(f, "├───┼────┼───┼────┤")?;
        writeln!(f, "│ H │ {:02x} │ L │ {:02x} │", *self.h, *self.l)?;
        writeln!(f, "├───┴────┼───┴────┤")?;
        writeln!(f, "│   SP   │  {:04x}  │", *self.sp)?;
        writeln!(f, "├────────┼────────┤")?;
        writeln!(f, "│   PC   │  {:04x}  │", *self.pc)?;
        write!(f, "└────────┴────────┘")
    }
}

/// 16-bit wide linked register.
#[derive(Copy, Clone)]
struct WideRegister {
    get: fn(&Registers) -> u16,
    set: fn(&mut Registers, u16),
}

impl WideRegister {
    pub fn get(&self, regs: &Registers) -> u16 {
        (self.get)(regs)
    }

    pub fn set(&self, regs: &mut Registers, value: u16) {
        (self.set)(regs, value);
    }
}

impl Debug for WideRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PseudoRegister")
    }
}

/// CPU flags.
#[derive(Copy, Clone, Debug)]
enum Flag {
    Z = 0b1000_0000,
    N = 0b0100_0000,
    H = 0b0010_0000,
    C = 0b0001_0000,
}

impl Enumflag for Flag {}

impl From<Flag> for u8 {
    fn from(value: Flag) -> Self {
        value as u8
    }
}

/// CPU run status.
#[derive(Debug, Default)]
enum Status {
    #[default]
    Enabled,
    Halted,
    _Stopped,
}

/// CPU execution state.
#[derive(Debug, Default)]
enum State {
    #[default]
    Fetch,
    Execute(Instruction),
    Done,
}

impl State {
    fn exec(mut self, cpu: &mut Cpu) -> Self {
        // If we're State::Done, proceed to State::Fetch this cycle
        if let State::Done = self {
            // Log previous register state
            trace!("Registers:\n{}", cpu.regs);

            // Check for pending interrupts
            let int = match cpu.ime {
                Ime::Enabled => cpu.pic.borrow().int(),
                _ => None,
            };

            // Handle pending interrupt...
            if let Some(int) = int {
                // Acknowledge the interrupt
                cpu.pic.borrow_mut().ack(int);
                // Skip State::Fetch
                let insn = Instruction::int(int);
                debug!("0xXXXX: {insn}");
                self = State::Execute(insn);
            }
            // ... or fetch next instruction
            else {
                // Proceed to State::Fetch
                self = State::Fetch;
            }
        }

        // If we're State::Fetch, proceed to State::Execute(_) this cycle
        if let State::Fetch = self {
            // Read the next instruction
            let pc = *cpu.regs.pc;
            let opcode = cpu.fetchbyte();

            // Decode the instruction
            let insn = Instruction::new(opcode);

            // Check for HALT bug
            if cpu.halt_bug {
                // Service the bug by rolling back the PC
                *cpu.regs.pc = cpu.regs.pc.wrapping_sub(1);
                cpu.halt_bug = false;
            }

            // Log the instruction
            // NOTE: Ensure that prefix instructions are logged correctly
            debug!(
                "{pc:#06x}: {}",
                if opcode == 0xcb {
                    let opcode = cpu.bus.borrow().read(*cpu.regs.pc as usize);
                    format!("{}", Instruction::prefix(opcode))
                } else {
                    format!("{insn}")
                }
            );

            // Enable interrupts (after EI, RETI)
            if let Ime::WillEnable = cpu.ime {
                cpu.ime = Ime::Enabled;
            }

            // Proceed to State::Execute(_)
            self = State::Execute(insn);
        }

        // Run the current State::Execute(_)
        if let State::Execute(insn) = self {
            // Execute a cycle of the instruction
            let insn = insn.exec(cpu);
            // Proceed to next State
            self = match insn {
                Some(insn) => State::Execute(insn),
                None => State::Done,
            };
        }

        self
    }
}

/// CPU interrupt master enable.
#[derive(Debug, Default)]
enum Ime {
    #[default]
    Disabled,
    Enabled,
    WillEnable,
}

impl Ime {
    fn enabled(&self) -> bool {
        matches!(self, Self::Enabled)
    }
}
