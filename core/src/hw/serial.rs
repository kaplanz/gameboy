//! Serial chip.

use std::collections::VecDeque;
use std::io::Read;

use log::{debug, trace, warn};
use remus::bus::Bus;
use remus::dev::Device;
use remus::reg::Register;
use remus::{Address, Block, Board, Cell, Machine, Shared};

use super::pic::{Interrupt, Pic};

/// Serial interface model.
#[derive(Debug, Default)]
pub struct Serial {
    // State
    ip: u8,
    rx: VecDeque<u8>,
    tx: VecDeque<u8>,
    // Control
    // ┌──────┬──────────┬─────┬───────┐
    // │ Size │   Name   │ Dev │ Alias │
    // ├──────┼──────────┼─────┼───────┤
    // │  2 B │ Control  │ Reg │ SC    │
    // └──────┴──────────┴─────┴───────┘
    file: File,
    /// Shared
    pic: Shared<Pic>,
}

impl Serial {
    /// Constructs a new `Serial`.
    pub fn new(pic: Shared<Pic>) -> Self {
        Self {
            pic,
            ..Default::default()
        }
    }

    /// Perform a tick of the external clock.
    #[allow(unused)]
    pub fn tick(&mut self) {
        // Extract control register
        let sc = self.file.sc.borrow();
        // Only tick if transferring on external clock
        if sc.ena && !sc.clk {
            drop(sc); // release borrow on `self`

            // Perform a cycle
            self.cycle();

            // When complete...
            if self.file.sb.borrow().mask == 0 {
                // ...trigger an interrupt
                self.pic.borrow_mut().req(Interrupt::Serial);
            }
        }
    }
}

impl Block for Serial {
    fn reset(&mut self) {
        // Control
        self.file.reset();
    }
}

impl Board for Serial {
    fn connect(&self, bus: &mut Bus) {
        // Connect boards
        self.file.connect(bus);
    }
}

impl Machine for Serial {
    fn enabled(&self) -> bool {
        // Extract control register
        let sc = self.file.sc.borrow();
        // Only enable if transferring on internal clock
        sc.ena && sc.clk
    }

    fn cycle(&mut self) {
        // Extract control registers
        let mut sc = self.file.sc.borrow_mut();
        let mut sb = self.file.sb.borrow_mut();
        let mask = sb.mask;

        // Determine receiving bit
        let rx = self.rx.front().map_or(true, |rx| rx & mask != 0);

        // Perform transfer-exchange
        let tx = sb.tex(rx);

        // Store transferred bit
        #[allow(clippy::match_bool)]
        let tx = match tx {
            true => 0xff,
            false => 0x00,
        };
        self.ip |= tx & mask;

        // Clean-up after transfer is complete
        if sb.mask == 0 {
            // Transfer out byte
            let tx = std::mem::take(&mut self.ip);
            self.tx.push_back(tx);
            // Mark as complete
            sc.ena = false;
            debug!("finished tx: {tx:#04x}");
        }
    }
}

impl Read for Serial {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.tx.read(buf)
    }
}

/// Control registers.
#[derive(Debug, Default)]
struct File {
    // ┌──────┬──────────┬─────┬───────┐
    // │ Size │   Name   │ Dev │ Alias │
    // ├──────┼──────────┼─────┼───────┤
    // │  1 B │ Data     │ Reg │ SB    │
    // │  1 B │ Control  │ Reg │ SC    │
    // └──────┴──────────┴─────┴───────┘
    sb: Shared<Data>,
    sc: Shared<Control>,
}

impl Block for File {
    fn reset(&mut self) {
        self.sb.reset();
        self.sc.reset();
    }
}

impl Board for File {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let sb = self.sb.clone().to_dynamic();
        let sc = self.sc.clone().to_dynamic();

        // Map devices on bus // ┌──────┬──────┬──────────┬─────┐
                              // │ Addr │ Size │   Name   │ Dev │
                              // ├──────┼──────┼──────────┼─────┤
        bus.map(0xff01, sb);  // │ ff01 │  1 B │ Data     │ Reg │
        bus.map(0xff02, sc);  // │ ff02 │  1 B │ Control  │ Reg │
                              // └──────┴──────┴──────────┴─────┘
    }
}

/// Serial data.
#[derive(Debug, Default)]
pub struct Data {
    data: Register<u8>,
    mask: u8,
}

impl Data {
    /// Shift-exchange, simultaneously shifting a bit out and in.
    fn tex(&mut self, rx: bool) -> bool {
        // Load data
        let mut data = self.data.load();
        // Perform transfer
        let tx = data & 0x80 != 0;
        data = (data << 1) | (rx as u8);
        trace!("tx: {}, rx: {}", tx as u8, rx as u8);
        // Update bitmask
        self.mask >>= 1;
        // Store data
        self.data.store(data);
        // Return output
        tx
    }
}

impl Address<u8> for Data {
    fn read(&self, _: usize) -> u8 {
        self.load()
    }

    fn write(&mut self, _: usize, value: u8) {
        self.store(value);
    }
}

impl Block for Data {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Cell<u8> for Data {
    fn load(&self) -> u8 {
        self.data.load()
    }

    fn store(&mut self, value: u8) {
        debug!("started tx: {value:#04x}");
        if self.mask != 0 {
            warn!("interrupted serial transfer");
        }
        // Store byte to transfer
        self.data.store(value);
        // Reset transfer sequence bit
        self.mask = 0b1000_0000;
    }
}

impl Device for Data {
    fn contains(&self, index: usize) -> bool {
        self.data.contains(index)
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

/// Serial control.
#[derive(Debug, Default)]
pub struct Control {
    ena: bool,
    clk: bool,
}

impl Address<u8> for Control {
    fn read(&self, _: usize) -> u8 {
        self.load()
    }

    fn write(&mut self, _: usize, value: u8) {
        self.store(value);
    }
}

impl Block for Control {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Cell<u8> for Control {
    fn load(&self) -> u8 {
        (self.ena as u8) << 7 | 0x7e | (self.clk as u8)
    }

    fn store(&mut self, value: u8) {
        self.ena = value & 0x80 != 0;
        self.clk = value & 0x01 != 0;
    }
}

impl Device for Control {
    fn contains(&self, index: usize) -> bool {
        index < 1
    }

    fn len(&self) -> usize {
        1
    }
}
