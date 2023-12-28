//! DMG-01: [Game Boy]
//!
//! [Game Boy]: https://en.wikipedia.org/wiki/Game_Boy

use std::cell::{Ref, RefMut};

use log::warn;
use remus::bus::Mux;
use remus::dev::Device;
use remus::mem::Ram;
use remus::{Block, Board, Location, Machine, Shared};

use self::cpu::Cpu;
use self::dbg::Doctor;
use self::noc::NoC;
use self::ppu::Vram;
use self::soc::SoC;
use crate::dmg::cpu::Processor;
use crate::emu::Emulator;
use crate::hw::cart::Cartridge;
use crate::hw::joypad::Joypad;
use crate::hw::pic::Pic;
use crate::hw::ppu::Ppu;
use crate::hw::serial::Serial;
use crate::hw::timer::Timer;

mod boot;
mod noc;
mod soc;

pub mod dbg;

pub use self::boot::Boot;
pub use crate::emu::Screen as Dimensions;
pub use crate::hw::cpu::sm83 as cpu;
pub use crate::hw::joypad::Button;
pub use crate::hw::ppu::Screen;
pub use crate::hw::{cart, pic, ppu, serial, timer};

pub type Wram = Ram<u8, 0x2000>;
pub type Hram = Ram<u8, 0x007f>;

/// DMG-01 screen specification.
pub const SCREEN: Dimensions = Dimensions {
    width: 160,
    height: 144,
};

/// DMG-01 Game Boy emulator.
#[derive(Debug)]
pub struct GameBoy {
    // State
    clock: u128,
    // Systems
    soc: SoC,
    // Memory
    wram: Shared<Wram>,
    vram: Shared<Vram>,
    // Peripherals
    joypad: Joypad,
    serial: Serial,
    timer: Timer,
    // External
    cart: Option<Cartridge>,
    // Shared
    noc: NoC,
    pic: Shared<Pic>,
    // Debug
    doc: Option<Doctor>,
}

impl Default for GameBoy {
    fn default() -> Self {
        // Construct shared blocks
        let vram = Shared::new(Vram::default());
        let noc = NoC::default();
        let pic = Shared::from(Pic::default());

        // Construct self
        Self {
            // State
            clock: u128::default(),
            // Systems
            soc: SoC::new(vram.clone(), &noc, pic.clone()),
            // Memory
            wram: Shared::new(Wram::default()),
            vram,
            // Peripherals
            joypad: Joypad::new(pic.clone()),
            serial: Serial::new(pic.clone()),
            timer: Timer::new(pic.clone()),
            // External
            cart: Option::default(),
            // Shared
            noc,
            pic,
            // Debug
            doc: Option::default(),
        }
    }
}

impl GameBoy {
    /// Constructs a new `GameBoy`.
    ///
    /// The returned instance will be fully set-up for emulation to begin
    /// without further setup.
    ///
    /// Note that [`Cartridge`]s must be manually loaded with [`Self::load`].
    #[must_use]
    pub fn new() -> Self {
        Self::default().setup().boot()
    }

    /// Constructs a new `GameBoy` using the provided boot ROM.
    ///
    /// The returned instance will be fully set-up for emulation to begin
    /// without further setup.
    ///
    /// Note that [`Cartridge`]s must be manually loaded with [`Self::load`].
    #[must_use]
    pub fn with(boot: Boot) -> Self {
        // Construct a default instance
        let mut this = Self::default();
        // Construct the managed boot ROM
        let ibus = this.noc.int.clone();
        let rom = boot::Rom::new(ibus, boot);
        // Set the boot ROM
        this.soc.boot = Some(rom);
        // Set up and return the instance
        this.setup()
    }

    /// Sets up the `GameBoy`, such that is it ready to be run.
    #[must_use]
    fn setup(mut self) -> Self {
        // Map devices onto internal buses
        self.connect();

        self
    }

