//! SM83 processor core.

use std::fmt::{Debug, Display};

use log::{debug, error, trace, warn};
use remus::mem::Memory;
use remus::mio::Bus;
use remus::reg::{Port, Register};
use remus::{Block, Byte, Word};

use self::insn::Instruction;
use crate::api::proc::Processor;
use crate::parts::pic;

#[cfg(feature = "debug")]
pub mod dbg;
pub mod insn;

/// Processor byte select.
///
/// See more details [here][regs].
///
/// [regs]: https://gbdev.io/pandocs/CPU_Registers_and_Flags.html
#[derive(Clone, Copy, Debug)]
pub enum Select8 {
    /// Accumulator register.
    A,
    /// Flags register.
    F,
    /// General register B.
    B,
    /// General register C.
    C,
    /// General register D.
    D,
    /// General register E.
    E,
    /// Address (HI) byte.
    H,
    /// Address (LO) byte.
    L,
}

/// Processor word select.
///
/// See more details [here][regs].
///
/// [regs]: https://gbdev.io/pandocs/CPU_Registers_and_Flags.html
#[derive(Clone, Copy, Debug)]
pub enum Select16 {
    /// Joint AF register.
    AF,
    /// Joint BC register.
    BC,
    /// Joint DE register.
    DE,
    /// Address register.
    HL,
    /// Stack pointer.
    SP,
    /// Program counter.
    PC,
}

/// Central processing unit.
#[derive(Debug)]
pub struct Cpu {
    /// Processor registers.
    pub reg: Control,
    /// Processor internals.
    etc: Internal,
    /// Internal bus.
    bus: Bus,
    /// Interrupt line.
    int: pic::Line,
}

/// Processor internals.
#[derive(Debug, Default)]
struct Internal {
    /// Execution stage.
    stage: Stage,
    /// Running status.
    run: Status,
    /// Interrupt master enable.
    ime: Ime,
    /// [Halt bug][bug] flag.
    ///
    /// [bug]: https://gbdev.io/pandocs/halt.html#halt-bug
    halt_bug: bool,
}

impl Internal {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Cpu {
    /// Constructs a new `Cpu`.
    #[must_use]
    pub fn new(bus: Bus, int: pic::Line) -> Self {
        Self {
            reg: Control::default(),
            etc: Internal::default(),
            bus,
            int,
        }
    }

    /// Gets the CPU's execution stage.
    #[must_use]
    pub fn stage(&self) -> &Stage {
        &self.etc.stage
    }

    /// Read the byte at the given address.
    #[must_use]
    pub fn read(&self, addr: Word) -> Byte {
        self.bus
            .read(addr)
            .inspect_err(|err| warn!("failed to read [{addr:#06x}] (default: `0xff`): {err}"))
            .unwrap_or(0xff)
    }

    /// Write to the byte at the given address.
    pub fn write(&mut self, addr: Word, data: Byte) {
        let _ = self
            .bus
            .write(addr, data)
            .inspect_err(|err| warn!("failed to write [{addr:#06x}] <- {data:#04x}: {err}"));
    }

    /// Fetch the next byte after PC.
    fn fetchbyte(&mut self) -> Byte {
        // Load PC
        let mut pc = self.reg.pc.load();
        // Read at PC
        let byte = self.read(pc);
        // Increment PC
        pc = pc.wrapping_add(1);
        self.reg.pc.store(pc);
        // Return fetched byte
        byte
    }

    /// Read the byte at HL.
    fn readbyte(&mut self) -> Byte {
        // Load value of HL
        let hl = self.reg.hl().load();
        // Read at HL
        self.read(hl)
    }

    /// Write to the byte at HL
    fn writebyte(&mut self, byte: Byte) {
        // Load value of HL
        let hl = self.reg.hl().load();
        // Write to HL
        self.write(hl, byte);
    }

    /// Pop the byte at SP.
    fn popbyte(&mut self) -> Byte {
        // Load SP
        let mut sp = self.reg.sp.load();
        // Read at SP
        let byte = self.read(sp);
        // Increment SP
        sp = sp.wrapping_add(1);
        self.reg.sp.store(sp);
        // Return popped byte
        byte
    }

    /// Push to the byte at SP.
    fn pushbyte(&mut self, byte: Byte) {
        // Increment SP
        let mut sp = self.reg.sp.load();
        sp = sp.wrapping_sub(1);
        self.reg.sp.store(sp);
        // Push to SP
        self.write(sp, byte);
    }
}

impl Block for Cpu {
    fn ready(&self) -> bool {
        self.etc.run == Status::Enabled
    }

    fn cycle(&mut self) {
        self.etc.stage = std::mem::take(&mut self.etc.stage).exec(self);
    }

    fn reset(&mut self) {
        self.etc.reset();
        self.reg.reset();
    }
}

impl Port<Byte> for Cpu {
    type Select = Select8;

