//! SM83 CPU core.
//!
//! Model for the CPU core present on the Sharp LR35902.

use std::fmt::{Debug, Display, Write};

use enuf::Enuf;
use log::{debug, error, trace};
use remus::reg::Register;
use remus::{Address, Block, Board, Cell, Linked, Location, Machine, Shared};
use thiserror::Error;

use crate::arch::Bus;
use crate::hw::pic::Pic;

mod insn;

pub use self::insn::Instruction;
pub use super::Processor;

/// Register sets.
pub mod reg {
    /// 8-bit register set.
    #[derive(Clone, Copy, Debug)]
    pub enum Byte {
        A,
        F,
        B,
        C,
        D,
        E,
        H,
        L,
    }

    /// 16-bit register set.
    #[derive(Clone, Copy, Debug)]
    pub enum Word {
        AF,
        BC,
        DE,
        HL,
        SP,
        PC,
    }
}

/// SM83 central processing unit.
#[derive(Debug)]
pub struct Cpu {
    // State
    stage: Stage,
    run: Status,
    ime: Ime,
    halt_bug: bool,
    // Control
    file: File,
    // Shared
    bus: Shared<Bus>,
    pic: Shared<Pic>,
}

impl Cpu {
    /// Constructs a new `Cpu`.
    #[must_use]
    pub fn new(bus: Shared<Bus>, pic: Shared<Pic>) -> Self {
        Self {
            // State
            stage: Stage::default(),
            run: Status::default(),
            ime: Ime::default(),
            halt_bug: bool::default(),
            // Control
            file: File::default(),
            // Shared
            bus,
            pic,
        }
    }

    /// Gets the current execution stage.
    #[must_use]
    pub fn stage(&self) -> &Stage {
        &self.stage
    }

    /// Read the byte at the given address.
    #[must_use]
    pub fn read(&self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    /// Write to the byte at the given address.
    pub fn write(&mut self, addr: u16, byte: u8) {
        self.bus.write(addr, byte);
    }

    /// Fetch the next byte after PC.
    fn fetchbyte(&mut self) -> u8 {
        // Load PC
        let mut pc = self.file.pc.load();
        // Read at PC
        let byte = self.read(pc);
        // Increment PC
        pc = pc.wrapping_add(1);
        self.file.pc.store(pc);
        // Return fetched byte
        byte
    }

    /// Fetch the next word after PC.
    fn fetchword(&mut self) -> u16 {
        let mut word = [0; 2];
        // Fetch lower byte of word
        word[0] = self.fetchbyte();
        // Fetch upper byte of word
        word[1] = self.fetchbyte();
        // Combine bytes into word
        u16::from_le_bytes(word)
    }

    /// Read the byte at HL.
    fn readbyte(&mut self) -> u8 {
        // Load value of HL
        let hl = self.file.hl.load(&self.file);
        // Read at HL
        self.read(hl)
    }

    /// Write to the byte at HL
    fn writebyte(&mut self, byte: u8) {
        // Load value of HL
        let hl = self.file.hl.load(&self.file);
        // Write to HL
        self.write(hl, byte);
    }

    /// Pop the byte at SP.
    fn popbyte(&mut self) -> u8 {
        // Load SP
        let mut sp = self.file.sp.load();
        // Read at SP
        let byte = self.read(sp);
        // Increment SP
        sp = sp.wrapping_add(1);
        self.file.sp.store(sp);
        // Return popped byte
        byte
    }

    /// Pop the word at SP.
    fn popword(&mut self) -> u16 {
        let mut word = [0; 2];
        // Pop lower byte of word
        word[0] = self.popbyte();
        // Pop lower byte of word
        word[1] = self.popbyte();
        // Combine bytes into word
        u16::from_le_bytes(word)
    }

    /// Push to the byte at SP.
    fn pushbyte(&mut self, byte: u8) {
        // Increment SP
        let mut sp = self.file.sp.load();
        sp = sp.wrapping_sub(1);
        self.file.sp.store(sp);
        // Push to SP
        self.write(sp, byte);
    }

    /// Push to the word at SP.
    fn pushword(&mut self, word: u16) {
        let word = word.to_le_bytes();
        // Push upper byte of word
        self.pushbyte(word[1]);
        // Push lower byte of word
        self.pushbyte(word[0]);
    }

    /// Prepares an introspective view of the state.
    #[must_use]
    pub(crate) fn doctor(&self) -> Option<String> {
        // Check if we're ready for the next doctor entry
        if let Stage::Execute(_) = self.stage {
            None
        } else {
            let mut s = String::new();
            let _ = write!(&mut s, "A:{:02X} ", self.file.a.load());
            let _ = write!(&mut s, "F:{:02X} ", self.file.f.load());
            let _ = write!(&mut s, "B:{:02X} ", self.file.b.load());
            let _ = write!(&mut s, "C:{:02X} ", self.file.c.load());
            let _ = write!(&mut s, "D:{:02X} ", self.file.d.load());
            let _ = write!(&mut s, "E:{:02X} ", self.file.e.load());
            let _ = write!(&mut s, "H:{:02X} ", self.file.h.load());
            let _ = write!(&mut s, "L:{:02X} ", self.file.l.load());
            let _ = write!(&mut s, "SP:{:04X} ", self.file.sp.load());
            let _ = write!(&mut s, "PC:{:04X} ", self.file.pc.load());
            let pcmem: Vec<_> = (0..4)
                .map(|i| self.file.pc.load() + i)
                .map(|addr| self.read(addr))
                .collect();
            let _ = write!(
                &mut s,
                "PCMEM:{:02X},{:02X},{:02X},{:02X}",
                pcmem[0], pcmem[1], pcmem[2], pcmem[3],
            );
            Some(s)
        }
    }
}

impl Block for Cpu {
    fn reset(&mut self) {
        // State
        std::mem::take(&mut self.run);
        std::mem::take(&mut self.stage);
        std::mem::take(&mut self.ime);
        std::mem::take(&mut self.halt_bug);
        // Control
        self.file.reset();
    }
}

impl Board<u16, u8> for Cpu {
    fn connect(&self, _: &mut Bus) {}
}

impl Linked<Bus> for Cpu {
    fn mine(&self) -> Shared<Bus> {
        self.bus.clone()
    }

