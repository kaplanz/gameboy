//! DMG: [Game Boy].
//!
//! [Game Boy]: https://en.wikipedia.org/wiki/Game_Boy

use log::warn;
use remus::mio::Mmio;
use remus::reg::Port;
use remus::{Block, Word};

use self::apu::Apu;
use self::cpu::Cpu;
use self::joypad::Joypad;
use self::pcb::Motherboard;
use self::pic::Pic;
use self::ppu::Ppu;
use self::serial::Serial;
use self::timer::Timer;
use crate::api::proc::Processor;
use crate::api::{self, Core};

mod noc;
mod soc;

#[cfg(feature = "debug")]
pub mod dbg;
pub mod mem;
pub mod pcb;

pub use self::boot::Boot;
pub use self::cart::Cartridge;
pub use self::joypad::Button;
pub use self::noc::Mmap;
pub use self::soc::Chip;
pub use crate::parts::cpu::sm83 as cpu;
pub use crate::parts::{apu, boot, cart, dma, joypad, pic, ppu, serial, timer};

/// Clock frequency.
///
/// Crystal oscillator frequency of 4 KiHz.
#[allow(clippy::doc_markdown)]
pub const FREQ: u32 = 4_194_304;

pub use self::ppu::LCD;

/// Game Boy handheld game console.
#[derive(Debug, Default)]
pub struct GameBoy {
    /// Boot ROM.
    boot: Option<boot::Chip>,
    /// Game cartridge.
    cart: Option<Cartridge>,
    /// DMG-01 Motherboard.
    pcb: Motherboard,
}

impl GameBoy {
    /// Constructs a new `GameBoy`.
    ///
    /// # Note
    ///
    /// In order to insert a [`Cartridge`], it must be separately
    /// [loaded][`api::cart::Support::load`].
    #[must_use]
    pub fn new() -> Self {
        Self::default().prep().boot()
    }

    /// Constructs a new `GameBoy`, initialized with the provided boot ROM.
    ///
    /// # Note
    ///
    /// In order to insert a [`Cartridge`], it must be separately
    /// [loaded][`api::cart::Support::load`].
    #[must_use]
    pub fn with(boot: Boot) -> Self {
        Self {
            boot: Some(boot::Chip::new(boot)),
            ..Default::default()
        }
        .prep()
    }

    /// Prepares a `GameBoy` such that it can readily be used.
    #[must_use]
    fn prep(mut self) -> Self {
        self.attach();
        self
    }

    /// Simulate booting for `GameBoy`s with no [`Cartridge`].
    #[must_use]
    #[rustfmt::skip]
    fn boot(mut self) -> Self {
        // Initialize registers
        type Select = <Cpu as Port<Word>>::Select;
        self.pcb.soc.cpu.store(Select::AF, 0x01b0_u16);
        self.pcb.soc.cpu.store(Select::BC, 0x0013_u16);
        self.pcb.soc.cpu.store(Select::DE, 0x00d8_u16);
        self.pcb.soc.cpu.store(Select::HL, 0x014d_u16);
        self.pcb.soc.cpu.store(Select::SP, 0xfffe_u16);
        self.pcb.soc.cpu.write(0xff40, 0x91); // enable LCD
        self.pcb.soc.cpu.write(0xff50, 0x01); // disable boot
        // Execute necessary setup
        self.pcb.soc.cpu.exec(0xfb); // EI ; enable interrupts
        // Hand-off program control
        self.pcb.soc.cpu.goto(0x0100);
        self
    }

    /// Connect memory-mapped I/O to the network-on-chip.
    fn attach(&mut self) {
        // Borrow network
        let Mmap { ibus, ebus, .. } = &self.pcb.noc;
        let ibus = &mut *ibus.borrow_mut();
        let ebus = &mut *ebus.borrow_mut();
        // Attach modules
        if let Some(boot) = self.boot.as_mut() {
            boot.attach(ibus);
        }
        if let Some(cart) = self.cart.as_mut() {
            cart.attach(ebus);
        }
    }
}

impl Block for GameBoy {
    fn ready(&self) -> bool {
        self.pcb.ready()
    }

    fn cycle(&mut self) {
        self.pcb.cycle();
    }

    fn reset(&mut self) {
        self.boot.as_mut().map(Block::reset);
        self.cart.as_mut().map(Block::reset);
        self.pcb.reset();
    }
}

impl Core for GameBoy {}

impl api::audio::Support for GameBoy {
    type Audio = Apu;

    fn audio(&self) -> &Self::Audio {
        &self.pcb.soc.apu
    }

    fn audio_mut(&mut self) -> &mut Self::Audio {
        &mut self.pcb.soc.apu
    }
}

impl api::cart::Support for GameBoy {
    /// Game ROM cartridge.
    type Cartridge = Cartridge;

    /// Loads a game [`Cartridge`] into the [`GameBoy`].
    ///
    /// If a cartridge has already been loaded, it will be disconnected and
    /// replaced.
    fn load(&mut self, cart: Self::Cartridge) {
        // Disconnect previous cartridge
        if let Some(cart) = self.eject() {
            warn!("ejected previous cartridge: {}", cart.header());
        };
        // Insert supplied cartridge
        let ebus = &mut *self.pcb.noc.ebus.borrow_mut();
        cart.attach(ebus);
        self.cart = Some(cart);
    }

    fn cart(&mut self) -> Option<&Self::Cartridge> {
        self.cart.as_ref()
    }

    fn eject(&mut self) -> Option<Self::Cartridge> {
        // Disconnect from bus
        let ebus = &mut *self.pcb.noc.ebus.borrow_mut();
        if let Some(cart) = &self.cart {
            cart.detach(ebus);
        }
        // Remove inserted cartridge
        self.cart.take()
    }
}

impl api::proc::Support for GameBoy {
    type Proc = Cpu;

    fn cpu(&self) -> &Self::Proc {
        &self.pcb.soc.cpu
    }

    fn cpu_mut(&mut self) -> &mut Self::Proc {
        &mut self.pcb.soc.cpu
    }
}

impl api::joypad::Support for GameBoy {
    type Joypad = Joypad;

    fn joypad(&self) -> &Self::Joypad {
        &self.pcb.soc.joy
    }

    fn joypad_mut(&mut self) -> &mut Self::Joypad {
        &mut self.pcb.soc.joy
    }
}

impl api::serial::Support for GameBoy {
    type Serial = Serial;

    fn serial(&self) -> &Self::Serial {
        &self.pcb.soc.ser
    }

    fn serial_mut(&mut self) -> &mut Self::Serial {
        &mut self.pcb.soc.ser
    }
}

impl api::video::Support for GameBoy {
    type Video = Ppu;

    fn video(&self) -> &Self::Video {
        &self.pcb.soc.ppu
    }

    fn video_mut(&mut self) -> &mut Self::Video {
        &mut self.pcb.soc.ppu
    }
}

impl GameBoy {
    /// Gets the core's interrupt controller.
    #[must_use]
    pub fn pic(&self) -> &Pic {
        &self.pcb.soc.pic
    }

    /// Mutably gets the core's interrupt controller.
    pub fn pic_mut(&mut self) -> &mut Pic {
        &mut self.pcb.soc.pic
    }

    /// Gets the core's timer.
    #[must_use]
    pub fn timer(&self) -> &Timer {
        &self.pcb.soc.tma
    }

    /// Mutably gets the core's timer.
    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.pcb.soc.tma
    }
}

#[cfg(test)]
mod tests;