    #[rustfmt::skip]
    fn connect(&mut self) {
        // Extract buses
        let ibus = &mut *self.noc.int.borrow_mut();
        let ebus = &mut *self.noc.ext.borrow_mut();
        let vbus = &mut *self.noc.vid.borrow_mut();

        // Extract devices
        let vram = self.vram.clone().to_dynamic();
        let wram = self.wram.clone().to_dynamic();
        let echo = self.wram.clone().to_dynamic();

        // Map devices on bus            // ┌──────┬────────┬────────────┬─────┐
                                         // │ Addr │  Size  │    Name    │ Dev │
                                         // ├──────┼────────┼────────────┼─────┤
        // mapped by `soc`               // │ 0000 │  256 B │ Boot       │ ROM │
        // mapped by `cart`              // │ 0000 │ 32 KiB │ Cartridge  │ ROM │
        vbus.map(0x8000..=0x9fff, vram); // │ 8000 │  8 KiB │ Video      │ RAM │
        // mapped by `cart`              // │ a000 │  8 KiB │ External   │ RAM │
        ebus.map(0xc000..=0xdfff, wram); // │ c000 │  8 KiB │ Work       │ RAM │
        ebus.map(0xe000..=0xffff, echo); // │ e000 │ 7680 B │ Echo       │ RAM │
        // mapped by `ppu`               // │ fe00 │  160 B │ Object     │ RAM │
                                         // │ fea0 │   96 B │ Unmapped   │ --- │
        // mapped by `joypad`            // │ ff00 │    1 B │ Controller │ Reg │
        // mapped by `serial`            // │ ff01 │    2 B │ Serial     │ Reg │
                                         // │ ff03 │    1 B │ Unmapped   │ --- │
        // mapped by `timer`             // │ ff04 │    4 B │ Timer      │ Reg │
                                         // │ ff08 │    7 B │ Unmapped   │ --- │
        // mapped by `pic`               // │ ff0f │    1 B │ Interrupt  │ Reg │
        // mapped by `apu`               // │ ff10 │   23 B │ Audio      │ APU │
                                         // │ ff27 │    9 B │ Unmapped   │ --- │
        // mapped by `apu`               // │ ff30 │   16 B │ Waveform   │ RAM │
        // mapped by `ppu`               // │ ff40 │   12 B │ LCD        │ PPU │
                                         // │ ff4c │    4 B │ Unmapped   │ --- │
        // mapped by `soc`               // │ ff50 │    1 B │ Boot       │ Reg │
                                         // │ ff51 │   47 B │ Unmapped   │ --- │
        // mapped by `soc`               // │ ff80 │  127 B │ High       │ RAM │
        // mapped by `pic`               // │ ffff │    1 B │ Interrupt  │ Reg │
                                         // └──────┴────────┴────────────┴─────┘

        // Systems
        self.soc.connect(ibus);
        // Peripherals
        self.joypad.connect(ibus);
        self.serial.connect(ibus);
        self.timer.connect(ibus);
        // Shared
        self.pic.connect(ibus);
    }

    /// Simulate booting for `GameBoy`s with no [`Cartridge`].
    #[must_use]
    fn boot(mut self) -> Self {
        type Register = <Cpu as Location<u16>>::Register;

        // Execute setup code
        self.soc.cpu.exec(0xfb); // EI ; enable interrupts

        // Initialize registers
        self.soc.cpu.store(Register::AF, 0x01b0_u16);
        self.soc.cpu.store(Register::BC, 0x0013_u16);
        self.soc.cpu.store(Register::DE, 0x00d8_u16);
        self.soc.cpu.store(Register::HL, 0x014d_u16);
        self.soc.cpu.store(Register::SP, 0xfffe_u16);

        // Move the PC to the ROM's start
        self.soc.cpu.goto(0x0100);

        // Enable the LCD
        self.soc.cpu.write(0xff40, 0x91);
        // Disable the boot ROM
        self.soc.cpu.write(0xff50, 0x01);

        self
    }

