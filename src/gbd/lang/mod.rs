use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use derange::Derange;
use displaydoc::Display;

use super::Tick;
use crate::core::dmg::{cpu, pic, ppu, serial, timer};

mod parse;

pub use self::parse::Error;

#[derive(Clone, Debug)]
pub struct Program(VecDeque<Command>);

impl Program {
    #[allow(unused)]
    pub fn new(prog: impl Iterator<Item = Command>) -> Self {
        Self(prog.collect())
    }
}

impl Deref for Program {
    type Target = VecDeque<Command>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Program {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Program {
    type Item = Command;

    type IntoIter = <VecDeque<Command> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Debugger commands.
#[derive(Clone, Debug)]
pub enum Command {
    /// Set a [breakpoint][`Keyword::Break`].
    Break(u16),
    /// [Capture][`Keyword::Capture`] a screenshot.
    Capture(PathBuf, bool),
    /// [Continue][`Keyword::Continue`] execution.
    Continue,
    /// [Delete][`Keyword::Delete`] a breakpoint.
    Delete(usize),
    /// [Disable][`Keyword::Disable`] a breakpoint.
    Disable(usize),
    /// [Enable][`Keyword::Enable`] a breakpoint.
    Enable(usize),
    /// Change debugger [frequency][`Keyword::Freq`].
    Freq(Option<Tick>),
    /// [Go-to][`Keyword::Goto`] an address.
    Goto(u16),
    /// Print debugger [help][`Keyword::Help`].
    Help(Option<Keyword>),
    /// [Ignore][`Keyword::Ignore`] crossings of a breakpoint.
    Ignore(usize, usize),
    /// Print [info][`Keyword::Info`] about debugger state.
    Info(Option<Keyword>),
    /// [Jump][`Keyword::Jump`] and [continue][`Keyword::Continue`] execution at
    /// an address.
    Jump(u16),
    /// [List][`Keyword::List`] the current instruction.
    List,
    /// [Load][`Keyword::Load`] a register's value.
    Load(Vec<Location>),
    /// Change the [log][`Keyword::Log`] level.
    Log(Option<String>),
    /// [Quit][`Keyword::Load`] the emulator.
    Quit,
    /// [Read][`Keyword::Read`] a memory address.
    Read(u16),
    /// [Read][`Keyword::Read`] a range of memory addresses.
    ReadRange(Derange<u16>),
    /// Perform a [serial][`Keyword::Serial`] operation.
    Serial(Serial),
    /// [Reset][`Keyword::Reset`] the emulator.
    Reset,
    /// Perform a single execution [step][`Keyword::Step`].
    Step(Option<usize>),
    /// [Store][`Keyword::Load`] a value to a register.
    Store(Vec<Location>, Value),
    /// [Write][`Keyword::Write`] a value to a memory address.
    Write(u16, u8),
    /// [Write][`Keyword::Write`] a value to a range of memory addresses.
    WriteRange(Derange<u16>, u8),
}

/// Debugger keyword help.
#[derive(Clone, Debug, Display)]
pub enum Keyword {
    /**
     * Game Boy Debugger.
     *
     * COMMANDS:
     * * `break`,     `br`,   `b`: Set breakpoint.
     * * `capture`,   `ps`       : Capture a screenshot.
     * * `continue`,  `cont`, `c`: Resume execution.
     * * `delete`,    `del`      : Delete breakpoint.
     * * `disable`,   `dis`,  `d`: Disable breakpoint.
     * * `enable`,    `en`,   `e`: Enable breakpoint.
     * * `frequency`, `freq`, `d`: Change step frequency.
     * * `goto`,      `go`,   `g`: Goto address.
     * * `help`,              `h`: Print help.
     * * `ignore`,    `ig`       : Ignore breakpoint.
     * * `jump`,      `jp`,   `j`: Jump and execute.
     * * `list`,      `ls`,   `l`: List instruction.
     * * `load`,      `ld`       : Load register.
     * * `log`,       `lo`       : Change logging level.
     * * `quit`,              `q`: Quit emulator.
     * * `read`,      `rd`,   `r`: Read address.
     * * `reset`,     `res`      : Reset emulator.
     * * `step`,              `s`: Perform debugger step.
     * * `store`,     `sr`       : Store register.
     * * `write`,     `wr`,   `w`: Write address.
     *
     * Use `help` for more information about how to use a command.
     */
    All,
    /**
     * `break <ADDRESS>`
     *
     * Set breakpoint at specified location.
     *
     * Note that due to the SM83 CPU supporting multi-byte instructions, there
     * is a chance that the specified breakpoint will not occur upon an
     * instruction boundary. When this occurs, the breakpoint will NOT trigger.
     *
     * Aliases: `br`, `b`
     */
    Break,
    /**
     * `capture[!] <PATH>`
     *
     * Capture and save a screenshot to the provided path.
     *
     * Screenshots are saved as a PNG formatted image. The filename may be
     * modified to include the "png" extension as needed.
     *
     * To forcefully overwrite the file at the selected path, pass the `!`
     * argument.
     *
     * Aliases: `ps`
     */
    Capture,
    /**
     * `continue`
     *
     * Continue program being debugged, after signal or breakpoint.
     *
     * Execution will continue until the next SIGINT signal (triggered most
     * commonly by supplying CTRL-C) is sent, or the executing program reaches a
     * breakpoint.
     *
     * Aliases: `cont`, `c`
     */
    Continue,
    /**
     * `delete <BREAKPOINT>`
     *
     * Delete breakpoint with specified index.
     *
     * Note that breakpoint indices are generally not reused (monotonically
     * increasing), however, if another breakpoint is later created at the
     * address associated with a deleted breakpoint, the index will be restored.
     *
     * Aliases: `del`
     */
    Delete,
    /**
     * `disable <BREAKPOINT>`
     *
     * Disable the breakpoint at the provided index, preventing it from pausing
     * execution when reached.
     *
     * Aliases: `dis`, `d`
     */
    Disable,
    /**
     * `enable <BREAKPOINT>`
     *
     * Enable the breakpoint at the provided index, causing it to pause
     * execution when reached.
     *
     * Aliases: `en`, `e`
     */
    Enable,
    /**
     * `frequency [TICK]`
     *
     * Print or set the debugger's execution frequency.
     *
     * `TICK` must be one of:
     * * `d`, `dot`:   True system clock, notably used by the PPU; occurs at a
     *                 frequency of 4 MiHz.
     * * `m`, `mach`:  Default frequency, used by CPU; always exactly 4 dots.
     * * `i`, `insn`:  Variable frequency, equal to the duration of the current
     *                 instruction; always 1-6 machine cycles.
     * * `l`, `line`:  Duration to draw one line of the LCD display; always
     *                 exactly 456 dots.
     * * `r`, `frame`: Duration to draw an entire frame of the LCD display;
     *                 always exactly 154 scanlines.
     *
     * Aliases: `freq`, `f`
     *
     * See also: `step`
     */
    Freq,
    /**
     * `goto <ADDRESS>`
     *
     * Set the PC to the specified address, without continuing execution.
     *
     * Note that if the current instruction has already been fetched, it will
     * complete execution at the specified address. This has the consequence of
     * potentially reading incorrect data if the executing instruction performs
     * a fetch.
     *
     * Aliases: `go`, `g`
     *
     * Alias of: `store pc <ADDRESS>`
     *
     * See also: `jump`
     */
    Goto,
    /**
     * `help [COMMAND]`
     *
     * Print help for the provided command.
     *
     * Aliases: `h`
     */
    Help,
    /**
     * `ignore <BREAKPOINT> <COUNT>`
     *
     * Ignore the next `COUNT` crossings of the breakpoint at the specified
     * index.
     *
     * Aliases: `ig`
     */
    Ignore,
    /**
     * `info [KEYWORD]`
     *
     * Print info about the debugger's state.
     *
     * Currently only supports breakpoints, with the `break` keyword.
     *
     * Aliases: `i`
     */
    Info,
    /**
     * `jump <ADDRESS>`
     *
     * Set the PC to the specified address, resuming execution.
     *
     * Note that if the current instruction has already been fetched, it will
     * complete execution at the specified address. This has the consequence of
     * potentially reading incorrect data if the executing instruction performs
     * a fetch.
     *
     * Aliases: `jp`, `j`
     */
    Jump,
    /**
     * `list`
     *
     * Print the instruction currently being executed.
     *
     * If no instruction is being executed, decode and print the instruction at
     * the current value of the PC.
     *
     * Aliases: `ls`, `l`
     *
     * See also: `goto`
     */
    List,
    /**
     * `load <REGISTER...>`
     *
     * Load the value of the specified register(s) and print.
     *
     * If specified using the special `lb` or `lw` alias, the specified resister
     * must be either byte or word size respectively.
     *
     * Valid 8-bit (byte) registers are:
     * * CPU: A, F, B, C, D, E, H, L
     * * PPU: LCDC, STAT, SCY, SCX, LYC, LY, DMA, BGP, OBP0, OBP1, WY, WX
     * * Timer: DIV, TIMA, TMA, TAC
     *
     * Valid 16-bit (word) registers are:
     * * CPU: AF, BC, DE, HL, SP, PC
     *
     * Aliases: `ld`
     * Special: `lb`, `lw`
     *
     * See also: `store`
     */
    Load,
    /**
     * `log [FILTER]`
     *
     * Print or change the current logging level filter.
     *
     * See the format specified by the `env_logger` crate for more details.
     *
     * Aliases: `lo`
     */
    Log,
    /**
     * `quit`
     *
     * Exit the debugger, closing the program.
     *
     * Aliases: `q`
     */
    Quit,
    /**
     * `read <ADDRESS | RANGE>`
     *
     * Read the byte(s) at the specified address or range and print.
     *
     * Ranges can be specified as one of the following (Rust semantics):
     * * `A..B`: Exclusive on the right
     * * `A..=B`: Inclusive
     *
     * Note that either or both of the left and right bounds can be omitted to
     * imply the start or end of memory. If the left is larger than the right,
     * the read will overflow, wrapping around.
     *
     * Aliases: `rd`, `r`
     *
     * See also: `write`
     */
    Read,
    /**
     * `reset`
     *
     * Reset the emulator, equivalent to cycling the console's power switch.
     *
     * Aliases: `res`
     */
    Reset,
    /**
     * `serial[!] [DATA]`
     *
     * Receive or transmit data to the serial port.
     *
     * To read and drain the send data buffer, pass the `!` argument.
     *
     * Data to be transmitted must be in one of the following forms:
     * * Buffer: A byte array, e.g. `[0x44, 0x61, 0x74, 0x61]`
     * * String: ASCII string, e.g. `"Hello, world!"`
     *
     * Aliases: `sx`
     */
    Serial,
    /**
     * `step [COUNT]`
     *
     * Perform a (or many) steps of the debugger at the specified frequency.
     *
     * Aliases: `s`
     *
     * See also: `freq`
     */
    Step,
    /**
     * `store <REGISTER...> <VALUE>`
     *
     * Store a value to the specified register(s) and print.
     *
     * If specified using the special `sb` or `sw` alias, the specified resister
     * must be either byte or word size respectively.
     *
     * Valid 8-bit (byte) registers are:
     * * CPU: A, F, B, C, D, E, H, L
     * * PPU: LCDC, STAT, SCY, SCX, LYC, LY, DMA, BGP, OBP0, OBP1, WY, WX
     * * Timer: DIV, TIMA, TMA, TAC
     *
     * Valid 16-bit (word) registers are:
     * * CPU: AF, BC, DE, HL, SP, PC
     *
     * Aliases: `sr`
     * Special: `sb`, `sw`
     *
     * See also: `load`
     */
    Store,
    /**
     * `write <ADDRESS | RANGE> <BYTE>`
     *
     * Write a byte to the specified address or range.
     *
     * When writing to a range, the byte is repeated for each address within the
     * range.
     *
     * Ranges can be specified as one of the following (Rust semantics):
     * * `A..B`: Exclusive on the right
     * * `A..=B`: Inclusive
     *
     * Note that either or both of the left and right bounds can be omitted to
     * imply the start or end of memory. If the left is larger than the right,
     * the write will overflow, wrapping around.
     *
     * Aliases: `wr`, `w`
     *
     * See also: `read`
     */
    Write,
}

#[derive(Clone, Debug)]
pub enum Location {
    Byte(cpu::reg::Byte),
    Word(cpu::reg::Word),
    Pic(pic::Control),
    Ppu(ppu::Control),
    Serial(serial::Control),
    Timer(timer::Control),
}

#[derive(Clone, Debug)]
pub enum Value {
    Byte(u8),
    Word(u16),
}

#[derive(Clone, Debug)]
pub enum Serial {
    Peek,
    Recv,
    Send(Vec<u8>),
}
