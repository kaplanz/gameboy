use rugby_arch::mem::Memory;
use rugby_arch::Byte;

use self::api::cart::Support as _;
use self::api::proc::Support as _;
use self::cart::mbc::Mbc;
use self::pic::Pic;
use self::timer::Timer;
use super::*;

/// Sample boot ROM.
const BOOT: &[Byte; 0x0100] = include_bytes!("../../../../roms/boot/sameboy/dmg_boot.bin");

/// Sample ROM header.
const GAME: &[Byte; 0x8000] = include_bytes!("../../../../roms/games/2048/2048.gb");

fn setup() -> GameBoy {
    // Instantiate a `Boot`
    let boot = Boot::from(*BOOT);
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
    let bus = emu.cpu_mut();

    // Ensure boot ROM starts enabled (compare against boot ROM).
    (0x0000..=0x00ff)
        .map(|addr| bus.read(addr))
        .zip(BOOT)
        .for_each(|(byte, &boot)| assert_eq!(byte, boot));

    // Disable boot ROM
    bus.write(0xff50, 0x01);

    // Check if disable was successful (compare against cartridge ROM).
    (0x0000..=0x00ff)
        .map(|addr| bus.read(addr))
        .zip(GAME)
        .for_each(|(byte, &game)| assert_eq!(byte, game));
}

#[test]
fn bus_all_works() {
    let mut emu = setup();
    let bus = &mut emu.main.soc.cpu;

    // Boot ROM
    if let Some(boot) = &emu.boot {
        (0x0000..=0x00ff)
            .map(|addr| boot.mem.borrow().boot.read(addr).unwrap())
            .zip(BOOT)
            .for_each(|(byte, &game)| assert_eq!(byte, game));
    }
    // Cartridge ROM
    if let Some(cart) = &emu.cart {
        (0x0100..=0x7fff)
            .map(|addr| cart.body().rom().read(addr).unwrap())
            .zip(&GAME[0x0100..=0x7fff])
            .for_each(|(byte, &boot)| assert_eq!(byte, boot));
    }
    // Video RAM
    (0x8000..=0x9fff).for_each(|addr| bus.write(addr, 0x03));
    (0x0000..=0x1fff)
        .map(|addr: Word| emu.main.vram.read(addr).unwrap())
        .for_each(|byte| assert_eq!(byte, 0x03));
    // External RAM
    if let Some(cart) = &emu.cart {
        (0xa000..=0xa3ff).for_each(|addr| bus.write(addr, 0x04));
        bus.write(0x0100, 0x0a); // enable RAM
        (0xa400..=0xa7ff).for_each(|addr| bus.write(addr, 0x40));
        (0x0000..=0x03ff)
            .map(|addr| cart.body().ram().read(addr).unwrap())
            .for_each(|byte| assert_eq!(byte, 0x00));
        (0x0400..=0x07ff)
            .map(|addr| cart.body().ram().read(addr).unwrap())
            .for_each(|byte| assert_eq!(byte, 0x40));
        (0x0800..=0x1fff)
            .map(|addr| cart.body().ram().read(addr).ok())
            .for_each(|byte| assert_eq!(byte, None));
    }
    // Object memory
    (0xfe00..=0xfe9f).for_each(|addr| bus.write(addr, 0x05));
    (0x0000..=0x009f)
        .map(|addr: Word| emu.main.soc.mem.oam.read(addr).unwrap())
        .for_each(|byte| assert_eq!(byte, 0x05));
    // Controller
    (0xff00..=0xff00).for_each(|addr| bus.write(addr, 0x60));
    (0x0000..=0x0000) // NOTE: Only bits 0x30 are writable
        .map(|addr| emu.main.soc.joy.con.read(addr).unwrap())
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
            <Timer as Port<Byte>>::Select::Div,
            <Timer as Port<Byte>>::Select::Tima,
            <Timer as Port<Byte>>::Select::Tma,
            <Timer as Port<Byte>>::Select::Tac,
        ])
        .map(|(_, reg)| emu.main.soc.tma.load(reg))
        .zip([0x00, 0x08, 0x08, 0xf8])
        .for_each(|(found, expected)| assert_eq!(found, expected));
    // Interrupt flag
    (0xff0f..=0xff0f).for_each(|addr| bus.write(addr, 0x09));
    (0x0000..=0x0000)
        .map(|_| emu.main.soc.pic.load(<Pic as Port<Byte>>::Select::If))
        .for_each(|byte| assert_eq!(byte, 0xe9));
    // Audio
    (0xff10..=0xff27).for_each(|addr| bus.write(addr, 0x0a));
    (0x0000..=0x0017)
        .map(|_| 0x0a) // FIXME
        .for_each(|byte| assert_eq!(byte, 0x0a));
    // Wave RAM
    (0xff30..=0xff3f).for_each(|addr| bus.write(addr, 0x0b));
    (0x0000..=0x000f)
        .map(|_| 0x0b) // FIXME
        .for_each(|byte| assert_eq!(byte, 0x0b));
    // LCD
    (0xff40..=0xff4b).for_each(|addr| bus.write(addr, 0x0c));
    (0x0000..=0x000b)
        .zip([
            <Ppu as Port<Byte>>::Select::Lcdc,
            <Ppu as Port<Byte>>::Select::Stat,
            <Ppu as Port<Byte>>::Select::Scy,
            <Ppu as Port<Byte>>::Select::Scx,
            <Ppu as Port<Byte>>::Select::Ly,
            <Ppu as Port<Byte>>::Select::Lyc,
            <Ppu as Port<Byte>>::Select::Dma,
            <Ppu as Port<Byte>>::Select::Bgp,
            <Ppu as Port<Byte>>::Select::Obp0,
            <Ppu as Port<Byte>>::Select::Obp1,
            <Ppu as Port<Byte>>::Select::Wy,
            <Ppu as Port<Byte>>::Select::Wx,
        ])
        .map(|(_, reg)| emu.main.soc.ppu.load(reg))
        .for_each(|byte| assert_eq!(byte, 0x0c));
    // Boot ROM disable
    (0xff50..=0xff50).for_each(|addr| bus.write(addr, 0x0d));
    if let Some(boot) = &emu.boot {
        (0x0000..=0x0000)
            .map(|addr| boot.reg.read(addr).unwrap())
            .for_each(|byte| assert_eq!(byte, 0xff));
    }
    // High RAM
    (0xff80..=0xfffe).for_each(|addr| bus.write(addr, 0x0e));
    (0x0000..=0x007e)
        .map(|addr: Word| emu.main.soc.mem.hram.read(addr).unwrap())
        .for_each(|byte| assert_eq!(byte, 0x0e));
    // Interrupt enable
    (0xffff..=0xffff).for_each(|addr| bus.write(addr, 0x0f));
    (0x0000..=0x0000)
        .map(|_| emu.main.soc.pic.load(<Pic as Port<Byte>>::Select::Ie))
        .for_each(|byte| assert_eq!(byte, 0xef));
}

#[test]
fn bus_unmapped_works() {
    let bus = &mut setup().main.noc.ibus;

    // Test unmapped addresses
    for range in [0xfea0..=0xfeff, 0xff03..=0xff03, 0xff27..=0xff2f] {
        for addr in range {
            // Write to every unmapped address
            assert!(bus.write(addr, 0xaa).is_err(), "{addr:#06x}");
            // Check the write didn't work
            assert!(bus.read(addr).is_err(), "{addr:#06x}");
        }
    }
}