    /// Loads a game [`Cartridge`] into the `GameBoy`
    ///
    /// If a cartridge has already been loaded, it will be disconnected and
    /// replaced.
    pub fn load(&mut self, cart: Cartridge) {
        // Disconnect cartridge from the bus
        let ebus = &mut *self.noc.ext.borrow_mut();
        if let Some(cart) = &self.cart {
            cart.disconnect(ebus);
        }
        // Connect the supplied cartridge
        cart.connect(ebus);
        self.cart = Some(cart);
    }

    /// Returns debug information about the model.
    #[must_use]
    pub fn debug(&mut self) -> dbg::Debug {
        dbg::Debug::new(self)
    }

    /// (Re)sets introspection with [Gameboy Doctor][gbdoc].
    ///
    /// # Note
    ///
    /// Any uncollected logs will be lost.
    ///
    /// [gbdoc]: https://robertheaton.com/gameboy-doctor
    pub fn doctor(&mut self, enable: bool) {
        self.doc = enable.then(Doctor::default);
    }

    /// Gets the `GameBoy`'s CPU.
    #[must_use]
    pub fn cpu(&self) -> &Cpu {
        &self.soc.cpu
    }

    /// Mutably gets the `GameBoy`'s CPU.
    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.soc.cpu
    }

    /// Gets the `GameBoy`'s programmable interrupt controller.
    #[must_use]
    pub fn pic(&self) -> Ref<Pic> {
        self.pic.borrow()
    }

    /// Mutably gets the `GameBoy`'s programmable interrupt controller.
    pub fn pic_mut(&mut self) -> RefMut<Pic> {
        self.pic.borrow_mut()
    }

    /// Gets the `GameBoy`'s PPU.
    #[must_use]
    pub fn ppu(&self) -> &Ppu {
        &self.soc.ppu
    }

    /// Mutably gets the `GameBoy`'s PPU.
    pub fn ppu_mut(&mut self) -> &mut Ppu {
        &mut self.soc.ppu
    }

    /// Gets the `GameBoy`'s serial.
    #[must_use]
    pub fn serial(&self) -> &Serial {
        &self.serial
    }

    /// Mutably gets the `GameBoy`'s serial.
    pub fn serial_mut(&mut self) -> &mut Serial {
        &mut self.serial
    }

    /// Gets the `GameBoy`'s timer.
    #[must_use]
    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    /// Mutably gets the `GameBoy`'s timer.
    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
    }
}

impl Block for GameBoy {
    #[rustfmt::skip]
    fn reset(&mut self) {
        // Systems
        self.soc.reset();
        // Memory
        self.wram.reset();
        self.vram.reset();
        // Peripherals
        self.joypad.reset();
        self.serial.reset();
        self.timer.reset();
        // External
        if let Some(cart) = &mut self.cart {
            cart.reset();
        }
        // Shared
        self.noc.int.reset();
        self.noc.ext.reset();
        self.noc.vid.reset();
        self.pic.reset();
    }
}

impl Emulator for GameBoy {
    type Input = Button;

    type Screen = Screen;

    fn send(&mut self, keys: &[Self::Input]) {
        self.joypad.input(keys);
    }

    fn redraw(&self, mut callback: impl FnMut(&Screen)) {
        if self.soc.ppu.ready() {
            callback(self.soc.ppu.screen());
        }
    }
}

impl Machine for GameBoy {
    fn enabled(&self) -> bool {
        self.soc.cpu.enabled()
    }

    fn cycle(&mut self) {
        // CPU: 1 MiHz
        if self.clock % 4 == 0 {
            // Wake on pending interrupt
            if !self.soc.cpu.enabled() && self.pic.borrow().int().is_some() {
                self.soc.cpu.wake();
            }
            // Collect doctor entries
            if let Some(doc) = &mut self.doc {
                if let Some(entry) = self.soc.cpu.doctor() {
                    doc.0.push(entry);
                }
            }
            // Cycle CPU
            if self.soc.cpu.enabled() {
                self.soc.cpu.cycle();
            }
        }

        // DMA: 1 MiHz
        if self.clock % 4 == 0 && self.soc.dma.enabled() {
            self.soc.dma.cycle();
        }

        // PPU: 4 MiHz
        if self.soc.ppu.enabled() {
            self.soc.ppu.cycle();
        }

        // Serial: 8192 Hz
        if self.clock % 0x200 == 0 && self.serial.enabled() {
            self.serial.cycle();
        }

        // Timer: 4 MiHz
        if self.timer.enabled() {
            self.timer.cycle();
        }

        // Update executed cycle count
        let carry;
        (self.clock, carry) = self.clock.overflowing_add(1);
        if carry {
            warn!("internal cycle counter overflowed; resetting");
        }
    }
}

