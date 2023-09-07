use std::io::Read;

use gameboy::dmg::cart::Cartridge;
use gameboy::dmg::GameBoy;
use remus::Machine;
use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

const TIMEOUT: usize = 100_000_000;

fn emulate(rom: &[u8]) -> Result<()> {
    // Instantiate a cartridge
    let cart = Cartridge::new(rom).unwrap();
    // Create an emulator instance
    let mut emu = GameBoy::new();
    // Load the cartridge
    emu.load(cart);
    // Buffer all transferred data
    let mut buf = Vec::new();

    // Loop until completion or timeout
    for _ in 0..TIMEOUT {
        emu.cycle();

        // Check for success or failure
        if passed(&mut emu, &mut buf)? {
            return Ok(());
        }
    }

    // Fail with timeout error if reached
    Err(Error::Timeout)
}

fn passed(emu: &mut GameBoy, buf: &mut Vec<u8>) -> Result<bool> {
    // Extract serial output
    emu.serial_mut().read_to_end(buf)?;
    // Calculate pass/fail conditions
    let repr = String::from_utf8_lossy(buf);
    let pass = repr.contains("Passed");
    let fail = repr.contains("Failed");
    // Check for failure
    if fail {
        return Err(Error::Failed);
    }
    // Return success
    Ok(pass)
}

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
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
    blargg_cgb_sound_cgb_sound                            = "../roms/test/blargg/cgb_sound/cgb_sound.gb";
    blargg_cgb_sound_rom_singles_01_registers             = "../roms/test/blargg/cgb_sound/rom_singles/01-registers.gb";
    blargg_cgb_sound_rom_singles_02_len_ctr               = "../roms/test/blargg/cgb_sound/rom_singles/02-len ctr.gb";
    blargg_cgb_sound_rom_singles_03_trigger               = "../roms/test/blargg/cgb_sound/rom_singles/03-trigger.gb";
    blargg_cgb_sound_rom_singles_04_sweep                 = "../roms/test/blargg/cgb_sound/rom_singles/04-sweep.gb";
    blargg_cgb_sound_rom_singles_05_sweep_details         = "../roms/test/blargg/cgb_sound/rom_singles/05-sweep details.gb";
    blargg_cgb_sound_rom_singles_06_overflow_on_trigger   = "../roms/test/blargg/cgb_sound/rom_singles/06-overflow on trigger.gb";
    blargg_cgb_sound_rom_singles_07_len_sweep_period_sync = "../roms/test/blargg/cgb_sound/rom_singles/07-len sweep period sync.gb";
    blargg_cgb_sound_rom_singles_08_len_ctr_during_power  = "../roms/test/blargg/cgb_sound/rom_singles/08-len ctr during power.gb";
    blargg_cgb_sound_rom_singles_09_wave_read_while_on    = "../roms/test/blargg/cgb_sound/rom_singles/09-wave read while on.gb";
    blargg_cgb_sound_rom_singles_10_wave_trigger_while_on = "../roms/test/blargg/cgb_sound/rom_singles/10-wave trigger while on.gb";
    blargg_cgb_sound_rom_singles_11_regs_after_power      = "../roms/test/blargg/cgb_sound/rom_singles/11-regs after power.gb";
    blargg_cgb_sound_rom_singles_12_wave                  = "../roms/test/blargg/cgb_sound/rom_singles/12-wave.gb";
    blargg_cpu_instrs_cpu_instrs                          = "../roms/test/blargg/cpu_instrs/cpu_instrs.gb";
    blargg_cpu_instrs_individual_01_special               = "../roms/test/blargg/cpu_instrs/individual/01-special.gb";
    blargg_cpu_instrs_individual_02_interrupts            = "../roms/test/blargg/cpu_instrs/individual/02-interrupts.gb";
    blargg_cpu_instrs_individual_03_op_sp_hl              = "../roms/test/blargg/cpu_instrs/individual/03-op sp,hl.gb";
    blargg_cpu_instrs_individual_04_op_r_imm              = "../roms/test/blargg/cpu_instrs/individual/04-op r,imm.gb";
    blargg_cpu_instrs_individual_05_op_rp                 = "../roms/test/blargg/cpu_instrs/individual/05-op rp.gb";
    blargg_cpu_instrs_individual_06_ld_r_r                = "../roms/test/blargg/cpu_instrs/individual/06-ld r,r.gb";
    blargg_cpu_instrs_individual_07_jr_jp_call_ret_rst    = "../roms/test/blargg/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb";
    blargg_cpu_instrs_individual_08_misc_instrs           = "../roms/test/blargg/cpu_instrs/individual/08-misc instrs.gb";
    blargg_cpu_instrs_individual_09_op_r_r                = "../roms/test/blargg/cpu_instrs/individual/09-op r,r.gb";
    blargg_cpu_instrs_individual_10_bit_ops               = "../roms/test/blargg/cpu_instrs/individual/10-bit ops.gb";
    blargg_cpu_instrs_individual_11_op_a_hl               = "../roms/test/blargg/cpu_instrs/individual/11-op a,(hl).gb";
    blargg_dmg_sound_dmg_sound                            = "../roms/test/blargg/dmg_sound/dmg_sound.gb";
    blargg_dmg_sound_rom_singles_01_registers             = "../roms/test/blargg/dmg_sound/rom_singles/01-registers.gb";
    blargg_dmg_sound_rom_singles_02_len_ctr               = "../roms/test/blargg/dmg_sound/rom_singles/02-len ctr.gb";
    blargg_dmg_sound_rom_singles_03_trigger               = "../roms/test/blargg/dmg_sound/rom_singles/03-trigger.gb";
    blargg_dmg_sound_rom_singles_04_sweep                 = "../roms/test/blargg/dmg_sound/rom_singles/04-sweep.gb";
    blargg_dmg_sound_rom_singles_05_sweep_details         = "../roms/test/blargg/dmg_sound/rom_singles/05-sweep details.gb";
    blargg_dmg_sound_rom_singles_06_overflow_on_trigger   = "../roms/test/blargg/dmg_sound/rom_singles/06-overflow on trigger.gb";
    blargg_dmg_sound_rom_singles_07_len_sweep_period_sync = "../roms/test/blargg/dmg_sound/rom_singles/07-len sweep period sync.gb";
    blargg_dmg_sound_rom_singles_08_len_ctr_during_power  = "../roms/test/blargg/dmg_sound/rom_singles/08-len ctr during power.gb";
    blargg_dmg_sound_rom_singles_09_wave_read_while_on    = "../roms/test/blargg/dmg_sound/rom_singles/09-wave read while on.gb";
    blargg_dmg_sound_rom_singles_10_wave_trigger_while_on = "../roms/test/blargg/dmg_sound/rom_singles/10-wave trigger while on.gb";
    blargg_dmg_sound_rom_singles_11_regs_after_power      = "../roms/test/blargg/dmg_sound/rom_singles/11-regs after power.gb";
    blargg_dmg_sound_rom_singles_12_wave_write_while_on   = "../roms/test/blargg/dmg_sound/rom_singles/12-wave write while on.gb";
    blargg_halt_bug                                       = "../roms/test/blargg/halt_bug.gb";
    blargg_instr_timing_instr_timing                      = "../roms/test/blargg/instr_timing/instr_timing.gb";
    blargg_interrupt_time_interrupt_time                  = "../roms/test/blargg/interrupt_time/interrupt_time.gb";
    blargg_mem_timing_individual_01_read_timing           = "../roms/test/blargg/mem_timing/individual/01-read_timing.gb";
    blargg_mem_timing_individual_02_write_timing          = "../roms/test/blargg/mem_timing/individual/02-write_timing.gb";
    blargg_mem_timing_individual_03_modify_timing         = "../roms/test/blargg/mem_timing/individual/03-modify_timing.gb";
    blargg_mem_timing_mem_timing                          = "../roms/test/blargg/mem_timing/mem_timing.gb";
    blargg_mem_timing_2_mem_timing                        = "../roms/test/blargg/mem_timing-2/mem_timing.gb";
    blargg_mem_timing_2_rom_singles_01_read_timing        = "../roms/test/blargg/mem_timing-2/rom_singles/01-read_timing.gb";
    blargg_mem_timing_2_rom_singles_02_write_timing       = "../roms/test/blargg/mem_timing-2/rom_singles/02-write_timing.gb";
    blargg_mem_timing_2_rom_singles_03_modify_timing      = "../roms/test/blargg/mem_timing-2/rom_singles/03-modify_timing.gb";
    blargg_oam_bug_oam_bug                                = "../roms/test/blargg/oam_bug/oam_bug.gb";
    blargg_oam_bug_rom_singles_1_lcd_sync                 = "../roms/test/blargg/oam_bug/rom_singles/1-lcd_sync.gb";
    blargg_oam_bug_rom_singles_2_causes                   = "../roms/test/blargg/oam_bug/rom_singles/2-causes.gb";
    blargg_oam_bug_rom_singles_3_non_causes               = "../roms/test/blargg/oam_bug/rom_singles/3-non_causes.gb";
    blargg_oam_bug_rom_singles_4_scanline_timing          = "../roms/test/blargg/oam_bug/rom_singles/4-scanline_timing.gb";
    blargg_oam_bug_rom_singles_5_timing_bug               = "../roms/test/blargg/oam_bug/rom_singles/5-timing_bug.gb";
    blargg_oam_bug_rom_singles_6_timing_no_bug            = "../roms/test/blargg/oam_bug/rom_singles/6-timing_no_bug.gb";
    blargg_oam_bug_rom_singles_7_timing_effect            = "../roms/test/blargg/oam_bug/rom_singles/7-timing_effect.gb";
    blargg_oam_bug_rom_singles_8_instr_effect             = "../roms/test/blargg/oam_bug/rom_singles/8-instr_effect.gb";
}
