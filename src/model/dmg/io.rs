//! Input/Output ports.

use std::cell::RefCell;
use std::rc::Rc;

use gameboy_core::hw::{joypad, ppu, timer};
use remus::bus::Bus;
use remus::mem::Ram;
use remus::reg::Register;
use remus::{Block, Device};

use super::boot;

/// Input/Output connections.
///
/// This struct is plain old data (POD), and its members are accessed by
/// [`GameBoy`](super::GameBoy).
#[rustfmt::skip]
#[derive(Debug, Default)]
pub struct InOut {
    pub bus: Rc<RefCell<Bus>>,
    // ┌────────┬──────────────────┬─────┐
    // │  SIZE  │       NAME       │ DEV │
    // ├────────┼──────────────────┼─────┤
    // │    1 B │       Controller │ Reg │
    // │    2 B │    Communication │ Reg │
    // │    4 B │  Divider & Timer │ Reg │
    // │    1 B │   Interrupt Flag │ Reg │
    // │   23 B │            Sound │ RAM │
    // │   16 B │         Waveform │ RAM │
    // │   16 B │              LCD │ PPU │
    // │    1 B │ Boot ROM Disable │ Reg │
    // └────────┴──────────────────┴─────┘
    pub con:   Rc<RefCell<joypad::Register>>,
    pub com:   Rc<RefCell<Register<u16>>>,
    pub timer: Rc<RefCell<timer::Registers>>,
    pub iflag: Rc<RefCell<Register<u8>>>,
    pub sound: Rc<RefCell<Ram<0x17>>>,
    pub wave:  Rc<RefCell<Ram<0x10>>>,
    pub lcd:   Rc<RefCell<ppu::Registers>>,
    pub boot:  Rc<RefCell<boot::RomDisable>>,
}

impl InOut {
    #[rustfmt::skip]
    fn memmap(&mut self) {
        // Prepare bus
        self.bus.take();
        let mut bus = self.bus.borrow_mut();

        // Prepare devices
        let con = self.con.clone();
        let com = self.com.clone();
        let timer = self.timer.clone();
        let iflag = self.iflag.clone();
        let sound = self.sound.clone();
        let wave = self.wave.clone();
        let lcd = self.lcd.clone();
        let boot = self.boot.clone();

        // Map devices in I/O // ┌────────┬─────────────────┬─────┐
                              // │  SIZE  │      NAME       │ DEV │
                              // ├────────┼─────────────────┼─────┤
        bus.map(0x00, con);   // │    1 B │      Controller │ Reg │
        bus.map(0x01, com);   // │    2 B │   Communication │ Reg │
                              // │    1 B │        Unmapped │ --- │
        bus.map(0x04, timer); // │    4 B │ Divider & Timer │ Reg │
                              // │    7 B │        Unmapped │ --- │
        bus.map(0x0f, iflag); // │    1 B │  Interrupt Flag │ Reg │
        bus.map(0x10, sound); // │   23 B │           Sound │ RAM │
                              // │    9 B │        Unmapped │ --- │
        bus.map(0x30, wave);  // │   16 B │        Waveform │ RAM │
        bus.map(0x40, lcd);   // │   12 B │             LCD │ Ppu │
                              // │    4 B │        Unmapped │ --- │
        bus.map(0x50, boot);  // │    1 B │   Boot ROM Bank │ Reg │
                              // │   47 B │        Unmapped │ --- │
                              // └────────┴─────────────────┴─────┘
    }
}

impl Block for InOut {
    fn reset(&mut self) {
        // Re-map bus
        self.memmap();
    }
}

impl Device for InOut {
    fn contains(&self, index: usize) -> bool {
        self.bus.borrow().contains(index)
    }

    fn len(&self) -> usize {
        self.bus.borrow().len()
    }

    fn read(&self, index: usize) -> u8 {
        self.bus.borrow().read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        self.bus.borrow_mut().write(index, value);
    }
}