#[cfg(test)]
mod tests {
    use remus::Address;

    use super::*;

    /// Sample boot ROM.
    const BOOT: &[u8; 0x0100] = include_bytes!("../../../../roms/boot/sameboy/dmg_boot.bin");

    /// Sample ROM header.
    const GAME: &[u8; 0x8000] = include_bytes!("../../../../roms/games/2048/2048.gb");

    fn setup() -> GameBoy {
        // Instantiate a `Boot`
        let boot = Boot::from(BOOT);
        // Instantiate a `Cartridge`
        let cart = Cartridge::new(GAME).unwrap();
        // Create a `GameBoy` instance
        let mut emu = GameBoy::with(boot);
        // Load the cartridge into the emulator
        emu.load(cart);

        emu
    }

    #[test]
    fn boot_disable_works() {
        let mut emu = setup();
        let bus = &mut emu.soc.cpu;

        // Ensure boot ROM starts enabled:
        // - Perform comparison against boot ROM contents
        (0x0000..=0x0100)
            .map(|addr| bus.read(addr))
            .zip(BOOT)
            .for_each(|(read, &rom)| assert_eq!(read, rom));

        // Disable boot ROM
        bus.write(0xff50, 0x01);

        // Check if disable was successful:
        // - Perform comparison against cartridge ROM contents
        (0x0000..=0x0100)
            .map(|addr| bus.read(addr))
            .zip(GAME)
            .for_each(|(read, &rom)| assert_eq!(read, rom));
    }