    fn link(&mut self, it: Shared<Bus>) {
        self.bus = it;
    }
}

impl Linked<Pic> for Cpu {
    fn mine(&self) -> Shared<Pic> {
        self.pic.clone()
    }

    fn link(&mut self, it: Shared<Pic>) {
        self.pic = it;
    }
}

impl Location<u8> for Cpu {
    type Register = reg::Byte;

    fn load(&self, reg: Self::Register) -> u8 {
        match reg {
            reg::Byte::A => self.file.a.load(),
            reg::Byte::F => self.file.f.load(),
            reg::Byte::B => self.file.b.load(),
            reg::Byte::C => self.file.c.load(),
            reg::Byte::D => self.file.d.load(),
            reg::Byte::E => self.file.e.load(),
            reg::Byte::H => self.file.h.load(),
            reg::Byte::L => self.file.l.load(),
        }
    }

    fn store(&mut self, reg: Self::Register, value: u8) {
        match reg {
            reg::Byte::A => self.file.a.store(value),
            reg::Byte::F => self.file.f.store(value),
            reg::Byte::B => self.file.b.store(value),
            reg::Byte::C => self.file.c.store(value),
            reg::Byte::D => self.file.d.store(value),
            reg::Byte::E => self.file.e.store(value),
            reg::Byte::H => self.file.h.store(value),
            reg::Byte::L => self.file.l.store(value),
        }
    }
}

impl Location<u16> for Cpu {
    type Register = reg::Word;

    fn load(&self, reg: Self::Register) -> u16 {
        let file = &self.file;
        match reg {
            reg::Word::AF => file.af.load(file),
            reg::Word::BC => file.bc.load(file),
            reg::Word::DE => file.de.load(file),
            reg::Word::HL => file.hl.load(file),
            reg::Word::SP => file.sp.load(),
            reg::Word::PC => file.pc.load(),
        }
    }

    fn store(&mut self, reg: Self::Register, value: u16) {
        let file = &mut self.file;
        match reg {
            reg::Word::AF => (file.af.store)(file, value),
            reg::Word::BC => (file.bc.store)(file, value),
            reg::Word::DE => (file.de.store)(file, value),
            reg::Word::HL => (file.hl.store)(file, value),
            reg::Word::SP => file.sp.store(value),
            reg::Word::PC => file.pc.store(value),
        }
    }
}

impl Machine for Cpu {
    fn enabled(&self) -> bool {
        matches!(self.run, Status::Enabled)
    }

