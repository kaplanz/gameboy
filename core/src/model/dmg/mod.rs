//! DMG: [Game Boy].
//!
//! [Game Boy]: https://en.wikipedia.org/wiki/Game_Boy

use log::warn;
use rugby_arch::mio::Mmio;
use rugby_arch::reg::Port;
use rugby_arch::{Block, Word};

use self::apu::Apu;
use self::cpu::Cpu;
use self::joypad::Joypad;
use self::pcb::Motherboard;
use self::ppu::Ppu;
use self::serial::Serial;
use crate::api::core::{self, Core};
use crate::api::part::proc::Processor;

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
    main: Motherboard,
}

impl GameBoy {
    /// Constructs a new `GameBoy`.
    #[must_use]
    pub fn new() -> Self {
        let mut this = Self::default();

        // Simulate bootup sequence
        this.boot();

        this
    }

    /// Constructs a new `GameBoy`, initialized with the provided boot ROM.
    #[must_use]
    pub fn with(boot: Boot) -> Self {
        let mut this = Self::default();

        // Initialize boot ROM
        let boot = boot::Chip::new(boot);
        boot.attach(&mut this.main.noc.ibus.borrow_mut());
        this.boot = Some(boot);

        this
    }

    /// Simulate the bootup sequence.
    ///
    /// This prepares the `GameBoy` to run the contents of a game cartridge.
    /// When no [boot ROM](Boot) is installed, this must be called before
    /// cartridge execution.
    #[rustfmt::skip]
    pub fn boot(&mut self) {
        let cpu = &mut self.main.soc.cpu;

        // Initialize registers
        #[allow(clippy::items_after_statements)]
        type Select = <Cpu as Port<Word>>::Select;
        cpu.store(Select::AF, 0x01b0_u16);
        cpu.store(Select::BC, 0x0013_u16);
        cpu.store(Select::DE, 0x00d8_u16);
        cpu.store(Select::HL, 0x014d_u16);
        cpu.store(Select::SP, 0xfffe_u16);

        // Perform bootup sequence
        cpu.write(0xff40, 0x91); // enable display
        cpu.write(0xff50, 0x01); // disable boot ROM
        cpu.exec(0xfb);          // enable interrupts
        cpu.goto(0x0100);        // transfer program control
    }

    /// Gets the inserted game cartridge, if any.
    #[must_use]
    pub fn cart(&self) -> Option<&Cartridge> {
        self.cart.as_ref()
    }

    /// Inserts a game cartridge.
    ///
    /// If a cartridge is already inserted, it will first be
    /// [ejected](Self::eject).
    pub fn insert(&mut self, cart: Cartridge) {
        // Disconnect previous cartridge
        if let Some(cart) = self.eject() {
            warn!("ejected previous cartridge: {}", cart.header());
        };
        // Insert supplied cartridge
        let ebus = &mut *self.main.noc.ebus.borrow_mut();
        cart.attach(ebus);
        self.cart = Some(cart);
    }

    /// Ejects the inserted game cartridge, if any.
    pub fn eject(&mut self) -> Option<Cartridge> {
        // Disconnect from bus
        let ebus = &mut *self.main.noc.ebus.borrow_mut();
        if let Some(cart) = &self.cart {
            cart.detach(ebus);
        }
        // Remove inserted cartridge
        self.cart.take()
    }
}

impl Block for GameBoy {
    fn ready(&self) -> bool {
        self.main.ready()
    }

    fn cycle(&mut self) {
        self.main.cycle();
    }

    #[rustfmt::skip]
    fn reset(&mut self) {
        self.main.reset();
        self.boot.as_mut().map(Block::reset).unwrap_or_else(|| self.boot());
        self.cart.as_mut().map(Block::reset);
    }
}

impl Core for GameBoy {}

impl core::has::Audio for GameBoy {
    type Audio = Apu;

    fn audio(&self) -> &Self::Audio {
        &self.main.soc.apu
    }

    fn audio_mut(&mut self) -> &mut Self::Audio {
        &mut self.main.soc.apu
    }
}

impl core::has::Processor for GameBoy {
    type Proc = Cpu;

    fn proc(&self) -> &Self::Proc {
        &self.main.soc.cpu
    }

    fn proc_mut(&mut self) -> &mut Self::Proc {
        &mut self.main.soc.cpu
    }
}

impl core::has::Joypad for GameBoy {
    type Joypad = Joypad;

    fn joypad(&self) -> &Self::Joypad {
        &self.main.soc.joy
    }

    fn joypad_mut(&mut self) -> &mut Self::Joypad {
        &mut self.main.soc.joy
    }
}

impl core::has::Serial for GameBoy {
    type Serial = Serial;

    fn serial(&self) -> &Self::Serial {
        &self.main.soc.ser
    }

    fn serial_mut(&mut self) -> &mut Self::Serial {
        &mut self.main.soc.ser
    }
}

impl core::has::Video for GameBoy {
    type Video = Ppu;

    fn video(&self) -> &Self::Video {
        &self.main.soc.ppu
    }

    fn video_mut(&mut self) -> &mut Self::Video {
        &mut self.main.soc.ppu
    }
}

/// Debug introspection.
#[cfg(feature = "debug")]
impl GameBoy {
    /// Borrows the core's system-on-chip.
    #[must_use]
    pub fn chip(&self) -> &Chip {
        &self.main.soc
    }

    /// Mutably borrows the core's system-on-chip.
    pub fn chip_mut(&mut self) -> &mut Chip {
        &mut self.main.soc
    }
}

#[cfg(test)]
mod tests;
