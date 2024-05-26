#![allow(non_snake_case)]

use std::fmt::{Debug, Display};

use rugby::arch::reg::Port;
use rugby::arch::{Block, Word};
use rugby::core::dmg::cpu::Cpu;
use rugby::core::dmg::{Cartridge, GameBoy};
use rugby::prelude::*;
use thiserror::Error;

/// Number of cycles after which the test is considered to have failed due to a
/// timeout error.
const TIMEOUT: usize = 10_000_000;

/// Perform integration test emulation.
fn emulate(rom: &[u8]) -> Result<()> {
    // Instantiate a cartridge
    let cart = Cartridge::new(rom).unwrap();
    // Create an emulator instance
    let mut emu = GameBoy::new();
    // Load the cartridge
    emu.load(cart);

    // Loop until completion or timeout
    for _ in 0..TIMEOUT {
        emu.cycle();

        // Check for success or failure
        match check(&emu) {
            Err(Error::Running) => continue,
            res => return res,
        }
    }

    // Fail with timeout if reached
    Err(Error::Timeout)
}

/// Check for test results.
fn check(emu: &GameBoy) -> Result<()> {
    type Select = <Cpu as Port<Word>>::Select;
    // Extract register values
    let cpu = emu.cpu();
    let bc: u16 = cpu.load(Select::BC);
    let de: u16 = cpu.load(Select::DE);
    let hl: u16 = cpu.load(Select::HL);
    // Calculate pass/fail conditions
    let pass = (bc == 0x0305) && (de == 0x080d) && (hl == 0x1522);
    let fail = (bc == 0x4242) && (de == 0x4242) && (hl == 0x4242);
    // Report results
    if fail {
        Err(Error::Failed)
    } else if pass {
        Ok(())
    } else {
        Err(Error::Running)
    }
}

/// A convenient type alias for [`Result`](std::result::Result).
type Result<T, E = Error> = std::result::Result<T, E>;

