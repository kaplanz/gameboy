//! DMG-01: [Game Boy]
//!
//! [Game Boy]: https://en.wikipedia.org/wiki/Game_Boy

use std::cell::RefCell;
use std::rc::Rc;

use remus::bus::Bus;
use remus::{Block, Device, Machine};

use self::mem::Memory;
use crate::dev::Unmapped;
use crate::emu::{screen, Emulator};
use crate::hw::apu::Apu;
use crate::hw::cart::Cartridge;
use crate::hw::cpu::{Processor, Sm83 as Cpu};
use crate::hw::joypad::Joypad;
use crate::hw::pic::Pic;
use crate::hw::ppu::{self, Ppu};
use crate::hw::serial::Serial;
use crate::hw::timer::Timer;

mod boot;
mod mem;

pub use boot::Rom as BootRom;

pub trait Board {
    fn connect(&self, bus: &mut Bus);

    fn disconnect(&self, _bus: &mut Bus) {}
}

pub use crate::hw::cart;
pub use crate::hw::joypad::Button;
pub use crate::hw::ppu::Screen;

/// Screen info.
pub const SCREEN: screen::Info = screen::Info {
    width: 160,
    height: 144,
};

/// DMG-01 Game Boy emulator.
#[derive(Debug, Default)]
pub struct GameBoy {
    // State
    clock: usize,
    // Processors
    apu: Apu,
    cpu: Cpu,
    ppu: Ppu,
    // Peripherals
    joypad: Joypad,
    serial: Serial,
    timer: Timer,
    // Memory
    cart: Option<Cartridge>,
    mem: Memory,
    // Connections
    bus: Rc<RefCell<Bus>>,
    pic: Rc<RefCell<Pic>>,
}

impl GameBoy {
    /// Constructs a new `GameBoy`.
    ///
    /// The returned instance will be fully set-up for emulation to begin
    /// without further prior setup.
    #[must_use]
    pub fn new() -> Self {
        Self::default().setup().boot()
    }

    /// Constructs a new `GameBoy` using the provided boot ROM.
    ///
    /// The returned instance will be fully set-up for emulation to begin
    /// without further prior setup.
    #[must_use]
    pub fn with(boot: BootRom) -> Self {
        let mem = Memory::with(boot);
        Self {
            mem,
            ..Default::default()
        }
        .setup()
    }

    /// Loads a `Cartridge` into the `GameBoy`.
    pub fn load(&mut self, cart: Cartridge) {
        // Disconnect any connected cartridge from the bus
        let bus = &mut *self.bus.borrow_mut();
        if let Some(cart) = &self.cart {
            cart.disconnect(bus);
        }
        // Store and connect the supplied cartridge
        cart.connect(bus);
        self.cart = Some(cart);
    }

    /// Returns the VRAM's current state from the model.
    ///
    /// # Panics
    ///
    /// Cannot panic, as VRAM always has a fixed size.
    #[must_use]
    pub fn debug(&self) -> Debug {
        Debug {
            ppu: self.ppu.debug(),
        }
    }

    fn setup(mut self) -> Self {
        // Connect bus
        self.cpu.set_bus(self.bus.clone());
        self.ppu.set_bus(self.bus.clone());

        // Connect PIC
        self.cpu.set_pic(self.pic.clone());
        self.joypad.set_pic(self.pic.clone());
        self.ppu.set_pic(self.pic.clone());
        self.timer.set_pic(self.pic.clone());

        // Reset all devices
        self.reset();

        // Make connections
        self.connect(&mut self.bus.borrow_mut());

        self
    }

    fn boot(mut self) -> Self {
        type Register = <Cpu as Processor>::Register;

        // Execute setup code
        self.cpu.exec(0xfb); // ei ; enable interrupts

        // Initialize registers
        self.cpu.set(Register::AF, 0x01b0);
        self.cpu.set(Register::BC, 0x0013);
        self.cpu.set(Register::DE, 0x00d8);
        self.cpu.set(Register::HL, 0x014d);
        self.cpu.set(Register::SP, 0xfffe);

        // Move the PC to the ROM's start
        self.cpu.goto(0x0100);

        // Enable the LCD
        self.bus.borrow_mut().write(0xff40, 0x91);
        // Disable the boot ROM
        self.bus.borrow_mut().write(0xff50, 0x01);

        self
    }
}

impl Block for GameBoy {
    #[rustfmt::skip]
    fn reset(&mut self) {
        // Reset processors
        self.cpu.reset();
        self.ppu.reset();

        // Reset peripherals
        self.apu.reset();
        self.joypad.reset();
        self.serial.reset();
        self.timer.reset();

        // Reset memory
        if let Some(cart) = &mut self.cart {
            cart.reset();
        }
        self.mem.reset();

        // Reset connections
        self.bus.borrow_mut().reset();
        self.pic.borrow_mut().reset();
    }
}