    fn cycle(&mut self) {
        self.stage = std::mem::take(&mut self.stage).exec(self);
    }
}

impl Processor for Cpu {
    type Instruction = Instruction;

    fn insn(&self) -> Self::Instruction {
        if let Stage::Execute(insn) = &self.stage {
            insn.clone()
        } else {
            // Fetch opcode at PC
            let pc = self.file.pc.load();
            let opcode = self.read(pc);
            // Construct instruction
            Instruction::new(opcode)
        }
    }

    fn goto(&mut self, addr: u16) {
        self.file.pc.store(addr);
    }

    fn exec(&mut self, opcode: u8) {
        // Create a new instruction...
        let mut insn = Ok(Some(Instruction::new(opcode)));
        // ... then execute it until completion
        while let Ok(Some(work)) = insn {
            insn = work.exec(self);
        }
        // Report any errors
        if let Err(err) = insn {
            error!("{err}");
        }
    }

    fn run(&mut self, prog: &[u8]) {
        for &opcode in prog {
            self.exec(opcode);
        }
    }

    fn wake(&mut self) {
        self.run = Status::Enabled;
    }
}

/// CPU register file.
#[derive(Debug)]
struct File {
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
    a: Register<u8>,
    f: Register<u8>,
    af: Wide,
    b: Register<u8>,
    c: Register<u8>,
    bc: Wide,
    d: Register<u8>,
    e: Register<u8>,
    de: Wide,
    h: Register<u8>,
    l: Register<u8>,
    hl: Wide,
    sp: Register<u16>,
    pc: Register<u16>,
}

impl Block for File {
    fn reset(&mut self) {
        // NOTE: Registers (other than PC) hold undefined values after a reset.
        self.pc.reset();
    }
}

impl Default for File {
    fn default() -> Self {
        Self {
            a: Register::default(),
            f: Register::default(),
            af: Wide {
                load: |file: &File| {
                    let a = file.a.load();
                    let f = file.f.load();
                    u16::from_be_bytes([a, f])
                },
                store: |file: &mut File, af: u16| {
                    let af = af.to_le_bytes();
                    file.a.store(af[1]);
                    file.f.store(af[0]);
                },
            },
            b: Register::default(),
            c: Register::default(),
            bc: Wide {
                load: |file: &File| {
                    let b = file.b.load();
                    let c = file.c.load();
                    u16::from_be_bytes([b, c])
                },
                store: |file: &mut File, bc: u16| {
                    let bc = bc.to_le_bytes();
                    file.b.store(bc[1]);
                    file.c.store(bc[0]);
                },
            },
            d: Register::default(),
            e: Register::default(),
            de: Wide {
                load: |file: &File| {
                    let d = file.d.load();
                    let e = file.e.load();
                    u16::from_be_bytes([d, e])
                },
                store: |file: &mut File, de: u16| {
                    let de = de.to_le_bytes();
                    file.d.store(de[1]);
                    file.e.store(de[0]);
                },
            },
            h: Register::default(),
            l: Register::default(),
            hl: Wide {
                load: |file: &File| {
                    let h = file.h.load();
                    let l = file.l.load();
                    u16::from_be_bytes([h, l])
                },
                store: |file: &mut File, hl: u16| {
                    let hl = hl.to_le_bytes();
                    file.h.store(hl[1]);
                    file.l.store(hl[0]);
                },
            },
            sp: Register::default(),
            pc: Register::default(),
        }
    }
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌───┬────┬───┬────┐")?;
        writeln!(
            f,
            "│ A │ {:02x} │ F │ {:02x} │",
            self.a.load(),
            self.f.load()
        )?;
        writeln!(f, "├───┼────┼───┼────┤")?;
        writeln!(
            f,
            "│ B │ {:02x} │ C │ {:02x} │",
            self.b.load(),
            self.c.load()
        )?;
        writeln!(f, "├───┼────┼───┼────┤")?;
        writeln!(
            f,
            "│ D │ {:02x} │ E │ {:02x} │",
            self.d.load(),
            self.e.load()
        )?;
        writeln!(f, "├───┼────┼───┼────┤")?;
        writeln!(
            f,
            "│ H │ {:02x} │ L │ {:02x} │",
            self.h.load(),
            self.l.load()
        )?;
        writeln!(f, "├───┴────┼───┴────┤")?;
        writeln!(f, "│   SP   │  {:04x}  │", self.sp.load())?;
        writeln!(f, "├────────┼────────┤")?;
        writeln!(f, "│   PC   │  {:04x}  │", self.pc.load())?;
        write!(f, "└────────┴────────┘")
    }
}