    #[test]
    fn bus_all_works() {
        let mut emu = setup();
        let bus = &mut emu.soc.cpu;

        // Boot ROM
        (0x0000..=0x00ff)
            .map(|addr| emu.soc.boot.as_ref().unwrap().rom().read(addr))
            .any(|byte| byte != 0x01);
        // Cartridge ROM
        if let Some(cart) = &emu.cart {
            (0x0100..=0x7fff).for_each(|addr| bus.write(addr, 0x02));
            assert!((0x0100..=0x7fff)
                .map(|addr| cart.rom().read(addr))
                .any(|byte| byte != 0x02));
        }
        // Video RAM
        (0x8000..=0x9fff).for_each(|addr| bus.write(addr, 0x03));
        (0x0000..=0x1fff)
            .map(|addr: u16| emu.soc.ppu.vram().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x03));
        // External RAM
        if let Some(cart) = &emu.cart {
            (0xa000..=0xbfff).for_each(|addr| bus.write(addr, 0x04));
            (0x0000..=0x1fff) // NOTE: External RAM is disabled for this ROM
                .map(|addr| cart.ram().read(addr))
                .for_each(|byte| assert_eq!(byte, 0xff));
        }
        // Object RAM
        (0xfe00..=0xfe9f).for_each(|addr| bus.write(addr, 0x05));
        (0x0000..=0x009f)
            .map(|addr: u16| emu.soc.ppu.oam().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x05));
        // Controller
        (0xff00..=0xff00).for_each(|addr| bus.write(addr, 0x60));
        (0x0000..=0x0000) // NOTE: Only bits 0x30 are writable
            .map(|addr| emu.joypad.con().read(addr))
            .for_each(|byte| assert_eq!(byte, 0xef));
        // Serial
        (0xff01..=0xff03).for_each(|addr| bus.write(addr, 0x07));
        (0x0000..=0x0002)
            .map(|_| 0x07) // FIXME
            .for_each(|byte| assert_eq!(byte, 0x07));
        // Timer
        (0xff04..=0xff07).for_each(|addr| bus.write(addr, 0x08));
        (0x0000..=0x0003)
            .zip([
                <Timer as Location<u8>>::Register::Div,
                <Timer as Location<u8>>::Register::Tima,
                <Timer as Location<u8>>::Register::Tma,
                <Timer as Location<u8>>::Register::Tac,
            ])
            .map(|(_, reg)| emu.timer.load(reg))
            .zip([0x00, 0x08, 0x08, 0x00])
            .for_each(|(found, expected)| assert_eq!(found, expected));
        // Interrupt Flag
        (0xff0f..=0xff0f).for_each(|addr| bus.write(addr, 0x09));
        (0x0000..=0x0000)
            .map(|_| emu.pic.load(<Pic as Location<u8>>::Register::If))
            .for_each(|byte| assert_eq!(byte, 0xe9));
        // Audio
        (0xff10..=0xff27).for_each(|addr| bus.write(addr, 0x0a));
        (0x0000..=0x0017)
            .map(|_| 0x0a) // FIXME
            .for_each(|byte| assert_eq!(byte, 0x0a));
        // Waveform RAM
        (0xff30..=0xff3f).for_each(|addr| bus.write(addr, 0x0b));
        (0x0000..=0x000f)
            .map(|_| 0x0b) // FIXME
            .for_each(|byte| assert_eq!(byte, 0x0b));
        // LCD
        (0xff40..=0xff4b).for_each(|addr| bus.write(addr, 0x0c));
        (0x0000..=0x000b)
            .zip([
                <Ppu as Location<u8>>::Register::Lcdc,
                <Ppu as Location<u8>>::Register::Stat,
                <Ppu as Location<u8>>::Register::Scy,
                <Ppu as Location<u8>>::Register::Scx,
                <Ppu as Location<u8>>::Register::Ly,
                <Ppu as Location<u8>>::Register::Lyc,
                <Ppu as Location<u8>>::Register::Dma,
                <Ppu as Location<u8>>::Register::Bgp,
                <Ppu as Location<u8>>::Register::Obp0,
                <Ppu as Location<u8>>::Register::Obp1,
                <Ppu as Location<u8>>::Register::Wy,
                <Ppu as Location<u8>>::Register::Wx,
            ])
            .map(|(_, reg)| emu.soc.ppu.load(reg))
            .for_each(|byte| assert_eq!(byte, 0x0c));
        // Boot ROM Disable
        (0xff50..=0xff50).for_each(|addr| bus.write(addr, 0x0d));
        (0x0000..=0x0000)
            .map(|addr| emu.soc.boot.as_ref().unwrap().ctrl().read(addr))
            .for_each(|byte| assert_eq!(byte, 0xff));
        // High RAM
        (0xff80..=0xfffe).for_each(|addr| bus.write(addr, 0x0e));
        (0x0000..=0x007e)
            .map(|addr: u16| emu.soc.hram.read(addr))
            .for_each(|byte| assert_eq!(byte, 0x0e));
        // Interrupt Enable
        (0xffff..=0xffff).for_each(|addr| bus.write(addr, 0x0f));
        (0x0000..=0x0000)
            .map(|_| emu.pic.load(<Pic as Location<u8>>::Register::Ie))
            .for_each(|byte| assert_eq!(byte, 0xef));
    }

    #[test]
    fn bus_unmapped_works() {
        let mut emu = setup();
        let bus = &mut emu.soc.cpu;

        // Define unmapped addresses
        let unmapped = [0xfea0..=0xfeff, 0xff03..=0xff03, 0xff27..=0xff2f];

        // Test unmapped addresses
        for gap in unmapped {
            for addr in gap {
                // Write to every unmapped address
                bus.write(addr, 0xaa);
                // Check the write didn't work
                assert_eq!(bus.read(addr), 0xff, "{addr:#06x}");
            }
        }
    }
}