impl Board for GameBoy {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Connect processors
        self.cpu.connect(bus);
        self.ppu.connect(bus);

        // Connect peripherals
        self.apu.connect(bus);
        self.joypad.connect(bus);
        self.serial.connect(bus);
        self.timer.connect(bus);

        // Connect memory
        if let Some(cart) = &self.cart {
            cart.connect(bus);
        }
        self.mem.connect(bus);

        // Connect connections
        self.pic.borrow().connect(bus);

        // Map devices on bus  // ┌──────┬────────┬────────────┬─────┐
                               // │ Addr │  Size  │    Name    │ Dev │
                               // ├──────┼────────┼────────────┼─────┤
        // mapped by `mem`     // │ 0000 │  256 B │ Boot       │ ROM │
        // mapped by `cart`    // │ 0000 │ 32 KiB │ Cartridge  │ ROM │
        // mapped by `ppu`     // │ 8000 │  8 KiB │ Video      │ RAM │
        // mapped by `cart`    // │ a000 │  8 KiB │ External   │ RAM │
        // mapped by `mem`     // │ c000 │  8 KiB │ Work       │ RAM │
        // mapped by `mem`     // │ e000 │ 7680 B │ Echo       │ RAM │
        // mapped by `ppu`     // │ fe00 │  160 B │ Object     │ RAM │
                               // │ fea0 │   96 B │ Unmapped   │ --- │
        // mapped by `joypad`  // │ ff00 │    1 B │ Controller │ Reg │
        // mapped by `serial`  // │ ff01 │    2 B │ Serial     │ Reg │
                               // │ ff03 │    1 B │ Unmapped   │ --- │
        // mapped by `timer`   // │ ff04 │    4 B │ Timer      │ Reg │
                               // │ ff08 │    7 B │ Unmapped   │ --- │
        // mapped by `pic`     // │ ff0f │    1 B │ Interrupt  │ Reg │
        // mapped by `apu`     // │ ff10 │   23 B │ Audio      │ APU │
                               // │ ff27 │    9 B │ Unmapped   │ --- │
        // mapped by `apu`     // │ ff30 │   16 B │ Waveform   │ RAM │
        // mapped by `ppu`     // │ ff40 │   12 B │ LCD        │ PPU │
                               // │ ff4c │    4 B │ Unmapped   │ --- │
        // mapped by `mem`     // │ ff50 │    1 B │ Boot       │ Reg │
                               // │ ff51 │   47 B │ Unmapped   │ --- │
        // mapped by `mem`     // │ ff80 │  127 B │ High       │ RAM │
        // mapped by `pic`     // │ ffff │    1 B │ Interrupt  │ Reg │
                               // └──────┴────────┴────────────┴─────┘

        // NOTE: use fallback to report invalid reads as `0xff`
        let unmap = Unmapped::<0x10000>::new().to_shared();
        bus.map(0x0000, unmap);
    }
}

impl Emulator for GameBoy {
    type Input = Button;

    type Screen = Screen;

    fn send(&mut self, keys: &[Self::Input]) {
        self.joypad.input(keys);
    }

    fn redraw(&self, mut callback: impl FnMut(&Screen)) {
        if self.ppu.ready() {
            callback(self.ppu.screen());
        }
    }
}

impl Machine for GameBoy {
    fn enabled(&self) -> bool {
        self.cpu.enabled()
    }

    fn cycle(&mut self) {
        // CPU runs on a 1 MiHz clock: implement using a simple clock divider
        if self.clock % 4 == 0 {
            // Wake disabled CPU if interrupts pending
            if !self.cpu.enabled() && self.pic.borrow().int().is_some() {
                self.cpu.wake();
            }

            // Cycle CPU if enabled
            if self.cpu.enabled() {
                self.cpu.cycle();
            }
        }

        // PPU runs on a 4 MiHz clock
        if self.ppu.enabled() {
            self.ppu.cycle();
        }

        // Timer runs on a 4 MiHz clock
        if self.timer.enabled() {
            self.timer.cycle();
        }

        // Keep track of cycles executed
        self.clock = self.clock.wrapping_add(1);
    }
}

#[derive(Debug)]
pub struct Debug {
    pub ppu: ppu::Debug,
}

#[cfg(test)]
mod tests {
    use remus::Device;

    use super::*;

