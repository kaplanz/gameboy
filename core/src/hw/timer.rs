//! Hardware timer.

use std::cell::RefCell;
use std::rc::Rc;

use remus::bus::Bus;
use remus::reg::Register;
use remus::{Block, Board, Cell, Location, Machine, Shared};

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
    clock: usize,
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
    pub fn div(&self) -> Shared<Register<u8>> {
        self.file.div.clone()
    }

    /// Gets a reference to the timer's counter register.
    #[must_use]
    pub fn tima(&self) -> Shared<Register<u8>> {
        self.file.tima.clone()
    }

    /// Gets a reference to the timer's modulo register.
    #[must_use]
    pub fn tma(&self) -> Shared<Register<u8>> {
        self.file.tma.clone()
    }

    /// Gets a reference to the timer's control register.
    #[must_use]
    pub fn tac(&self) -> Shared<Register<u8>> {
        self.file.tac.clone()
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

    fn cycle(&mut self) {
        // Extract control registers
        let div = self.file.div.load();
        let tima = self.file.tima.load();
        let tma = self.file.tma.load();
        let tac = self.file.tac.load();

        // Increment DIV every 256 cycles
        if self.clock % 0x100 == 0 {
            self.file.div.store(div.wrapping_add(1));
        }

        // Increment TIMA if enabled
        if tac & 0x04 != 0 {
            // Determine TIMA divider
            let div = match tac & 0x03 {
                0b00 => 0x0400,
                0b01 => 0x0010,
                0b10 => 0x0040,
                0b11 => 0x0100,
                _ => unreachable!(),
            };
            // Check if this is a tic cycle
            if self.clock % div == 0 {
                // Increment TIMA
                let tima = if let Some(tima) = tima.checked_add(1) {
                    tima
                } else {
                    // Schedule Timer interrupt
                    self.pic.borrow_mut().req(Interrupt::Timer);
                    // Restart from TMA
                    tma
                };
                self.file.tima.store(tima);
            };
        }

        // Keep track of cycle count
        self.clock = self.clock.wrapping_add(1);
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
    div:  Shared<Register<u8>>,
    tima: Shared<Register<u8>>,
    tma:  Shared<Register<u8>>,
    tac:  Shared<Register<u8>>,
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
