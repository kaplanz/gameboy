//! Audio processing unit.

use std::cell::RefCell;
use std::rc::Rc;

use remus::bus::Bus;
use remus::mem::Ram;
use remus::reg::Register;
use remus::{Block, Machine, SharedDevice};

use crate::dmg::Board;

pub type Wave = Ram<0x0010>;

/// APU model.
#[derive(Debug, Default)]
pub struct Apu {
    /// State
    /// Connections
    /// Control
    // ┌────────┬──────────┬─────┬───────┐
    // │  Size  │   Name   │ Dev │ Alias │
    // ├────────┼──────────┼─────┼───────┤
    // │   23 B │ Control  │ Reg │       │
    // └────────┴──────────┴─────┴───────┘
    ctl: Control,
    /// Devices
    // ┌────────┬──────────┬─────┬───────┐
    // │  Size  │   Name   │ Dev │ Alias │
    // ├────────┼──────────┼─────┼───────┤
    // │   16 B │ Waveform │ RAM │ WAVE  │
    // └────────┴──────────┴─────┴───────┘
    wave: Rc<RefCell<Wave>>,
}

impl Apu {
    /// Gets a shared reference to the APU's waveform RAM.
    #[must_use]
    pub fn wave(&self) -> SharedDevice {
        self.wave.clone()
    }
}

impl Block for Apu {
    fn reset(&mut self) {
        // Reset control
        self.ctl.reset();

        // Reset memory
        self.wave.borrow_mut().reset();
    }
}

impl Board for Apu {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Connect boards
        self.ctl.connect(bus);

        // Extract memory
        let wave = self.wave();

        // Map devices on bus  // ┌──────┬────────┬──────────┬─────┐
                               // │ Addr │  Size  │   Name   │ Dev │
                               // ├──────┼────────┼──────────┼─────┤
        bus.map(0xff30, wave); // │ ff30 │   16 B │ Waveform │ RAM │
                               // └──────┴────────┴──────────┴─────┘
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

/// Control registers.
#[rustfmt::skip]
#[derive(Debug, Default)]
struct Control {
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
    nr52: Rc<RefCell<Register<u8>>>,
    nr51: Rc<RefCell<Register<u8>>>,
    nr50: Rc<RefCell<Register<u8>>>,
    // Sound Channel 1 — Pulse with wavelength sweep
    nr10: Rc<RefCell<Register<u8>>>,
    nr11: Rc<RefCell<Register<u8>>>,
    nr12: Rc<RefCell<Register<u8>>>,
    nr13: Rc<RefCell<Register<u8>>>,
    nr14: Rc<RefCell<Register<u8>>>,
    // Sound Channel 2 — Pulse
    nr21: Rc<RefCell<Register<u8>>>,
    nr22: Rc<RefCell<Register<u8>>>,
    nr23: Rc<RefCell<Register<u8>>>,
    nr24: Rc<RefCell<Register<u8>>>,
    // Sound Channel 3 — Wave output
    nr30: Rc<RefCell<Register<u8>>>,
    nr31: Rc<RefCell<Register<u8>>>,
    nr32: Rc<RefCell<Register<u8>>>,
    nr33: Rc<RefCell<Register<u8>>>,
    nr34: Rc<RefCell<Register<u8>>>,
    // Sound Channel 4 — Noise
    nr41: Rc<RefCell<Register<u8>>>,
    nr42: Rc<RefCell<Register<u8>>>,
    nr43: Rc<RefCell<Register<u8>>>,
    nr44: Rc<RefCell<Register<u8>>>,
}

impl Block for Control {
    fn reset(&mut self) {}
}

impl Board for Control {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let nr52 = self.nr52.clone();
        let nr51 = self.nr51.clone();
        let nr50 = self.nr50.clone();
        let nr10 = self.nr10.clone();
        let nr11 = self.nr11.clone();
        let nr12 = self.nr12.clone();
        let nr13 = self.nr13.clone();
        let nr14 = self.nr14.clone();
        let nr21 = self.nr21.clone();
        let nr22 = self.nr22.clone();
        let nr23 = self.nr23.clone();
        let nr24 = self.nr24.clone();
        let nr30 = self.nr30.clone();
        let nr31 = self.nr31.clone();
        let nr32 = self.nr32.clone();
        let nr33 = self.nr33.clone();
        let nr34 = self.nr34.clone();
        let nr41 = self.nr41.clone();
        let nr42 = self.nr42.clone();
        let nr43 = self.nr43.clone();
        let nr44 = self.nr44.clone();

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