    // Boot ROM contents.
    const BOOTROM: [u8; 0x100] = [
        0x31, 0xfe, 0xff, 0xaf, 0x21, 0xff, 0x9f, 0x32, 0xcb, 0x7c, 0x20, 0xfb, 0x21, 0x26, 0xff,
        0x0e, 0x11, 0x3e, 0x80, 0x32, 0xe2, 0x0c, 0x3e, 0xf3, 0xe2, 0x32, 0x3e, 0x77, 0x77, 0x3e,
        0xfc, 0xe0, 0x47, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1a, 0xcd, 0x95, 0x00, 0xcd, 0x96,
        0x00, 0x13, 0x7b, 0xfe, 0x34, 0x20, 0xf3, 0x11, 0xd8, 0x00, 0x06, 0x08, 0x1a, 0x13, 0x22,
        0x23, 0x05, 0x20, 0xf9, 0x3e, 0x19, 0xea, 0x10, 0x99, 0x21, 0x2f, 0x99, 0x0e, 0x0c, 0x3d,
        0x28, 0x08, 0x32, 0x0d, 0x20, 0xf9, 0x2e, 0x0f, 0x18, 0xf3, 0x67, 0x3e, 0x64, 0x57, 0xe0,
        0x42, 0x3e, 0x91, 0xe0, 0x40, 0x04, 0x1e, 0x02, 0x0e, 0x0c, 0xf0, 0x44, 0xfe, 0x90, 0x20,
        0xfa, 0x0d, 0x20, 0xf7, 0x1d, 0x20, 0xf2, 0x0e, 0x13, 0x24, 0x7c, 0x1e, 0x83, 0xfe, 0x62,
        0x28, 0x06, 0x1e, 0xc1, 0xfe, 0x64, 0x20, 0x06, 0x7b, 0xe2, 0x0c, 0x3e, 0x87, 0xe2, 0xf0,
        0x42, 0x90, 0xe0, 0x42, 0x15, 0x20, 0xd2, 0x05, 0x20, 0x4f, 0x16, 0x20, 0x18, 0xcb, 0x4f,
        0x06, 0x04, 0xc5, 0xcb, 0x11, 0x17, 0xc1, 0xcb, 0x11, 0x17, 0x05, 0x20, 0xf5, 0x22, 0x23,
        0x22, 0x23, 0xc9, 0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00, 0x83,
        0x00, 0x0c, 0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc, 0x6e,
        0xe6, 0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc,
        0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e, 0x3c, 0x42, 0xb9, 0xa5, 0xb9, 0xa5, 0x42, 0x3c, 0x21,
        0x04, 0x01, 0x11, 0xa8, 0x00, 0x1a, 0x13, 0xbe, 0x20, 0xfe, 0x23, 0x7d, 0xfe, 0x34, 0x20,
        0xf5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xfb, 0x86, 0x20, 0xfe, 0x3e, 0x01, 0xe0,
        0x50,
    ];

    // Test ROM contents.
    const HEADER: [u8; 0x150] = [
        0xc3, 0x8b, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc3, 0x8b, 0x02, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x87, 0xe1, 0x5f, 0x16, 0x00,
        0x19, 0x5e, 0x23, 0x56, 0xd5, 0xe1, 0xe9, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xc3, 0xfd, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xc3, 0x12, 0x27,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xc3, 0x12, 0x27, 0xff, 0xff, 0xff, 0xff, 0xff, 0xc3, 0x7e,
        0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0x00, 0xc3, 0x50, 0x01, 0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73,
        0x00, 0x83, 0x00, 0x0c, 0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e, 0xdc,
        0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc,
        0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02,
        0x01, 0x00, 0x00, 0xdc, 0x31, 0xbb,
    ];

    fn setup() -> GameBoy {
        // Instantiate a `BootRom`
        let boot = BootRom::try_from(&BOOTROM).unwrap();
        // Instantiate a `Cartridge`
        let cart = Cartridge::new(&HEADER).unwrap();
        // Create a `GameBoy` instance
        let mut emu = GameBoy::with(boot);
        // Load the cartridge into the emulator
        emu.load(cart);

        emu
    }

    #[test]
    fn boot_disable_works() {
        let emu = setup();

        // Ensure boot ROM starts enabled:
        // - Perform comparison against boot ROM contents
        (0x0000..=0x0100)
            .map(|addr| emu.bus.borrow().read(addr))
            .zip(BOOTROM)
            .for_each(|(read, rom)| assert_eq!(read, rom));

        // Disable boot ROM
        emu.bus.borrow_mut().write(0xff50, 0x01);

        // Check if disable was successful:
        // - Perform comparison against cartridge ROM contents
        (0x0000..=0x0100)
            .map(|addr| emu.bus.borrow().read(addr))
            .zip(HEADER)
            .for_each(|(read, rom)| assert_eq!(read, rom));
    }