    fn load(&self, reg: Self::Select) -> Byte {
        match reg {
            Select8::A => self.reg.a.load(),
            Select8::F => self.reg.f.load(),
            Select8::B => self.reg.b.load(),
            Select8::C => self.reg.c.load(),
            Select8::D => self.reg.d.load(),
            Select8::E => self.reg.e.load(),
            Select8::H => self.reg.h.load(),
            Select8::L => self.reg.l.load(),
        }
    }

    fn store(&mut self, reg: Self::Select, value: Byte) {
        match reg {
            Select8::A => self.reg.a.store(value),
            Select8::F => self.reg.f.store(value),
            Select8::B => self.reg.b.store(value),
            Select8::C => self.reg.c.store(value),
            Select8::D => self.reg.d.store(value),
            Select8::E => self.reg.e.store(value),
            Select8::H => self.reg.h.store(value),
            Select8::L => self.reg.l.store(value),
        }
    }
}

impl Port<Word> for Cpu {
    type Select = Select16;

    fn load(&self, reg: Self::Select) -> Word {
        match reg {
            Select16::AF => self.reg.af().load(),
            Select16::BC => self.reg.bc().load(),
            Select16::DE => self.reg.de().load(),
            Select16::HL => self.reg.hl().load(),
            Select16::SP => self.reg.sp.load(),
            Select16::PC => self.reg.pc.load(),
        }
    }

    fn store(&mut self, reg: Self::Select, value: Word) {
        match reg {
            Select16::AF => self.reg.af_mut().store(value),
            Select16::BC => self.reg.bc_mut().store(value),
            Select16::DE => self.reg.de_mut().store(value),
            Select16::HL => self.reg.hl_mut().store(value),
            Select16::SP => self.reg.sp.store(value),
            Select16::PC => self.reg.pc.store(value),
        }
    }
}

impl Processor for Cpu {
    type Insn = Instruction;

    fn insn(&self) -> Self::Insn {
        if let Stage::Execute(insn) = &self.etc.stage {
            insn.clone()
        } else {
            // Fetch opcode at PC
            let pc = self.reg.pc.load();
            let op = self.read(pc);
            // Construct instruction
            Instruction::new(op)
        }
    }

    fn goto(&mut self, addr: Word) {
        self.reg.pc.store(addr);
    }

    fn exec(&mut self, code: Byte) {
        // Create a new instruction...
        let mut insn = Ok(Some(Instruction::new(code)));
        // ... then execute it until completion
        while let Ok(Some(work)) = insn {
            insn = work.exec(self);
        }
        // Report any errors
        if let Err(err) = insn {
            error!("{err}");
        }
    }

    fn run(&mut self, prog: &[Byte]) {
        for &code in prog {
            self.exec(code);
        }
    }

    fn wake(&mut self) {
        self.etc.run = Status::Enabled;
    }
}

/// Processor registers.
///
/// | Size | Name | Description                   |
/// |------|------|-------------------------------|
/// | Byte | A    | Accumulator register.         |
/// | Byte | F    | Flags register.               |
/// | Byte | B    | General register B.           |
/// | Byte | C    | General register C.           |
/// | Byte | D    | General register D.           |
/// | Byte | E    | General register E.           |
/// | Byte | H    | Address (HI) byte.            |
/// | Byte | L    | Address (LO) byte.            |
/// | Word | SP   | Stack pointer.                |
/// | Word | PC   | Program counter.              |
#[derive(Debug, Default)]
pub struct Control {
    /// Accumulator register.
    pub a: Byte,
    /// Flags register.
    pub f: Byte,
    /// General register B.
    pub b: Byte,
    /// General register C.
    pub c: Byte,
    /// General register D.
    pub d: Byte,
    /// General register E.
    pub e: Byte,
    /// Address (HI) byte.
    pub h: Byte,
    /// Address (LO) byte.
    pub l: Byte,
    /// Stack pointer.
    pub sp: Word,
    /// Program counter.
    pub pc: Word,
}

impl Control {
    /// Joint AF register.
    fn af(&self) -> Alias {
        Alias {
            hi: &self.a,
            lo: &self.f,
        }
    }

    /// Joint mutable AF register.
    fn af_mut(&mut self) -> AliasMut {
        AliasMut {
            hi: &mut self.a,
            lo: &mut self.f,
        }
    }

    /// Joint BC register.
    fn bc(&self) -> Alias {
        Alias {
            hi: &self.b,
            lo: &self.c,
        }
    }

    /// Joint mutable BC register.
    fn bc_mut(&mut self) -> AliasMut {
        AliasMut {
            hi: &mut self.b,
            lo: &mut self.c,
        }
    }

    /// Joint DE register.
    fn de(&self) -> Alias {
        Alias {
            hi: &self.d,
            lo: &self.e,
        }
    }

    /// Joint mutable DE register.
    fn de_mut(&mut self) -> AliasMut {
        AliasMut {
            hi: &mut self.d,
            lo: &mut self.e,
        }
    }

    /// Address register.
    fn hl(&self) -> Alias {
        Alias {
            hi: &self.h,
            lo: &self.l,
        }
    }

