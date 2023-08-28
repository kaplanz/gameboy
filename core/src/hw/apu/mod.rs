//! Audio processing unit.

use remus::bus::Bus;
use remus::mem::Ram;
use remus::reg::Register;
use remus::{Block, Board, Machine, Shared};

pub type Wave = Ram<0x0010>;

/// APU model.
#[derive(Debug, Default)]
pub struct Apu {
    /// State
    /// Connections
    /// Control
    // ┌──────┬──────────┬─────┐
    // │ Size │   Name   │ Dev │
    // ├──────┼──────────┼─────┤
    // │ 23 B │ Control  │ Reg │
    // └──────┴──────────┴─────┘
    file: File,
    /// Devices
    // ┌──────┬──────────┬─────┬───────┐
    // │ Size │   Name   │ Dev │ Alias │
    // ├──────┼──────────┼─────┼───────┤
    // │ 16 B │ Waveform │ RAM │ WAVE  │
    // └──────┴──────────┴─────┴───────┘
    wave: Shared<Wave>,
}

impl Apu {
    /// Gets a shared reference to the APU's waveform RAM.
    #[must_use]
    pub fn wave(&self) -> Shared<Wave> {
        self.wave.clone()
    }
}

impl Block for Apu {
    fn reset(&mut self) {
        // Reset control
        self.file.reset();

        // Reset memory
        self.wave.reset();
    }
}

impl Board for Apu {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Connect boards
        self.file.connect(bus);

        // Extract memory
        let wave = self.wave().to_dynamic();

        // Map devices on bus  // ┌──────┬──────┬──────────┬─────┐
                               // │ Addr │ Size │   Name   │ Dev │
                               // ├──────┼──────┼──────────┼─────┤
        bus.map(0xff30, wave); // │ ff30 │ 16 B │ Waveform │ RAM │
                               // └──────┴──────┴──────────┴─────┘
    }
}

impl Machine for Apu {
    fn enabled(&self) -> bool {
        todo!()
    }

    fn cycle(&mut self) {
        todo!()
    }
}

/// APU control register file.
#[rustfmt::skip]
#[derive(Debug, Default)]
struct File {
    // ┌──────┬────────────────────┬─────┬───────┐
    // │ Size │        Name        │ Dev │ Alias │
    // ├──────┼────────────────────┼─────┼───────┤
    // │  1 B │ Audio Enable       │ Reg │ AUDEN │
    // │  1 B │ Sound Panning      │ Reg │       │
    // │  1 B │ Master Volume      │ Reg │       │
    // │  1 B │ CH1 Sweep          │ Reg │       │
    // │  1 B │ CH1 Length + Duty  │ Reg │       │
    // │  1 B │ CH1 Volume + Env.  │ Reg │       │
    // │  1 B │ CH1 Wavelength Low │ Reg │       │
    // │  1 B │ CH1 Wave Hi + Ctl. │ Reg │       │
    // │  1 B │ CH2 Length + Duty  │ Reg │       │
    // │  1 B │ CH2 Volume + Env.  │ Reg │       │
    // │  1 B │ CH2 Wavelength Low │ Reg │       │
    // │  1 B │ CH2 Wave Hi + Ctl. │ Reg │       │
    // │  1 B │ CH3 DAC Enable     │ Reg │       │
    // │  1 B │ CH3 Length Timer   │ Reg │       │
    // │  1 B │ CH3 Output Level   │ Reg │       │
    // │  1 B │ CH3 Waveform Low   │ Reg │       │
    // │  1 B │ CH3 Wave Hi + Ctl. │ Reg │       │
    // │  1 B │ CH4 Length Timer   │ Reg │       │
    // │  1 B │ CH4 Volume + Env.  │ Reg │       │
    // │  1 B │ CH4 Freq. + Rand.  │ Reg │       │
    // │  1 B │ CH4 Control        │ Reg │       │
    // └──────┴────────────────────┴─────┴───────┘
    // Global Control Registers
    nr52: Shared<Register<u8>>,
    nr51: Shared<Register<u8>>,
    nr50: Shared<Register<u8>>,
    // Sound Channel 1 — Pulse with wavelength sweep
    nr10: Shared<Register<u8>>,
    nr11: Shared<Register<u8>>,
    nr12: Shared<Register<u8>>,
    nr13: Shared<Register<u8>>,
    nr14: Shared<Register<u8>>,
    // Sound Channel 2 — Pulse
    nr21: Shared<Register<u8>>,
    nr22: Shared<Register<u8>>,
    nr23: Shared<Register<u8>>,
    nr24: Shared<Register<u8>>,
    // Sound Channel 3 — Wave output
    nr30: Shared<Register<u8>>,
    nr31: Shared<Register<u8>>,
    nr32: Shared<Register<u8>>,
    nr33: Shared<Register<u8>>,
    nr34: Shared<Register<u8>>,
    // Sound Channel 4 — Noise
    nr41: Shared<Register<u8>>,
    nr42: Shared<Register<u8>>,
    nr43: Shared<Register<u8>>,
    nr44: Shared<Register<u8>>,
}

