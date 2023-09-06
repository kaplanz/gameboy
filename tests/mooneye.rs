#![allow(non_snake_case)]

use gameboy::core::dmg::cpu::Cpu;
use gameboy::dmg::cart::Cartridge;
use gameboy::dmg::GameBoy;
use remus::{Location, Machine};
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
    ($test:ident, $path:tt) => {
        #[test]
        fn $test() -> Result<()> {
            emulate(include_bytes!(concat!(
                "../roms/test/mooneye/",
                $path,
            )))
        }
    };
}

test!(acceptance_add_sp_e_timing, "acceptance/add_sp_e_timing.gb");
test!(acceptance_bits_mem_oam, "acceptance/bits/mem_oam.gb");
test!(acceptance_bits_reg_f, "acceptance/bits/reg_f.gb");
test!(acceptance_bits_unused_hwio_GS, "acceptance/bits/unused_hwio-GS.gb");
test!(acceptance_boot_div_dmg0, "acceptance/boot_div-dmg0.gb");
test!(acceptance_boot_div_dmgABCmgb, "acceptance/boot_div-dmgABCmgb.gb");
test!(acceptance_boot_hwio_dmg0, "acceptance/boot_hwio-dmg0.gb");
test!(acceptance_boot_hwio_dmgABCmgb, "acceptance/boot_hwio-dmgABCmgb.gb");
test!(acceptance_boot_regs_dmg0, "acceptance/boot_regs-dmg0.gb");
test!(acceptance_boot_regs_dmgABC, "acceptance/boot_regs-dmgABC.gb");
test!(acceptance_call_cc_timing, "acceptance/call_cc_timing.gb");
test!(acceptance_call_cc_timing2, "acceptance/call_cc_timing2.gb");
test!(acceptance_call_timing, "acceptance/call_timing.gb");
test!(acceptance_call_timing2, "acceptance/call_timing2.gb");
test!(acceptance_di_timing_GS, "acceptance/di_timing-GS.gb");
test!(acceptance_div_timing, "acceptance/div_timing.gb");
test!(acceptance_ei_sequence, "acceptance/ei_sequence.gb");
test!(acceptance_ei_timing, "acceptance/ei_timing.gb");
test!(acceptance_halt_ime0_ei, "acceptance/halt_ime0_ei.gb");
test!(acceptance_halt_ime0_nointr_timing, "acceptance/halt_ime0_nointr_timing.gb");
test!(acceptance_halt_ime1_timing, "acceptance/halt_ime1_timing.gb");
test!(acceptance_halt_ime1_timing2_GS, "acceptance/halt_ime1_timing2-GS.gb");
test!(acceptance_if_ie_registers, "acceptance/if_ie_registers.gb");
test!(acceptance_instr_daa, "acceptance/instr/daa.gb");
test!(acceptance_interrupts_ie_push, "acceptance/interrupts/ie_push.gb");
test!(acceptance_intr_timing, "acceptance/intr_timing.gb");
test!(acceptance_jp_cc_timing, "acceptance/jp_cc_timing.gb");
test!(acceptance_jp_timing, "acceptance/jp_timing.gb");
test!(acceptance_ld_hl_sp_e_timing, "acceptance/ld_hl_sp_e_timing.gb");
test!(acceptance_oam_dma_basic, "acceptance/oam_dma/basic.gb");
test!(acceptance_oam_dma_reg_read, "acceptance/oam_dma/reg_read.gb");
test!(acceptance_oam_dma_restart, "acceptance/oam_dma_restart.gb");
test!(acceptance_oam_dma_sources_GS, "acceptance/oam_dma/sources-GS.gb");
test!(acceptance_oam_dma_start, "acceptance/oam_dma_start.gb");
test!(acceptance_oam_dma_timing, "acceptance/oam_dma_timing.gb");
test!(acceptance_pop_timing, "acceptance/pop_timing.gb");
test!(acceptance_ppu_hblank_ly_scx_timing_GS, "acceptance/ppu/hblank_ly_scx_timing-GS.gb");
test!(acceptance_ppu_intr_1_2_timing_GS, "acceptance/ppu/intr_1_2_timing-GS.gb");
test!(acceptance_ppu_intr_2_0_timing, "acceptance/ppu/intr_2_0_timing.gb");
test!(acceptance_ppu_intr_2_mode0_timing, "acceptance/ppu/intr_2_mode0_timing.gb");
test!(acceptance_ppu_intr_2_mode0_timing_sprites, "acceptance/ppu/intr_2_mode0_timing_sprites.gb");
test!(acceptance_ppu_intr_2_mode3_timing, "acceptance/ppu/intr_2_mode3_timing.gb");
test!(acceptance_ppu_intr_2_oam_ok_timing, "acceptance/ppu/intr_2_oam_ok_timing.gb");
test!(acceptance_ppu_lcdon_timing_GS, "acceptance/ppu/lcdon_timing-GS.gb");
test!(acceptance_ppu_lcdon_write_timing_GS, "acceptance/ppu/lcdon_write_timing-GS.gb");
test!(acceptance_ppu_stat_irq_blocking, "acceptance/ppu/stat_irq_blocking.gb");
test!(acceptance_ppu_stat_lyc_onoff, "acceptance/ppu/stat_lyc_onoff.gb");
test!(acceptance_ppu_vblank_stat_intr_GS, "acceptance/ppu/vblank_stat_intr-GS.gb");
test!(acceptance_push_timing, "acceptance/push_timing.gb");
test!(acceptance_rapid_di_ei, "acceptance/rapid_di_ei.gb");
test!(acceptance_ret_cc_timing, "acceptance/ret_cc_timing.gb");
test!(acceptance_ret_timing, "acceptance/ret_timing.gb");
test!(acceptance_reti_intr_timing, "acceptance/reti_intr_timing.gb");
test!(acceptance_reti_timing, "acceptance/reti_timing.gb");
test!(acceptance_rst_timing, "acceptance/rst_timing.gb");
test!(acceptance_serial_boot_sclk_align_dmgABCmgb, "acceptance/serial/boot_sclk_align-dmgABCmgb.gb");
test!(acceptance_timer_div_write, "acceptance/timer/div_write.gb");
test!(acceptance_timer_rapid_toggle, "acceptance/timer/rapid_toggle.gb");
test!(acceptance_timer_tim00, "acceptance/timer/tim00.gb");
test!(acceptance_timer_tim00_div_trigger, "acceptance/timer/tim00_div_trigger.gb");
test!(acceptance_timer_tim01, "acceptance/timer/tim01.gb");
test!(acceptance_timer_tim01_div_trigger, "acceptance/timer/tim01_div_trigger.gb");
test!(acceptance_timer_tim10, "acceptance/timer/tim10.gb");
test!(acceptance_timer_tim10_div_trigger, "acceptance/timer/tim10_div_trigger.gb");
test!(acceptance_timer_tim11, "acceptance/timer/tim11.gb");
test!(acceptance_timer_tim11_div_trigger, "acceptance/timer/tim11_div_trigger.gb");
test!(acceptance_timer_tima_reload, "acceptance/timer/tima_reload.gb");
test!(acceptance_timer_tima_write_reloading, "acceptance/timer/tima_write_reloading.gb");
test!(acceptance_timer_tma_write_reloading, "acceptance/timer/tma_write_reloading.gb");
