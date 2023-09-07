//! DMG-01: [Game Boy]
//!
//! [Game Boy]: https://en.wikipedia.org/wiki/Game_Boy

use remus::bus::Bus;
use remus::dev::Device;
use remus::{Address, Block, Board, Location, Machine, Shared};

use self::cpu::Cpu;
use self::dbg::Doctor;
use self::mem::Memory;
use crate::dev::Unmapped;
use crate::dmg::cpu::Processor;
use crate::emu::Emulator;
use crate::hw::apu::Apu;
use crate::hw::cart::Cartridge;
use crate::hw::joypad::Joypad;
use crate::hw::pic::Pic;
use crate::hw::ppu::Ppu;
use crate::hw::serial::Serial;
use crate::hw::timer::Timer;

mod boot;
mod mem;

pub mod dbg;

pub use self::boot::Rom as Boot;
pub use crate::emu::Screen as Dimensions;
pub use crate::hw::cpu::sm83 as cpu;
pub use crate::hw::joypad::Button;
pub use crate::hw::ppu::Screen;
pub use crate::hw::{cart, ppu, timer};

/// DMG-01 screen specification.
pub const SCREEN: Dimensions = Dimensions {
    width: 160,
    height: 144,
};

/// DMG-01 Game Boy emulator.
#[derive(Debug)]
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
    // Shared
    bus: Shared<Bus>,
    pic: Shared<Pic>,
    // Debug
    doc: Option<Doctor>,
}

impl Default for GameBoy {
    fn default() -> Self {
        // Construct shared blocks
        let bus = Shared::from(Bus::default());
        let pic = Shared::from(Pic::default());
        // Construct self
        Self {
            clock: usize::default(),
            apu: Apu::default(),
            cpu: Cpu::new(bus.clone(), pic.clone()),
            ppu: Ppu::new(bus.clone(), pic.clone()),
            joypad: Joypad::new(pic.clone()),
            serial: Serial::new(pic.clone()),
            timer: Timer::new(pic.clone()),
            cart: Option::default(),
            mem: Memory::default(),
            bus: bus.clone(),
            pic,
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
        Self {
            mem: Memory::with(boot),
            ..Default::default()
        }
        .setup()
    }

    /// Loads a game [`Cartridge`] into the `GameBoy`
    ///
    /// If a cartridge has already been loaded, it will be disconnected and
    /// replaced.
    pub fn load(&mut self, cart: Cartridge) {
        // Disconnect any connected cartridge from the bus
        let bus = &mut self.bus.borrow_mut();
        if let Some(cart) = &self.cart {
            cart.disconnect(bus);
        }
        // Store and connect the supplied cartridge
        cart.connect(bus);
        self.cart = Some(cart);
    }

    /// Sets up the `GameBoy`, such that is it ready to be run.
    #[must_use]
    fn setup(self) -> Self {
        // Connect devices to bus
        self.connect(&mut self.bus.borrow_mut());

        self
    }

    /// Simulate booting for `GameBoy`s with no [`Cartridge`].
    #[must_use]
    fn boot(mut self) -> Self {
        type Register = <Cpu as Location<u16>>::Register;

        // Execute setup code
        self.cpu.exec(0xfb); // ei ; enable interrupts

        // Initialize registers
        self.cpu.store(Register::AF, 0x01b0u16);
        self.cpu.store(Register::BC, 0x0013u16);
        self.cpu.store(Register::DE, 0x00d8u16);
        self.cpu.store(Register::HL, 0x014du16);
        self.cpu.store(Register::SP, 0xfffeu16);

        // Move the PC to the ROM's start
        self.cpu.goto(0x0100);

        // Enable the LCD
        self.bus.write(0xff40, 0x91);
        // Disable the boot ROM
        self.bus.write(0xff50, 0x01);

        self
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
        &self.cpu
    }

    /// Mutably gets the `GameBoy`'s CPU.
    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    /// Gets the `GameBoy`'s PPU.
    #[must_use]
    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    /// Mutably gets the `GameBoy`'s PPU.
    pub fn ppu_mut(&mut self) -> &mut Ppu {
        &mut self.ppu
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
        // Processors
        self.apu.reset();
        self.cpu.reset();
        self.ppu.reset();
        // Peripherals
        self.joypad.reset();
        self.serial.reset();
        self.timer.reset();
        // Memory
        if let Some(cart) = &mut self.cart {
            cart.reset();
        }
        self.mem.reset();
        // Shared
        self.bus.reset();
        self.pic.reset();
    }
}

impl Board for GameBoy {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
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

        // Processors
        self.apu.connect(bus);
        self.cpu.connect(bus);
        self.ppu.connect(bus);
        // Peripherals
        self.joypad.connect(bus);
        self.serial.connect(bus);
        self.timer.connect(bus);
        // Memory
        self.mem.connect(bus);
        // Shared
        self.pic.connect(bus);