impl Block for File {
    fn reset(&mut self) {}
}

impl Board for File {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let nr52 = self.nr52.clone().to_dynamic();
        let nr51 = self.nr51.clone().to_dynamic();
        let nr50 = self.nr50.clone().to_dynamic();
        let nr10 = self.nr10.clone().to_dynamic();
        let nr11 = self.nr11.clone().to_dynamic();
        let nr12 = self.nr12.clone().to_dynamic();
        let nr13 = self.nr13.clone().to_dynamic();
        let nr14 = self.nr14.clone().to_dynamic();
        let nr21 = self.nr21.clone().to_dynamic();
        let nr22 = self.nr22.clone().to_dynamic();
        let nr23 = self.nr23.clone().to_dynamic();
        let nr24 = self.nr24.clone().to_dynamic();
        let nr30 = self.nr30.clone().to_dynamic();
        let nr31 = self.nr31.clone().to_dynamic();
        let nr32 = self.nr32.clone().to_dynamic();
        let nr33 = self.nr33.clone().to_dynamic();
        let nr34 = self.nr34.clone().to_dynamic();
        let nr41 = self.nr41.clone().to_dynamic();
        let nr42 = self.nr42.clone().to_dynamic();
        let nr43 = self.nr43.clone().to_dynamic();
        let nr44 = self.nr44.clone().to_dynamic();

        // Map devices on bus   // ┌──────┬──────┬────────────────────┬─────┐
                                // │ Addr │ Size │        Name        │ Dev │
                                // ├──────┼──────┼────────────────────┼─────┤
        bus.map(0xff10, nr10);  // │ ff10 │  1 B │ CH1 Sweep          │ Reg │
        bus.map(0xff11, nr11);  // │ ff11 │  1 B │ CH1 Length + Duty  │ Reg │
        bus.map(0xff12, nr12);  // │ ff12 │  1 B │ CH1 Volume + Env.  │ Reg │
        bus.map(0xff13, nr13);  // │ ff13 │  1 B │ CH1 Wavelength Low │ Reg │
        bus.map(0xff14, nr14);  // │ ff14 │  1 B │ CH1 Wave Hi + Ctl. │ Reg │
                                // │ ff15 │  1 B │ Unmapped           │ --- │
        bus.map(0xff16, nr21);  // │ ff16 │  1 B │ CH2 Length + Duty  │ Reg │
        bus.map(0xff17, nr22);  // │ ff17 │  1 B │ CH2 Volume + Env.  │ Reg │
        bus.map(0xff18, nr23);  // │ ff18 │  1 B │ CH2 Wavelength Low │ Reg │
        bus.map(0xff19, nr24);  // │ ff19 │  1 B │ CH2 Wave Hi + Ctl. │ Reg │
        bus.map(0xff1a, nr30);  // │ ff1a │  1 B │ CH3 DAC Enable     │ Reg │
        bus.map(0xff1b, nr31);  // │ ff1b │  1 B │ CH3 Length Timer   │ Reg │
        bus.map(0xff1c, nr32);  // │ ff1c │  1 B │ CH3 Output Level   │ Reg │
        bus.map(0xff1d, nr33);  // │ ff1d │  1 B │ CH3 Waveform Low   │ Reg │
        bus.map(0xff1e, nr34);  // │ ff1e │  1 B │ CH3 Wave Hi + Ctl. │ Reg │
                                // │ ff1f │  1 B │ Unmapped           │ --- │
        bus.map(0xff20, nr41);  // │ ff20 │  1 B │ CH4 Length Timer   │ Reg │
        bus.map(0xff21, nr42);  // │ ff21 │  1 B │ CH4 Volume + Env.  │ Reg │
        bus.map(0xff22, nr43);  // │ ff22 │  1 B │ CH4 Freq. + Rand.  │ Reg │
        bus.map(0xff23, nr44);  // │ ff23 │  1 B │ CH4 Control        │ Reg │
        bus.map(0xff24, nr50);  // │ ff24 │  1 B │ Master Volume      │ Reg │
        bus.map(0xff25, nr51);  // │ ff25 │  1 B │ Sound Panning      │ Reg │
        bus.map(0xff26, nr52);  // │ ff26 │  1 B │ Audio Enable       │ Reg │
                                // └──────┴──────┴────────────────────┴─────┘
    }
}
