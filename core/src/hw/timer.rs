use std::cell::RefCell;
use std::rc::Rc;

use remus::bus::Bus;
use remus::reg::Register;
use remus::{Block, Device, Machine};

use super::pic::{Interrupt, Pic};

#[rustfmt::skip]
#[derive(Debug, Default)]
pub struct Timer {
    cycle: usize,
    pic: Rc<RefCell<Pic>>,
    pub regs: Rc<RefCell<Registers>>,
}

impl Timer {
    /// Set the timer's pic.
    pub fn set_pic(&mut self, pic: Rc<RefCell<Pic>>) {
        self.pic = pic;
    }
}

impl Block for Timer {
    fn reset(&mut self) {
        // Reset registers
        self.regs.borrow_mut().reset();
    }
}

impl Machine for Timer {
    fn enabled(&self) -> bool {
        true
    }

    fn cycle(&mut self) {
        // Borrow registers
        let regs = &*self.regs.borrow();

        // Increment DIV every 256 cycles
        if self.cycle % 0x100 == 0 {
            let div = &mut **regs.div.borrow_mut();
            *div = div.wrapping_add(1);
        }

        // Increment TIMA if enabled
        let tac = **regs.tac.borrow();
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
            if self.cycle % div == 0 {
                // Increment TIMA
                let tima = &mut **regs.tima.borrow_mut();
                *tima = match tima.checked_add(1) {
                    Some(tima) => tima,
                    None => {
                        // Schedule Timer interrupt
                        self.pic.borrow_mut().req(Interrupt::Timer);
                        // Restart from TMA
                        **regs.tma.borrow()
                    }
                };
            };
        }

        // Keep track of cycle count
        self.cycle = self.cycle.wrapping_add(1);
    }
}

#[rustfmt::skip]
#[derive(Debug, Default)]
pub struct Registers {
    bus: Bus,
    // ┌────────┬──────────────────┬─────┬───────┐
    // │  SIZE  │       NAME       │ DEV │ ALIAS │
    // ├────────┼──────────────────┼─────┼───────┤
    // │    1 B │ Divider Register │ Reg │ DIV   │
    // │    1 B │    Timer Counter │ Reg │ TIMA  │
    // │    1 B │     Timer Modulo │ Reg │ TMA   │
    // │    1 B │    Timer Control │ Reg │ TAC   │
    // └────────┴──────────────────┴─────┴───────┘
    pub div:  Rc<RefCell<Register<u8>>>,
    pub tima: Rc<RefCell<Register<u8>>>,
    pub tma:  Rc<RefCell<Register<u8>>>,
    pub tac:  Rc<RefCell<Register<u8>>>,
}

impl Block for Registers {
    #[rustfmt::skip]
    fn reset(&mut self) {
        // Reset self
        std::mem::take(self);
        // Reset bus                           // ┌──────┬──────────────────┬─────┐
        self.bus.reset();                      // │ SIZE │       NAME       │ DEV │
                                               // ├──────┼──────────────────┼─────┤
        self.bus.map(0x00, self.div.clone());  // │  1 B │ Divider Register │ Reg │
        self.bus.map(0x01, self.tima.clone()); // │  1 B │    Timer Counter │ Reg │
        self.bus.map(0x02, self.tma.clone());  // │  1 B │     Timer Modulo │ Reg │
        self.bus.map(0x03, self.tac.clone());  // │  1 B │    Timer Control │ Reg │
                                               // └──────┴──────────────────┴─────┘
    }
}

impl Device for Registers {
    fn contains(&self, index: usize) -> bool {
        self.bus.contains(index)
    }

    fn len(&self) -> usize {
        self.bus.len()
    }

    fn read(&self, index: usize) -> u8 {
        self.bus.read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        self.bus.write(index, value);
    }
}