/// 16-bit wide linked register.
#[derive(Copy, Clone)]
struct Wide {
    load: fn(&File) -> u16,
    store: fn(&mut File, u16),
}

impl Wide {
    pub fn load(&self, regs: &File) -> u16 {
        (self.load)(regs)
    }

    pub fn store(&self, regs: &mut File, value: u16) {
        (self.store)(regs, value);
    }
}

impl Debug for Wide {
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

impl Enuf for Flag {}

impl From<Flag> for u8 {
    fn from(value: Flag) -> Self {
        value as u8
    }
}

/// CPU run status.
#[derive(Debug, Default)]
pub enum Status {
    /// Enabled, normal execution.
    #[default]
    Enabled,
    /// Halted, awaiting an interrupt.
    Halted,
    /// Stopped, very low-power state.
    #[allow(unused)]
    Stopped,
}

/// CPU execution stage.
#[derive(Clone, Debug, Default)]
pub enum Stage {
    /// Fetch next instruction.
    #[default]
    Fetch,
    /// Execute fetched instruction.
    Execute(Instruction),
    /// Done executing instruction.
    Done,
}

impl Stage {
    #[allow(clippy::match_same_arms)]
    fn exec(mut self, cpu: &mut Cpu) -> Self {
        // If we're `Stage::Done`, proceed to `Stage::Fetch` this cycle
        if let Stage::Done = self {
            // Log previous register stage
            trace!("registers:\n{}", cpu.file);

            // Check for pending interrupts
            let int = match cpu.ime {
                Ime::Enabled => cpu.pic.borrow().int(),
                _ => None,
            };

            // Handle pending interrupt...
            if let Some(int) = int {
                // Acknowledge the interrupt
                cpu.pic.borrow_mut().ack(int);
                // Skip `Stage::Fetch`
                let insn = Instruction::int(int);
                debug!("{:#06x}: {insn}", int.handler());
                self = Stage::Execute(insn);
            }
            // ... or fetch next instruction
            else {
                // Proceed to `Stage::Fetch`
                self = Stage::Fetch;
            }
        }

        // If we're `Stage::Fetch,` proceed to `Stage::Execute(_)` this cycle
        if let Stage::Fetch = self {
            // Read the next instruction
            let pc = cpu.file.pc.load();
            let opcode = cpu.fetchbyte();

            // Decode the instruction
            let insn = Instruction::new(opcode);

            // Check for HALT bug
            if cpu.halt_bug {
                // Service the bug by rolling back the PC
                let mut pc = cpu.file.pc.load();
                pc = pc.wrapping_sub(1);
                cpu.file.pc.store(pc);
                cpu.halt_bug = false;
            }

            // Log the instruction
            // NOTE: Ensure that prefix instructions are logged correctly
            debug!(
                "{pc:#06x}: {}",
                if opcode == 0xcb {
                    let pc = cpu.file.pc.load();
                    let opcode = cpu.bus.read(pc);
                    format!("{}", Instruction::prefix(opcode))
                } else {
                    format!("{insn}")
                }
            );

            // Enable interrupts (after EI, RETI)
            if let Ime::WillEnable = cpu.ime {
                cpu.ime = Ime::Enabled;
            }

            // Proceed to `Stage::Execute(_)`
            self = Stage::Execute(insn);
        }

        // Run the current `Stage::Execute(_)`
        if let Stage::Execute(insn) = self {
            // Execute a cycle of the instruction
            let insn = insn.exec(cpu);
            // Proceed to next stage
            self = match insn {
                Ok(Some(insn)) => Stage::Execute(insn),
                Ok(None) => Stage::Done,
                Err(insn::Error::Overwrite(insn)) => Stage::Execute(insn),
                Err(err) => {
                    // Log the error
                    error!("{err}");
                    // Continue to next instruction
                    Stage::Done
                }
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

/// A type specifying general categories of [`Instruction`] error.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Instruction(#[from] insn::Error),
}