    /// Mutable address register.
    fn hl_mut(&mut self) -> AliasMut {
        AliasMut {
            hi: &mut self.h,
            lo: &mut self.l,
        }
    }
}

impl Block for Control {
    fn reset(&mut self) {
        std::mem::take(&mut self.pc);
    }
}

impl Display for Control {
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

/// Aliased register.
#[derive(Clone, Copy)]
struct Alias<'a> {
    pub lo: &'a Byte,
    pub hi: &'a Byte,
}

impl Alias<'_> {
    fn load(&self) -> Word {
        let value = [*self.lo, *self.hi];
        Word::from_le_bytes(value)
    }
}

impl Debug for Alias<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.load(), f)
    }
}

impl Display for Alias<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.load(), f)
    }
}

/// Aliased register.
struct AliasMut<'a> {
    pub lo: &'a mut Byte,
    pub hi: &'a mut Byte,
}

impl Register for AliasMut<'_> {
    type Value = Word;

    fn load(&self) -> Self::Value {
        let value = [*self.lo, *self.hi];
        Word::from_le_bytes(value)
    }

    fn store(&mut self, value: Self::Value) {
        let [lo, hi] = value.to_le_bytes();
        *self.lo = lo;
        *self.hi = hi;
    }
}

impl Debug for AliasMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.load(), f)
    }
}

impl Display for AliasMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.load(), f)
    }
}

/// Processor flags.
///
/// | Bit | Name | Explanation         |
/// |-----|------|---------------------|
/// |   7 | Z    | Zero flag.          |
/// |   6 | N    | Subtraction flag.   |
/// |   5 | H    | Half-carry flag.    |
/// |   4 | C    | Carry flag.         |
///
/// See more details [here][flag].
///
/// [flag]: https://gbdev.io/pandocs/CPU_Registers_and_Flags.html#the-flags-register-lower-8-bits-of-af-register
#[derive(Clone, Copy, Debug)]
pub enum Flag {
    /// Zero flag.
    Z = 0b1000_0000,
    /// Subtraction flag.
    N = 0b0100_0000,
    /// Half-carry flag.
    H = 0b0010_0000,
    /// Carry flag.
    C = 0b0001_0000,
}

impl Flag {
    /// Gets the value of the corresponding bit to the flag.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[must_use]
    fn get(self, value: &Byte) -> bool {
        value & self as Byte != 0
    }

    /// Sets the value of the corresponding bit from the flag.
    fn set(self, value: &mut Byte, enable: bool) {
        *value ^= (*value & self as Byte) ^ (!Byte::from(enable).wrapping_sub(1) & self as Byte);
    }
}

/// Processor running status.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Status {
    /// Enabled; normal execution.
    #[default]
    Enabled,
    /// Halted; awaiting interrupt.
    Halted,
    /// Stopped; very low-power.
    #[allow(unused)]
    Stopped,
}

/// Processor execution stage.
#[derive(Clone, Debug, Default)]
pub enum Stage {
    /// Fetch and decode.
    #[default]
    Fetch,
    /// Execute instruction.
    Execute(Instruction),
    /// Completed execution.
    Done,
}

impl Stage {
    #[allow(clippy::match_same_arms)]
    fn exec(mut self, cpu: &mut Cpu) -> Self {
        // If we're `Stage::Done`, proceed to `Stage::Fetch` this cycle
        if let Stage::Done = self {
            // Log previous register stage
            trace!("registers:\n{}", cpu.reg);

            // Check for pending interrupts
            let int = (cpu.etc.ime == Ime::Enabled)
                .then(|| cpu.int.fetch())
                .flatten();

            // Handle pending interrupt...
            if let Some(int) = int {
                // Acknowledge the interrupt
                cpu.int.clear(int);
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
            let pc = cpu.reg.pc.load();
            let op = cpu.fetchbyte();

            // Decode the instruction
            let insn = Instruction::new(op);

            // Check for HALT bug
            if cpu.etc.halt_bug {
                // Service the bug by rolling back the PC
                let mut pc = cpu.reg.pc.load();
                pc = pc.wrapping_sub(1);
                cpu.reg.pc.store(pc);
                cpu.etc.halt_bug = false;
            }

            // Log the instruction
            // NOTE: Ensure that prefix instructions are logged correctly
            debug!(
                "{pc:#06x}: {}",
                if op == 0xcb {
                    let pc = cpu.reg.pc.load();
                    let code = cpu.read(pc);
                    format!("{}", Instruction::prefix(code))
                } else {
                    format!("{insn}")
                }
            );

            // Enable interrupts (after EI, RETI)
            if let Ime::WillEnable = cpu.etc.ime {
                cpu.etc.ime = Ime::Enabled;
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

/// Interrupt master enable.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Ime {
    /// Prevent interrupts.
    #[default]
    Disabled,
    /// Allow interrupts.
    Enabled,
    /// Pending allow.
    WillEnable,
}

impl Ime {
    fn enabled(self) -> bool {
        self == Self::Enabled
    }
}
