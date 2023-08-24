//! Hardware timer.

use std::cell::RefCell;
use std::rc::Rc;

use remus::bus::Bus;
use remus::reg::Register;
use remus::{Block, Board, Machine, Shared};

use super::pic::{Interrupt, Pic};

/// Timer model.
#[rustfmt::skip]
#[derive(Debug, Default)]
pub struct Timer {
    // State
    clock: usize,
    // Connections
    pic: Rc<RefCell<Pic>>,
    // Control
    file: File,
}

impl Timer {
    /// Gets a reference to the timer's divider register.
    #[allow(unused)]
    #[must_use]
    pub fn div(&self) -> Shared<Register<u8>> {
        self.file.div.clone()
    }

    /// Gets a reference to the timer's counter register.
    #[allow(unused)]
    #[must_use]
    pub fn tima(&self) -> Shared<Register<u8>> {
        self.file.tima.clone()
    }

    /// Gets a reference to the timer's modulo register.
    #[allow(unused)]
    #[must_use]
    pub fn tma(&self) -> Shared<Register<u8>> {
        self.file.tma.clone()
    }

    /// Gets a reference to the timer's control register.
    #[allow(unused)]
    #[must_use]
    pub fn tac(&self) -> Shared<Register<u8>> {
        self.file.tac.clone()
    }

    /// Set the timer's pic.
    pub fn set_pic(&mut self, pic: Rc<RefCell<Pic>>) {
        self.pic = pic;
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

impl Machine for Timer {
    fn enabled(&self) -> bool {
        true
    }

    fn cycle(&mut self) {
        // Borrow control registers
        let div = &mut **self.file.div.borrow_mut();
        let tima = &mut **self.file.tima.borrow_mut();
        let tma = &**self.file.tma.borrow();
        let tac = &**self.file.tac.borrow();

        // Increment DIV every 256 cycles
        if self.clock % 0x100 == 0 {
            *div = div.wrapping_add(1);
        }

        // Increment TIMA if enabled
        if tac & 0x04 != 0 {
            // Determine TIMA divider
            let div = match tac & 0x03 {
                0b00 => 0x2000,
                0b01 => 0x0010,
                0b10 => 0x0040,
                0b11 => 0x0100,
                _ => unreachable!(),
            };
            // Check if this is a tic cycle
            if self.clock % div == 0 {
                // Increment TIMA
                *tima = if let Some(tima) = tima.checked_add(1) {
                    tima
                } else {
                    // Schedule Timer interrupt
                    self.pic.borrow_mut().req(Interrupt::Timer);
                    // Restart from TMA
                    *tma
                };
            };
        }

        // Keep track of cycle count
        self.clock = self.clock.wrapping_add(1);
    }
}

/// Control registers.
#[rustfmt::skip]
#[derive(Debug, Default)]
pub struct File {
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
        self.div.borrow_mut().reset();
        self.tima.borrow_mut().reset();
        self.tma.borrow_mut().reset();
        self.tac.borrow_mut().reset();
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
