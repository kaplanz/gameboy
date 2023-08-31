//! Hardware timer.

use std::cell::RefCell;
use std::rc::Rc;

use log::{debug, trace};
use remus::bus::Bus;
use remus::reg::Register;
use remus::{Address, Block, Board, Cell, Device, Location, Machine, Shared};

use super::pic::{Interrupt, Pic};

/// 8-bit timer control register set.
///
/// For more info, see [here][regs].
///
/// [regs]: https://gbdev.io/pandocs/Timer_and_Divider_Registers.html
#[derive(Clone, Copy, Debug)]
pub enum Control {
    /// `0xFF04`: Divider register.
    Div,
    /// `0xFF05`: Timer counter.
    Tima,
    /// `0xFF06`: Timer modulo.
    Tma,
    /// `0xFF07`: Timer control.
    Tac,
}

/// Timer model.
#[rustfmt::skip]
#[derive(Debug, Default)]
pub struct Timer {
    // State
    andres: bool,
    // Connections
    pic: Rc<RefCell<Pic>>,
    // Control
    // ┌──────┬──────────┬─────┐
    // │ Size │   Name   │ Dev │
    // ├──────┼──────────┼─────┤
    // │  4 B │ Control  │ Reg │
    // └──────┴──────────┴─────┘
    file: File,
}

impl Timer {
    /// Sets the timer's programmable interrupt controller.
    pub fn set_pic(&mut self, pic: Rc<RefCell<Pic>>) {
        self.pic = pic;
    }

    /// Gets a reference to the timer's divider register.
    #[must_use]
    pub fn div(&self) -> Shared<Div> {
        self.file.div.clone()
    }

    /// Gets a reference to the timer's counter register.
    #[must_use]
    pub fn tima(&self) -> Shared<Tima> {
        self.file.tima.clone()
    }

    /// Gets a reference to the timer's modulo register.
    #[must_use]
    pub fn tma(&self) -> Shared<Tma> {
        self.file.tma.clone()
    }

    /// Gets a reference to the timer's control register.
    #[must_use]
    pub fn tac(&self) -> Shared<Tac> {
        self.file.tac.clone()
    }

    /// Calculates the AND result.
    ///
    /// Used to determine whether TIMA will be incremented, as documented by
    /// [Hacktix][gbedg].
    ///
    /// [gbedg]: https://github.com/Hacktix/GBEDG/blob/master/timers/index.md#timer-operation
    fn and(&self) -> bool {
        let ena = self.file.tac.borrow().ena();
        let sel = self.file.tac.borrow().sel();
        let div = self.file.div.borrow().div();
        let bit = sel & div == 0;
        ena & bit
    }
}

impl Block for Timer {
    fn reset(&mut self) {
        self.file.reset();
    }
}

impl Board for Timer {
    fn connect(&self, bus: &mut Bus) {
        self.file.connect(bus);
    }
}

#[rustfmt::skip]
impl Location<u8> for Timer {
    type Register = Control;

    fn load(&self, reg: Self::Register) -> u8 {
        match reg {
            Control::Div  => self.file.div.load(),
            Control::Tima => self.file.tima.load(),
            Control::Tma  => self.file.tma.load(),
            Control::Tac  => self.file.tac.load(),
        }
    }

    fn store(&mut self, reg: Self::Register, value: u8) {
        match reg {
            Control::Div  => self.file.div.store(value),
            Control::Tima => self.file.tima.store(value),
            Control::Tma  => self.file.tma.store(value),
            Control::Tac  => self.file.tac.store(value),
        }
    }
}

impl Machine for Timer {
    fn enabled(&self) -> bool {
        true
    }

    #[rustfmt::skip]
    fn cycle(&mut self) {
        // Extract control registers
        let div = self.file.div.borrow().div();
        let tima = self.file.tima.load();
        let reload = self.file.tima.borrow().reload;
        let tma = self.file.tma.load();

        // Check if TIMA should be incremented
        let and = self.and();           // calculate AND result
        let fell = self.andres && !and; // check if falling
        self.andres = and;              // store for next cycle

        // Calculate next TIMA
        if fell {
            // Increment TIMA
            let (tima, carry) = tima.overflowing_add(1);
            self.file.tima.store(tima);
            trace!("timer: {tima}");
            // Store a pending reload on overflow
            if carry {
                self.file.tima.borrow_mut().reload = Some(div.wrapping_add(4));
                debug!("scheduled timer reload");
            }
        } else if Some(div) == reload {
            // Reload from TMA
            self.file.tima.store(tma);
            // Schedule Timer interrupt
            self.pic.borrow_mut().req(Interrupt::Timer);
            debug!("interrupt requested");
        }

        // Increment the divider every T-cycle
        // Note: This has the effect of incrementing DIV every 256 T-cycles.
        self.file.div.borrow_mut().0.store(div.wrapping_add(1));
    }
}

