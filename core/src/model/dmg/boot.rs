//! Boot ROM.

use std::fmt::Debug;

use remus::bus::Mux;
use remus::dev::Device;
use remus::{mem, Address, Block, Board, Cell, Shared};

use crate::arch::Bus;
use crate::dev::ReadOnly;

/// Boot RAM model.
pub type Boot = mem::Rom<u8, 0x100>;

/// Boot ROM.
#[derive(Debug)]
pub struct Rom {
    // Control
    // ┌──────┬──────────┬─────┐
    // │ Size │   Name   │ Dev │
    // ├──────┼──────────┼─────┤
    // │  1 B │ Control  │ Reg │
    // └──────┴──────────┴─────┘
    ctrl: Shared<Control>,
    // Memory
    // ┌────────┬──────┬─────┐
    // │  Size  │ Name │ Dev │
    // ├────────┼──────┼─────┤
    // │  8 KiB │ Boot │ ROM │
    // └────────┴──────┴─────┘
    boot: Shared<Boot>,
}

impl Rom {
    /// Constructs a new `Rom`.
    #[must_use]
    pub fn new(mut bus: Shared<Bus>, rom: Boot) -> Self {
        // Construct shared blocks
        let boot = Shared::from(rom);
        let ctrl = Shared::from(Control::new({
            let boot = boot.clone().to_dynamic();
            Box::new(move || {
                bus.unmap(&boot);
            })
        }));
        // Construct self
        Self { ctrl, boot }
    }

    /// Gets a read-only reference to the boot ROM.
    #[must_use]
    pub fn rom(&self) -> ReadOnly<impl Device<u16, u8>> {
        ReadOnly::from(self.boot.clone())
    }

    /// Gets a reference to the boot ROM's control register.
    #[must_use]
    pub fn ctrl(&self) -> Shared<Control> {
        self.ctrl.clone()
    }
}

impl Address<u16, u8> for Rom {
    fn read(&self, index: u16) -> u8 {
        self.boot.read(index)
    }

    fn write(&mut self, index: u16, value: u8) {
        self.boot.write(index, value);
    }
}

impl Block for Rom {
    fn reset(&mut self) {
        // Control
        self.ctrl.reset();
        // Memory
        self.boot.reset();
    }
}

impl Board<u16, u8> for Rom {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let boot = self.rom().to_dynamic();
        let ctrl = self.ctrl().to_dynamic();

        // Map devices on bus           // ┌──────┬────────┬──────────┬─────┐
                                        // │ Addr │  Size  │   Name   │ Dev │
                                        // ├──────┼────────┼──────────┼─────┤
        bus.map(0x0000..=0x00ff, boot); // │ 0000 │  256 B │ Boot     │ ROM │
        bus.map(0xff50..=0xff50, ctrl); // │ ff50 │    1 B │ Control  │ Reg │
                                        // └──────┴────────┴──────────┴─────┘
    }
}

impl Device<u16, u8> for Rom {}

/// Boot ROM [`Control`].
pub struct Control {
    write: bool,
    unmap: Box<dyn FnMut()>,
}

impl Control {
    /// Constructs a new `Control`.
    pub fn new(unmap: Box<dyn FnMut()>) -> Self {
        Self {
            write: false,
            unmap,
        }
    }
}

impl Address<u16, u8> for Control {
    fn read(&self, _: u16) -> u8 {
        self.load()
    }

    fn write(&mut self, _: u16, value: u8) {
        self.store(value);
    }
}

impl Block for Control {}

impl Cell<u8> for Control {
    fn load(&self) -> u8 {
        0xfe | u8::from(self.write)
    }

    fn store(&mut self, _: u8) {
        if !self.write {
            (self.unmap)();
        }
        self.write |= true;
    }
}

impl Debug for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Control").finish()
    }
}

impl Device<u16, u8> for Control {}
