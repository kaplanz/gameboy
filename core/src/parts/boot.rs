//! Boot ROM.

use std::fmt::Debug;

use log::{debug, trace};
use rugby_arch::mem::{Error, Memory, Result};
use rugby_arch::mio::{Bus, Mmio};
use rugby_arch::reg::Register;
use rugby_arch::{Block, Byte, Shared, Word};

/// Boot ROM.
pub type Boot = rugby_arch::mem::Rom<[Byte; 0x100]>;

/// Boot mapper chip.
#[derive(Clone, Debug)]
pub struct Chip {
    /// Boot disable.
    pub reg: Shared<Control>,
    /// Boot bank.
    pub mem: Shared<Bank>,
}

impl Chip {
    /// Constructs a new `Rom`.
    #[must_use]
    pub fn new(rom: Boot) -> Self {
        trace!(
            "BOOT:\n{rom}",
            rom = phex::Printer::<Byte>::new(0, rom.inner())
        );
        let reg = Shared::new(Control::new());
        Self {
            mem: Shared::new(Bank::new(reg.clone(), rom)),
            reg,
        }
    }
}

impl Block for Chip {
    fn reset(&mut self) {
        self.reg.reset();
    }
}

impl Mmio for Chip {
    fn attach(&self, bus: &mut Bus) {
        bus.map(0x0000..=0x00ff, self.mem.clone().into());
        bus.map(0xff50..=0xff50, self.reg.clone().into());
    }
}

/// Boot disable register.
#[derive(Debug, Default)]
pub struct Control(bool);

impl Control {
    /// Constructs a new `Control`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for Control {
    fn reset(&mut self) {
        std::mem::take(&mut self.0);
    }
}

impl Memory for Control {
    fn read(&self, _: Word) -> Result<Byte> {
        Ok(self.load())
    }

    fn write(&mut self, _: Word, value: Byte) -> Result<()> {
        self.store(value);
        Ok(())
    }
}

impl Register for Control {
    type Value = Byte;

    fn load(&self) -> Self::Value {
        0xfe | Byte::from(self.0)
    }

    #[rustfmt::skip]
    fn store(&mut self, value: Self::Value) {
        let enabled = !self.0;            // is boot enabled?
        let disable = value & 0x01 != 0;  // disable request?

        // Disable cannot be undone (other than a reset). This is implemented in
        // the hardware through a feedback loop on this register's output back
        // to its input via an OR gate.
        self.0 |= disable;
        if enabled && disable {
            debug!("disabled boot");
        }
    }
}
/// Boot memory bank.
#[derive(Debug)]
pub struct Bank {
    /// Boot ROM.
    pub boot: Boot,
    /// Boot disable.
    reg: Shared<Control>,
}

impl Bank {
    /// Constructs a new `Rom`.
    fn new(reg: Shared<Control>, boot: Boot) -> Self {
        Self { boot, reg }
    }
}

impl Block for Bank {
    fn ready(&self) -> bool {
        self.reg.load() & 0x01 == 0
    }
}

impl Memory for Bank {
    fn read(&self, addr: Word) -> Result<Byte> {
        if self.ready() {
            self.boot.read(addr)
        } else {
            Err(Error::Busy)
        }
    }

    fn write(&mut self, addr: Word, data: Byte) -> Result<()> {
        if self.ready() {
            self.boot.write(addr, data)
        } else {
            Err(Error::Busy)
        }
    }
}