/// Control registers.
#[rustfmt::skip]
#[derive(Debug, Default)]
struct File {
    // ┌──────┬──────────┬─────┬───────┐
    // │ Size │   Name   │ Dev │ Alias │
    // ├──────┼──────────┼─────┼───────┤
    // │  1 B │ Divider  │ Reg │ DIV   │
    // │  1 B │ Counter  │ Reg │ TIMA  │
    // │  1 B │ Modulo   │ Reg │ TMA   │
    // │  1 B │ Control  │ Reg │ TAC   │
    // └──────┴──────────┴─────┴───────┘
    div:  Shared<Div>,
    tima: Shared<Tima>,
    tma:  Shared<Tma>,
    tac:  Shared<Tac>,
}

impl Block for File {
    fn reset(&mut self) {
        self.div.reset();
        self.tima.reset();
        self.tma.reset();
        self.tac.reset();
    }
}

impl Board for File {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let div  = self.div.clone().to_dynamic();
        let tima = self.tima.clone().to_dynamic();
        let tma  = self.tma.clone().to_dynamic();
        let tac  = self.tac.clone().to_dynamic();

        // Map devices on bus  // ┌──────┬──────┬──────────┬─────┐
                               // │ Addr │ Size │   Name   │ Dev │
                               // ├──────┼──────┼──────────┼─────┤
        bus.map(0xff04, div);  // │ ff04 │  1 B │ Divider  │ Reg │
        bus.map(0xff05, tima); // │ ff05 │  1 B │ Counter  │ Reg │
        bus.map(0xff06, tma);  // │ ff06 │  1 B │ Modulo   │ Reg │
        bus.map(0xff07, tac);  // │ ff07 │  1 B │ Control  │ Reg │
                               // └──────┴──────┴──────────┴─────┘
    }
}

/// Divider register.
#[derive(Debug, Default)]
pub struct Div(Register<u16>);

impl Div {
    /// Gets the internal clock (lower 8-bits).
    #[must_use]
    pub fn clk(&self) -> u8 {
        (self.0.load() & 0x00ff) as u8
    }

    /// Gets the full internal register value.
    #[must_use]
    pub fn div(&self) -> u16 {
        self.0.load()
    }
}

impl Address<u8> for Div {
    fn read(&self, _: usize) -> u8 {
        self.load()
    }

    fn write(&mut self, _: usize, value: u8) {
        self.store(value);
    }
}

impl Block for Div {
    fn reset(&mut self) {
        self.0.reset();
    }
}

impl Cell<u8> for Div {
    /// Loads the value of DIV (upper 8-bits).
    fn load(&self) -> u8 {
        (self.0.load() & 0xff00 >> 8) as u8
    }

    fn store(&mut self, _: u8) {
        self.0.store(0);
    }
}

impl Device for Div {
    fn contains(&self, index: usize) -> bool {
        (0..self.len()).contains(&index)
    }

    fn len(&self) -> usize {
        std::mem::size_of::<u8>()
    }
}

/// Timer counter.
#[derive(Debug, Default)]
pub struct Tima {
    reg: Register<u8>,
    reload: Option<u16>,
}

impl Address<u8> for Tima {
    fn read(&self, _: usize) -> u8 {
        self.load()
    }

    fn write(&mut self, _: usize, value: u8) {
        self.store(value);
    }
}

impl Block for Tima {
    fn reset(&mut self) {
        self.reg.reset();
    }
}

impl Cell<u8> for Tima {
    fn load(&self) -> u8 {
        self.reg.load()
    }

    fn store(&mut self, value: u8) {
        self.reg.store(value);
        self.reload = None; // reloads overridden on store
    }
}

impl Device for Tima {
    fn contains(&self, index: usize) -> bool {
        self.reg.contains(index)
    }

    fn len(&self) -> usize {
        self.reg.len()
    }
}

/// Timer modulo.
pub type Tma = Register<u8>;

/// Timer control.
#[derive(Debug, Default)]
pub struct Tac(Register<u8>);

impl Tac {
    /// Gets the enable bit.
    #[must_use]
    pub fn ena(&self) -> bool {
        self.0.load() & 0b100 != 0
    }

    /// Gets the clock select rate.
    #[must_use]
    pub fn sel(&self) -> u16 {
        match self.0.load() & 0b11 {
            0b01 => 0x0008,
            0b10 => 0x0020,
            0b11 => 0x0080,
            0b00 => 0x0200,
            _ => unreachable!(),
        }
    }
}

impl Address<u8> for Tac {
    fn read(&self, _: usize) -> u8 {
        self.load()
    }

    fn write(&mut self, _: usize, value: u8) {
        self.store(value);
    }
}

impl Block for Tac {
    fn reset(&mut self) {
        self.0.reset();
    }
}

impl Cell<u8> for Tac {
    fn load(&self) -> u8 {
        self.0.load()
    }

    fn store(&mut self, value: u8) {
        self.0.store(value & 0b111);
    }
}

impl Device for Tac {
    fn contains(&self, index: usize) -> bool {
        (0..self.len()).contains(&index)
    }

    fn len(&self) -> usize {
        std::mem::size_of::<u8>()
    }
}
