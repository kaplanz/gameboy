#![allow(non_snake_case)]

use remus::{Location, Machine};
use rugby::core::dmg::cart::Cartridge;
use rugby::core::dmg::cpu::Cpu;
use rugby::core::dmg::GameBoy;
use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

const TIMEOUT: usize = 10_000_000;

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
        if passed(&emu)? {
            return Ok(());
        }
    }

    // Fail with timeout error if reached
    Err(Error::Timeout)
}

fn passed(emu: &GameBoy) -> Result<bool> {
    type Register = <Cpu as Location<u16>>::Register;
    // Extract register values
    let cpu = emu.cpu();
    let bc: u16 = cpu.load(Register::BC);
    let de: u16 = cpu.load(Register::DE);
    let hl: u16 = cpu.load(Register::HL);
    // Calculate pass/fail conditions
    let pass = (bc == 0x0305) && (de == 0x080d) && (hl == 0x1522);
    let fail = (bc == 0x4242) && (de == 0x4242) && (hl == 0x4242);
    // Check for failure
    if fail {
        return Err(Error::Failed);
    }
    // Return success
    Ok(pass)
}

#[derive(Debug, Error)]
enum Error {
    #[error("failed test")]
    Failed,
    #[error("timeout reached")]
    Timeout,
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
    mooneye_acceptance_add_sp_e_timing                  = "../roms/test/mooneye/acceptance/add_sp_e_timing.gb";
    mooneye_acceptance_bits_mem_oam                     = "../roms/test/mooneye/acceptance/bits/mem_oam.gb";
    mooneye_acceptance_bits_reg_f                       = "../roms/test/mooneye/acceptance/bits/reg_f.gb";
    mooneye_acceptance_bits_unused_hwio_GS              = "../roms/test/mooneye/acceptance/bits/unused_hwio-GS.gb";
    mooneye_acceptance_boot_div_dmg0                    = "../roms/test/mooneye/acceptance/boot_div-dmg0.gb";
    mooneye_acceptance_boot_div_dmgABCmgb               = "../roms/test/mooneye/acceptance/boot_div-dmgABCmgb.gb";
    mooneye_acceptance_boot_hwio_dmg0                   = "../roms/test/mooneye/acceptance/boot_hwio-dmg0.gb";
    mooneye_acceptance_boot_hwio_dmgABCmgb              = "../roms/test/mooneye/acceptance/boot_hwio-dmgABCmgb.gb";
    mooneye_acceptance_boot_regs_dmg0                   = "../roms/test/mooneye/acceptance/boot_regs-dmg0.gb";
    mooneye_acceptance_boot_regs_dmgABC                 = "../roms/test/mooneye/acceptance/boot_regs-dmgABC.gb";
    mooneye_acceptance_call_cc_timing                   = "../roms/test/mooneye/acceptance/call_cc_timing.gb";
    mooneye_acceptance_call_cc_timing2                  = "../roms/test/mooneye/acceptance/call_cc_timing2.gb";
    mooneye_acceptance_call_timing                      = "../roms/test/mooneye/acceptance/call_timing.gb";
    mooneye_acceptance_call_timing2                     = "../roms/test/mooneye/acceptance/call_timing2.gb";
    mooneye_acceptance_di_timing_GS                     = "../roms/test/mooneye/acceptance/di_timing-GS.gb";
    mooneye_acceptance_div_timing                       = "../roms/test/mooneye/acceptance/div_timing.gb";
    mooneye_acceptance_ei_sequence                      = "../roms/test/mooneye/acceptance/ei_sequence.gb";
    mooneye_acceptance_ei_timing                        = "../roms/test/mooneye/acceptance/ei_timing.gb";
    mooneye_acceptance_halt_ime0_ei                     = "../roms/test/mooneye/acceptance/halt_ime0_ei.gb";
    mooneye_acceptance_halt_ime0_nointr_timing          = "../roms/test/mooneye/acceptance/halt_ime0_nointr_timing.gb";
    mooneye_acceptance_halt_ime1_timing                 = "../roms/test/mooneye/acceptance/halt_ime1_timing.gb";
    mooneye_acceptance_halt_ime1_timing2_GS             = "../roms/test/mooneye/acceptance/halt_ime1_timing2-GS.gb";
    mooneye_acceptance_if_ie_registers                  = "../roms/test/mooneye/acceptance/if_ie_registers.gb";
    mooneye_acceptance_instr_daa                        = "../roms/test/mooneye/acceptance/instr/daa.gb";
    mooneye_acceptance_interrupts_ie_push               = "../roms/test/mooneye/acceptance/interrupts/ie_push.gb";
    mooneye_acceptance_intr_timing                      = "../roms/test/mooneye/acceptance/intr_timing.gb";
    mooneye_acceptance_jp_cc_timing                     = "../roms/test/mooneye/acceptance/jp_cc_timing.gb";
    mooneye_acceptance_jp_timing                        = "../roms/test/mooneye/acceptance/jp_timing.gb";
    mooneye_acceptance_ld_hl_sp_e_timing                = "../roms/test/mooneye/acceptance/ld_hl_sp_e_timing.gb";
    mooneye_acceptance_oam_dma_basic                    = "../roms/test/mooneye/acceptance/oam_dma/basic.gb";
    mooneye_acceptance_oam_dma_reg_read                 = "../roms/test/mooneye/acceptance/oam_dma/reg_read.gb";
    mooneye_acceptance_oam_dma_restart                  = "../roms/test/mooneye/acceptance/oam_dma_restart.gb";
    mooneye_acceptance_oam_dma_sources_GS               = "../roms/test/mooneye/acceptance/oam_dma/sources-GS.gb";
    mooneye_acceptance_oam_dma_start                    = "../roms/test/mooneye/acceptance/oam_dma_start.gb";
    mooneye_acceptance_oam_dma_timing                   = "../roms/test/mooneye/acceptance/oam_dma_timing.gb";
    mooneye_acceptance_pop_timing                       = "../roms/test/mooneye/acceptance/pop_timing.gb";
    mooneye_acceptance_ppu_hblank_ly_scx_timing_GS      = "../roms/test/mooneye/acceptance/ppu/hblank_ly_scx_timing-GS.gb";
    mooneye_acceptance_ppu_intr_1_2_timing_GS           = "../roms/test/mooneye/acceptance/ppu/intr_1_2_timing-GS.gb";
    mooneye_acceptance_ppu_intr_2_0_timing              = "../roms/test/mooneye/acceptance/ppu/intr_2_0_timing.gb";
    mooneye_acceptance_ppu_intr_2_mode0_timing          = "../roms/test/mooneye/acceptance/ppu/intr_2_mode0_timing.gb";
    mooneye_acceptance_ppu_intr_2_mode0_timing_sprites  = "../roms/test/mooneye/acceptance/ppu/intr_2_mode0_timing_sprites.gb";
    mooneye_acceptance_ppu_intr_2_mode3_timing          = "../roms/test/mooneye/acceptance/ppu/intr_2_mode3_timing.gb";
    mooneye_acceptance_ppu_intr_2_oam_ok_timing         = "../roms/test/mooneye/acceptance/ppu/intr_2_oam_ok_timing.gb";
    mooneye_acceptance_ppu_lcdon_timing_GS              = "../roms/test/mooneye/acceptance/ppu/lcdon_timing-GS.gb";
    mooneye_acceptance_ppu_lcdon_write_timing_GS        = "../roms/test/mooneye/acceptance/ppu/lcdon_write_timing-GS.gb";
    mooneye_acceptance_ppu_stat_irq_blocking            = "../roms/test/mooneye/acceptance/ppu/stat_irq_blocking.gb";
    mooneye_acceptance_ppu_stat_lyc_onoff               = "../roms/test/mooneye/acceptance/ppu/stat_lyc_onoff.gb";
    mooneye_acceptance_ppu_vblank_stat_intr_GS          = "../roms/test/mooneye/acceptance/ppu/vblank_stat_intr-GS.gb";
    mooneye_acceptance_push_timing                      = "../roms/test/mooneye/acceptance/push_timing.gb";
    mooneye_acceptance_rapid_di_ei                      = "../roms/test/mooneye/acceptance/rapid_di_ei.gb";
    mooneye_acceptance_ret_cc_timing                    = "../roms/test/mooneye/acceptance/ret_cc_timing.gb";
    mooneye_acceptance_ret_timing                       = "../roms/test/mooneye/acceptance/ret_timing.gb";
    mooneye_acceptance_reti_intr_timing                 = "../roms/test/mooneye/acceptance/reti_intr_timing.gb";
    mooneye_acceptance_reti_timing                      = "../roms/test/mooneye/acceptance/reti_timing.gb";
    mooneye_acceptance_rst_timing                       = "../roms/test/mooneye/acceptance/rst_timing.gb";
    mooneye_acceptance_serial_boot_sclk_align_dmgABCmgb = "../roms/test/mooneye/acceptance/serial/boot_sclk_align-dmgABCmgb.gb";
    mooneye_acceptance_timer_div_write                  = "../roms/test/mooneye/acceptance/timer/div_write.gb";
    mooneye_acceptance_timer_rapid_toggle               = "../roms/test/mooneye/acceptance/timer/rapid_toggle.gb";
    mooneye_acceptance_timer_tim00                      = "../roms/test/mooneye/acceptance/timer/tim00.gb";
    mooneye_acceptance_timer_tim00_div_trigger          = "../roms/test/mooneye/acceptance/timer/tim00_div_trigger.gb";
    mooneye_acceptance_timer_tim01                      = "../roms/test/mooneye/acceptance/timer/tim01.gb";
    mooneye_acceptance_timer_tim01_div_trigger          = "../roms/test/mooneye/acceptance/timer/tim01_div_trigger.gb";
    mooneye_acceptance_timer_tim10                      = "../roms/test/mooneye/acceptance/timer/tim10.gb";
    mooneye_acceptance_timer_tim10_div_trigger          = "../roms/test/mooneye/acceptance/timer/tim10_div_trigger.gb";
    mooneye_acceptance_timer_tim11                      = "../roms/test/mooneye/acceptance/timer/tim11.gb";
    mooneye_acceptance_timer_tim11_div_trigger          = "../roms/test/mooneye/acceptance/timer/tim11_div_trigger.gb";
    mooneye_acceptance_timer_tima_reload                = "../roms/test/mooneye/acceptance/timer/tima_reload.gb";
    mooneye_acceptance_timer_tima_write_reloading       = "../roms/test/mooneye/acceptance/timer/tima_write_reloading.gb";
    mooneye_acceptance_timer_tma_write_reloading        = "../roms/test/mooneye/acceptance/timer/tma_write_reloading.gb";
}