/// Failure conditions caused by a test.
#[derive(Error)]
enum Error {
    /// Test has explicitly failed.
    #[error("test failed")]
    Failed,
    /// Test is still in progress.
    #[error("test in progress")]
    Running,
    /// Test has reached timeout.
    #[error("timeout reached")]
    Timeout,
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

macro_rules! test {
    ($($test:ident = $path:tt;)*) => {
        $(
            #[test]
            fn $test() -> Result<()> {
                emulate(include_bytes!($path))
            }
        )*
    };
}

test! {
    acceptance_add_sp_e_timing                  = "../roms/test/mooneye/acceptance/add_sp_e_timing.gb";
    acceptance_bits_mem_oam                     = "../roms/test/mooneye/acceptance/bits/mem_oam.gb";
    acceptance_bits_reg_f                       = "../roms/test/mooneye/acceptance/bits/reg_f.gb";
    acceptance_bits_unused_hwio_GS              = "../roms/test/mooneye/acceptance/bits/unused_hwio-GS.gb";
    acceptance_boot_div_dmg0                    = "../roms/test/mooneye/acceptance/boot_div-dmg0.gb";
    acceptance_boot_div_dmgABCmgb               = "../roms/test/mooneye/acceptance/boot_div-dmgABCmgb.gb";
    acceptance_boot_hwio_dmg0                   = "../roms/test/mooneye/acceptance/boot_hwio-dmg0.gb";
    acceptance_boot_hwio_dmgABCmgb              = "../roms/test/mooneye/acceptance/boot_hwio-dmgABCmgb.gb";
    acceptance_boot_regs_dmg0                   = "../roms/test/mooneye/acceptance/boot_regs-dmg0.gb";
    acceptance_boot_regs_dmgABC                 = "../roms/test/mooneye/acceptance/boot_regs-dmgABC.gb";
    acceptance_call_cc_timing                   = "../roms/test/mooneye/acceptance/call_cc_timing.gb";
    acceptance_call_cc_timing2                  = "../roms/test/mooneye/acceptance/call_cc_timing2.gb";
    acceptance_call_timing                      = "../roms/test/mooneye/acceptance/call_timing.gb";
    acceptance_call_timing2                     = "../roms/test/mooneye/acceptance/call_timing2.gb";
    acceptance_di_timing_GS                     = "../roms/test/mooneye/acceptance/di_timing-GS.gb";
    acceptance_div_timing                       = "../roms/test/mooneye/acceptance/div_timing.gb";
    acceptance_ei_sequence                      = "../roms/test/mooneye/acceptance/ei_sequence.gb";
    acceptance_ei_timing                        = "../roms/test/mooneye/acceptance/ei_timing.gb";
    acceptance_halt_ime0_ei                     = "../roms/test/mooneye/acceptance/halt_ime0_ei.gb";
    acceptance_halt_ime0_nointr_timing          = "../roms/test/mooneye/acceptance/halt_ime0_nointr_timing.gb";
    acceptance_halt_ime1_timing                 = "../roms/test/mooneye/acceptance/halt_ime1_timing.gb";
    acceptance_halt_ime1_timing2_GS             = "../roms/test/mooneye/acceptance/halt_ime1_timing2-GS.gb";
    acceptance_if_ie_registers                  = "../roms/test/mooneye/acceptance/if_ie_registers.gb";
    acceptance_instr_daa                        = "../roms/test/mooneye/acceptance/instr/daa.gb";
    acceptance_interrupts_ie_push               = "../roms/test/mooneye/acceptance/interrupts/ie_push.gb";
    acceptance_intr_timing                      = "../roms/test/mooneye/acceptance/intr_timing.gb";
    acceptance_jp_cc_timing                     = "../roms/test/mooneye/acceptance/jp_cc_timing.gb";
    acceptance_jp_timing                        = "../roms/test/mooneye/acceptance/jp_timing.gb";
    acceptance_ld_hl_sp_e_timing                = "../roms/test/mooneye/acceptance/ld_hl_sp_e_timing.gb";
    acceptance_oam_dma_basic                    = "../roms/test/mooneye/acceptance/oam_dma/basic.gb";
    acceptance_oam_dma_reg_read                 = "../roms/test/mooneye/acceptance/oam_dma/reg_read.gb";
    acceptance_oam_dma_restart                  = "../roms/test/mooneye/acceptance/oam_dma_restart.gb";
    acceptance_oam_dma_sources_GS               = "../roms/test/mooneye/acceptance/oam_dma/sources-GS.gb";
    acceptance_oam_dma_start                    = "../roms/test/mooneye/acceptance/oam_dma_start.gb";
    acceptance_oam_dma_timing                   = "../roms/test/mooneye/acceptance/oam_dma_timing.gb";
    acceptance_pop_timing                       = "../roms/test/mooneye/acceptance/pop_timing.gb";
    acceptance_ppu_hblank_ly_scx_timing_GS      = "../roms/test/mooneye/acceptance/ppu/hblank_ly_scx_timing-GS.gb";
    acceptance_ppu_intr_1_2_timing_GS           = "../roms/test/mooneye/acceptance/ppu/intr_1_2_timing-GS.gb";
    acceptance_ppu_intr_2_0_timing              = "../roms/test/mooneye/acceptance/ppu/intr_2_0_timing.gb";
    acceptance_ppu_intr_2_mode0_timing          = "../roms/test/mooneye/acceptance/ppu/intr_2_mode0_timing.gb";
    acceptance_ppu_intr_2_mode0_timing_sprites  = "../roms/test/mooneye/acceptance/ppu/intr_2_mode0_timing_sprites.gb";
    acceptance_ppu_intr_2_mode3_timing          = "../roms/test/mooneye/acceptance/ppu/intr_2_mode3_timing.gb";
    acceptance_ppu_intr_2_oam_ok_timing         = "../roms/test/mooneye/acceptance/ppu/intr_2_oam_ok_timing.gb";
    acceptance_ppu_lcdon_timing_GS              = "../roms/test/mooneye/acceptance/ppu/lcdon_timing-GS.gb";
    acceptance_ppu_lcdon_write_timing_GS        = "../roms/test/mooneye/acceptance/ppu/lcdon_write_timing-GS.gb";
    acceptance_ppu_stat_irq_blocking            = "../roms/test/mooneye/acceptance/ppu/stat_irq_blocking.gb";
    acceptance_ppu_stat_lyc_onoff               = "../roms/test/mooneye/acceptance/ppu/stat_lyc_onoff.gb";
    acceptance_ppu_vblank_stat_intr_GS          = "../roms/test/mooneye/acceptance/ppu/vblank_stat_intr-GS.gb";
    acceptance_push_timing                      = "../roms/test/mooneye/acceptance/push_timing.gb";
    acceptance_rapid_di_ei                      = "../roms/test/mooneye/acceptance/rapid_di_ei.gb";
    acceptance_ret_cc_timing                    = "../roms/test/mooneye/acceptance/ret_cc_timing.gb";
    acceptance_ret_timing                       = "../roms/test/mooneye/acceptance/ret_timing.gb";
    acceptance_reti_intr_timing                 = "../roms/test/mooneye/acceptance/reti_intr_timing.gb";
    acceptance_reti_timing                      = "../roms/test/mooneye/acceptance/reti_timing.gb";
    acceptance_rst_timing                       = "../roms/test/mooneye/acceptance/rst_timing.gb";
    acceptance_serial_boot_sclk_align_dmgABCmgb = "../roms/test/mooneye/acceptance/serial/boot_sclk_align-dmgABCmgb.gb";
    acceptance_timer_div_write                  = "../roms/test/mooneye/acceptance/timer/div_write.gb";
    acceptance_timer_rapid_toggle               = "../roms/test/mooneye/acceptance/timer/rapid_toggle.gb";
    acceptance_timer_tim00                      = "../roms/test/mooneye/acceptance/timer/tim00.gb";
    acceptance_timer_tim00_div_trigger          = "../roms/test/mooneye/acceptance/timer/tim00_div_trigger.gb";
    acceptance_timer_tim01                      = "../roms/test/mooneye/acceptance/timer/tim01.gb";
    acceptance_timer_tim01_div_trigger          = "../roms/test/mooneye/acceptance/timer/tim01_div_trigger.gb";
    acceptance_timer_tim10                      = "../roms/test/mooneye/acceptance/timer/tim10.gb";
    acceptance_timer_tim10_div_trigger          = "../roms/test/mooneye/acceptance/timer/tim10_div_trigger.gb";
    acceptance_timer_tim11                      = "../roms/test/mooneye/acceptance/timer/tim11.gb";
    acceptance_timer_tim11_div_trigger          = "../roms/test/mooneye/acceptance/timer/tim11_div_trigger.gb";
    acceptance_timer_tima_reload                = "../roms/test/mooneye/acceptance/timer/tima_reload.gb";
    acceptance_timer_tima_write_reloading       = "../roms/test/mooneye/acceptance/timer/tima_write_reloading.gb";
    acceptance_timer_tma_write_reloading        = "../roms/test/mooneye/acceptance/timer/tma_write_reloading.gb";
}