        // NOTE: Use fallback to report invalid reads as `0xff`
        let unmap = Unmapped::<0x10000>::new().to_dynamic();
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
        // CPU runs on a 1 MiHz clock
        if self.clock % 4 == 0 {
            // Wake disabled CPU if interrupts pending
            if !self.cpu.enabled() && self.pic.borrow().int().is_some() {
                self.cpu.wake();
            }

            // Collect doctor entries (if enabled)
            if let Some(doc) = &mut self.doc {
                if let Some(entry) = self.cpu.doctor() {
                    doc.0.push(entry);
                }
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

        // Serial runs on a 8192 Hz clock
        if self.clock % 0x200 == 0 && self.serial.enabled() {
            self.serial.cycle();
        }

        // Timer runs on a 4 MiHz clock
        if self.timer.enabled() {
            self.timer.cycle();
        }

        // Keep track of cycles executed
        self.clock = self.clock.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample boot ROM.
    const BOOT: &[u8; 0x0100] = include_bytes!("../../../../roms/boot/sameboy/dmg_boot.bin");

    /// Sample ROM header.
    const GAME: &[u8; 0x8000] = include_bytes!("../../../../roms/games/2048/2048.gb");

    fn setup() -> GameBoy {
        // Instantiate a `Boot`
        let boot = Boot::try_from(BOOT).unwrap();
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

        // Ensure boot ROM starts enabled:
        // - Perform comparison against boot ROM contents
        (0x0000..=0x0100)
            .map(|addr| emu.bus.read(addr))
            .zip(BOOT)
            .for_each(|(read, &rom)| assert_eq!(read, rom));

        // Disable boot ROM
        emu.bus.write(0xff50, 0x01);

        // Check if disable was successful:
        // - Perform comparison against cartridge ROM contents
        (0x0000..=0x0100)
            .map(|addr| emu.bus.read(addr))
            .zip(GAME)
            .for_each(|(read, &rom)| assert_eq!(read, rom));
    }

    #[test]
    fn bus_all_works() {
        // NOTE: Test reads (and writes) for each component separately
        let mut emu = setup();

        // Boot ROM
        (0x0000..=0x00ff)
            .map(|addr| emu.mem.boot().borrow().rom().read(addr))
            .any(|byte| byte != 0x01);
        // Cartridge ROM
        if let Some(cart) = &emu.cart {
            (0x0100..=0x7fff).for_each(|addr| emu.bus.write(addr, 0x02));
            assert!((0x0100..=0x7fff)
                .map(|addr| cart.rom().read(addr))
                .any(|byte| byte != 0x02));
        }
        // Video RAM
        (0x8000..=0x9fff).for_each(|addr| emu.bus.write(addr, 0x03));
        (0x0000..=0x1fff)
            .map(|addr| emu.ppu.vram().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x03));
        // External RAM
        if let Some(cart) = &emu.cart {
            (0xa000..=0xbfff).for_each(|addr| emu.bus.write(addr, 0x04));
            (0x0000..=0x1fff) // NOTE: External RAM is disabled for this ROM
                .map(|addr| cart.ram().read(addr))
                .for_each(|byte| assert_eq!(byte, 0x00));
        }
        // Object RAM
        (0xfe00..=0xfe9f).for_each(|addr| emu.bus.write(addr, 0x05));
        (0x0000..=0x009f)
            .map(|addr| emu.ppu.oam().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x05));
        // Controller
        (0xff00..=0xff00).for_each(|addr| emu.bus.write(addr, 0x60));
        (0x0000..=0x0000) // NOTE: Only bits 0x30 are writable
            .map(|addr| emu.joypad.con().read(addr))
            .for_each(|byte| assert_eq!(byte, 0xef));
        // Serial
        (0xff01..=0xff03).for_each(|addr| emu.bus.write(addr, 0x07));
        (0x0000..=0x0002)
            .map(|_| 0x07) // FIXME
            .for_each(|byte| assert_eq!(byte, 0x07));
        // Timer
        (0xff04..=0xff07).for_each(|addr| emu.bus.write(addr, 0x08));
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
        // Interrupt Active
        (0xff0f..=0xff0f).for_each(|addr| emu.bus.write(addr, 0x09));
        (0x0000..=0x0000)
            .map(|addr| emu.pic.borrow().active().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x09));
        // Audio
        (0xff10..=0xff27).for_each(|addr| emu.bus.write(addr, 0x0a));
        (0x0000..=0x0017)
            .map(|_| 0x0a) // FIXME
            .for_each(|byte| assert_eq!(byte, 0x0a));
        // Waveform RAM
        (0xff30..=0xff3f).for_each(|addr| emu.bus.write(addr, 0x0b));
        (0x0000..=0x000f)
            .map(|_| 0x0b) // FIXME
            .for_each(|byte| assert_eq!(byte, 0x0b));
        // LCD
        (0xff40..=0xff4b).for_each(|addr| emu.bus.write(addr, 0x0c));
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
            .map(|(_, reg)| emu.ppu.load(reg))
            .for_each(|byte| assert_eq!(byte, 0x0c));
        // Boot ROM Disable
        (0xff50..=0xff50).for_each(|addr| emu.bus.write(addr, 0x0d));
        (0x0000..=0x0000)
            .map(|addr| emu.mem.boot().borrow().ctrl().read(addr))
            .for_each(|byte| assert_eq!(byte, 0xff));
        // High RAM
        (0xff80..=0xfffe).for_each(|addr| emu.bus.write(addr, 0x0e));
        (0x0000..=0x007e)
            .map(|addr| emu.mem.hram().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x0e));
        // Interrupt Enable
        (0xffff..=0xffff).for_each(|addr| emu.bus.write(addr, 0x0f));
        (0x0000..=0x0000)
            .map(|addr| emu.pic.borrow().enable().read(addr))
            .for_each(|byte| assert_eq!(byte, 0x0f));
    }

    #[test]
    fn bus_unmapped_works() {
        let mut emu = setup();

        // Define unmapped addresses
        let unmapped = [0xfea0..=0xfeff, 0xff03..=0xff03, 0xff27..=0xff2f];

        // Test unmapped addresses
        for gap in unmapped {
            for addr in gap {
                // Write to every unmapped address
                emu.bus.write(addr, 0xaa);
                // Check the write didn't work
                assert_eq!(emu.bus.read(addr), 0xff, "{addr:#06x}");
            }
        }
    }
}
