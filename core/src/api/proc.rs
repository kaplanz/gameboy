//! Processor API.

/// Processor support.
pub trait Support {
    /// Compute interface.
    type Processor: Processor;

    /// Gets the core's processor.
    #[must_use]
    fn cpu(&self) -> &Self::Processor;

    /// Mutably gets the core's processor.
    #[must_use]
    fn cpu_mut(&mut self) -> &mut Self::Processor;
}

/// Processor interface.
pub trait Processor {
    /// Instruction Set Architecture (ISA).
    ///
    /// Represents a valid instance of an instruction in the processor's ISA.
    type Instruction;

    /// Gets the current instruction.
    fn insn(&self) -> Self::Instruction;

    /// Move the PC to the provided address.
    fn goto(&mut self, pc: u16);

    /// Execute the provided instruction in-place.
    fn exec(&mut self, opcode: u8);

    /// Run the provided program (i.e. instruction sequence) in-place.
    fn run(&mut self, prog: &[u8]);

    /// Enable (or wake) the processor.
    fn wake(&mut self);
}