    #[test]
    fn bus_all_works() {
        // NOTE: Test reads (and writes) for each component separately
        let emu = setup();

        // Boot ROM
        (0x0000..=0x00ff)
            .map(|addr| emu.mem.boot.borrow().rom().borrow().read(addr))
            .any(|byte| byte != 0x01);
        // Cartridge ROM
        if let Some(cart) = &emu.cart {
            (0x0100..=0x7fff).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x02));
            assert!((0x0100..=0x7fff)
                .map(|addr| cart.rom().borrow().read(addr))
                .any(|byte| byte != 0x02));
        }
        // Video RAM
        (0x8000..=0x9fff).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x03));
        (0x0000..=0x1fff)
            .map(|addr| emu.ppu.vram().borrow().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x03));
        // External RAM
        if let Some(cart) = &emu.cart {
            (0xa000..=0xbfff).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x04));
            (0x0000..=0x1fff)
                .map(|addr| cart.ram().borrow().read(addr))
                .for_each(|byte| assert_eq!(byte, 0x04));
        }
        // Object RAM
        (0xfe00..=0xfe9f).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x05));
        (0x0000..=0x009f)
            .map(|addr| emu.ppu.oam().borrow().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x05));
        // Controller
        (0xff00..=0xff00).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x60));
        (0x0000..=0x0000) // NOTE: Only bits 0x30 are writable
            .map(|addr| emu.joypad.con().borrow().read(addr))
            .for_each(|byte| assert_eq!(byte, 0xef));
        // Serial
        (0xff01..=0xff03).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x07));
        (0x0000..=0x0002)
            .map(|_| 0x07) // FIXME
            .for_each(|byte| assert_eq!(byte, 0x07));
        // Timer
        (0xff04..=0xff07).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x08));
        (0x0000..=0x0003)
            .zip([Timer::div, Timer::tima, Timer::tma, Timer::tac])
            .map(|(_, get)| get(&emu.timer).borrow().read(0))
            .for_each(|byte| assert_eq!(byte, 0x08));
        // Interrupt Active
        (0xff0f..=0xff0f).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x09));
        (0x0000..=0x0000)
            .map(|addr| emu.pic.borrow().active().borrow().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x09));
        // Audio
        (0xff10..=0xff27).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x0a));
        (0x0000..=0x0017)
            .map(|_| 0x0a) // FIXME
            .for_each(|byte| assert_eq!(byte, 0x0a));
        // Waveform RAM
        (0xff30..=0xff3f).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x0b));
        (0x0000..=0x000f)
            .map(|_| 0x0b) // FIXME
            .for_each(|byte| assert_eq!(byte, 0x0b));
        // LCD
        (0xff40..=0xff4b).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x0c));
        (0x0000..=0x000b)
            .zip([
                Ppu::lcdc,
                Ppu::stat,
                Ppu::scy,
                Ppu::scx,
                Ppu::ly,
                Ppu::lyc,
                Ppu::dma,
                Ppu::bgp,
                Ppu::obp0,
                Ppu::obp1,
                Ppu::wy,
                Ppu::wx,
            ])
            .map(|(_, get)| get(&emu.ppu).borrow().read(0))
            .for_each(|byte| assert_eq!(byte, 0x0c));
        // Boot ROM Disable
        (0xff50..=0xff50).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x0d));
        (0x0000..=0x0000)
            .map(|addr| emu.mem.boot.borrow().disable().borrow().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x0d));
        // High RAM
        (0xff80..=0xfffe).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x0e));
        (0x0000..=0x007e)
            .map(|addr| emu.mem.hram.borrow().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x0e));
        // Interrupt Enable
        (0xffff..=0xffff).for_each(|addr| emu.bus.borrow_mut().write(addr, 0x0f));
        (0x0000..=0x0000)
            .map(|addr| emu.pic.borrow().enable().borrow().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x0f));
    }

    #[test]
    #[should_panic]
    fn bus_boot_write_panics() {
        let emu = setup();

        // Write to boot ROM (should panic)
        emu.bus.borrow_mut().write(0x0000, 0xaa);
    }

    #[test]
    fn bus_unmapped_works() {
        let emu = setup();

        // Define unmapped addresses
        let unmapped = [0xfea0..=0xfeff, 0xff03..=0xff03, 0xff27..=0xff2f];

        // Test unmapped addresses
        for gap in unmapped {
            for addr in gap {
                // Write to every unmapped address
                emu.bus.borrow_mut().write(addr, 0xaa);
                // Check the write didn't work
                assert_eq!(emu.bus.borrow().read(addr), 0xff, "{addr:#06x}");
            }
        }
    }
}
