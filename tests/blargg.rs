use std::fmt::{Debug, Display};

use rugby::arch::Block;
use rugby::core::dmg::{Cartridge, GameBoy};
use rugby::prelude::*;
use thiserror::Error;

/// Number of cycles after which the test is considered to have failed due to a
/// timeout error.
const TIMEOUT: usize = 250_000_000;
/// Period (in cycles) between checks for success.
const PERIOD: usize = 10_000;

/// Perform integration test emulation.
fn emulate(rom: &[u8], check: fn(&mut GameBoy) -> Result<()>) -> Result<()> {
    // Instantiate a cartridge
    let cart = Cartridge::new(rom).unwrap();
    // Create an emulator instance
    let mut emu = GameBoy::new();
    // Load the cartridge
    emu.insert(cart);
    // Write in-progress sentinel
    emu.inside_mut().proc().write(0xa000, 0x80);

    // Loop until completion or timeout
    for cycle in 0..TIMEOUT {
        emu.cycle();

        // Await end of period...
        if cycle % PERIOD == 0 {
            // ... then check for success or failure
            match check(&mut emu) {
                Err(Error::Running) => continue,
                res => return res,
            }
        }
    }

    // Fail with timeout if reached
    Err(Error::Timeout)
}

/// Check for test results.
mod check {
    use std::io::BufRead;

    use rugby::core::dmg::GameBoy;
    use rugby::prelude::*;

    use super::{Error, Result};

    /// Check console for tests results.
    ///
    /// # About
    ///
    /// Information is printed on screen in a way that needs only minimum LCD
    /// support, and won't hang if LCD output isn't supported at all.
    /// Specifically, while polling LY to wait for vblank, it will time out if
    /// it takes too long, so LY always reading back as the same value won't
    /// hang the test. It's also OK if scrolling isn't supported; in this case,
    /// text will appear starting at the top of the screen.
    ///
    /// Everything printed on screen is also sent to the game link port by
    /// writing the character to SB, then writing $81 to SC. This is useful for
    /// tests which print lots of information that scrolls off screen.
    pub fn console(emu: &mut GameBoy) -> Result<()> {
        // Extract serial output
        let buf = emu.inside_mut().serial().rx().fill_buf()?;
        // Calculate pass/fail conditions
        let repr = String::from_utf8_lossy(buf);
        let pass = repr.contains("Passed");
        let fail = repr.contains("Failed");
        // Report results
        if fail {
            Err(Error::Failed)
        } else if pass {
            Ok(())
        } else {
            Err(Error::Running)
        }
    }

    /// Check memory for test results.
    ///
    /// # About
    ///
    /// Text output and the final result are also written to memory at $A000,
    /// allowing testing a very minimal emulator that supports little more than
    /// CPU and RAM. To reliably indicate that the data is from a test and not
    /// random data, $A001-$A003 are written with a signature: $DE,$B0,$61. If
    /// this is present, then the text string and final result status are valid.
    ///
    /// $A000 holds the overall status. If the test is still running, it holds
    /// $80, otherwise it holds the final result code.
    ///
    /// All text output is appended to a zero-terminated string at $A004. An
    /// emulator could regularly check this string for any additional
    /// characters, and output them, allowing real-time text output, rather than
    /// just printing the final output at the end.
    pub fn memory(emu: &mut GameBoy) -> Result<()> {
        // Extract memory output
        let res = emu.inside().proc().read(0xa000);
        let chk = [0xa001, 0xa002, 0xa003].map(|addr| emu.inside().proc().read(addr));
        // Calculate pass/fail conditions
        let chkd = chk == [0xde, 0xb0, 0x61];
        let done = chkd && res != 0x80;
        let pass = done && res == 0;
        let fail = done && res != 0;
        // Report results
        if fail {
            Err(Error::Failure(res))
        } else if pass {
            Ok(())
        } else {
            Err(Error::Running)
        }
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
    /// Test has explicitly failed with an error code.
    #[error("test failed with code: {0}")]
    Failure(u8),
    /// An I/O error occurred.
    #[error(transparent)]
    Ioput(#[from] std::io::Error),
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
    ($($test:ident = ($check:path, $path:tt);)*) => {
        $(
            #[test]
            fn $test() -> Result<()> {
                emulate(include_bytes!($path), $check)
            }
        )*
    };
}

test! {
    cpu_instrs_cpu_instrs                          = (check::console, "../roms/test/blargg/cpu_instrs/cpu_instrs.gb");
    cpu_instrs_individual_01_special               = (check::console, "../roms/test/blargg/cpu_instrs/individual/01-special.gb");
    cpu_instrs_individual_02_interrupts            = (check::console, "../roms/test/blargg/cpu_instrs/individual/02-interrupts.gb");
    cpu_instrs_individual_03_op_sp_hl              = (check::console, "../roms/test/blargg/cpu_instrs/individual/03-op sp,hl.gb");
    cpu_instrs_individual_04_op_r_imm              = (check::console, "../roms/test/blargg/cpu_instrs/individual/04-op r,imm.gb");
    cpu_instrs_individual_05_op_rp                 = (check::console, "../roms/test/blargg/cpu_instrs/individual/05-op rp.gb");
    cpu_instrs_individual_06_ld_r_r                = (check::console, "../roms/test/blargg/cpu_instrs/individual/06-ld r,r.gb");
    cpu_instrs_individual_07_jr_jp_call_ret_rst    = (check::console, "../roms/test/blargg/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb");
    cpu_instrs_individual_08_misc_instrs           = (check::console, "../roms/test/blargg/cpu_instrs/individual/08-misc instrs.gb");
    cpu_instrs_individual_09_op_r_r                = (check::console, "../roms/test/blargg/cpu_instrs/individual/09-op r,r.gb");
    cpu_instrs_individual_10_bit_ops               = (check::console, "../roms/test/blargg/cpu_instrs/individual/10-bit ops.gb");
    cpu_instrs_individual_11_op_a_hl               = (check::console, "../roms/test/blargg/cpu_instrs/individual/11-op a,(hl).gb");
    dmg_sound_dmg_sound                            = (check::memory,  "../roms/test/blargg/dmg_sound/dmg_sound.gb");
    dmg_sound_rom_singles_01_registers             = (check::memory,  "../roms/test/blargg/dmg_sound/rom_singles/01-registers.gb");
    dmg_sound_rom_singles_02_len_ctr               = (check::memory,  "../roms/test/blargg/dmg_sound/rom_singles/02-len ctr.gb");
    dmg_sound_rom_singles_03_trigger               = (check::memory,  "../roms/test/blargg/dmg_sound/rom_singles/03-trigger.gb");
    dmg_sound_rom_singles_04_sweep                 = (check::memory,  "../roms/test/blargg/dmg_sound/rom_singles/04-sweep.gb");
    dmg_sound_rom_singles_05_sweep_details         = (check::memory,  "../roms/test/blargg/dmg_sound/rom_singles/05-sweep details.gb");
    dmg_sound_rom_singles_06_overflow_on_trigger   = (check::memory,  "../roms/test/blargg/dmg_sound/rom_singles/06-overflow on trigger.gb");
    dmg_sound_rom_singles_07_len_sweep_period_sync = (check::memory,  "../roms/test/blargg/dmg_sound/rom_singles/07-len sweep period sync.gb");
    dmg_sound_rom_singles_08_len_ctr_during_power  = (check::memory,  "../roms/test/blargg/dmg_sound/rom_singles/08-len ctr during power.gb");
    dmg_sound_rom_singles_09_wave_read_while_on    = (check::memory,  "../roms/test/blargg/dmg_sound/rom_singles/09-wave read while on.gb");
    dmg_sound_rom_singles_10_wave_trigger_while_on = (check::memory,  "../roms/test/blargg/dmg_sound/rom_singles/10-wave trigger while on.gb");
    dmg_sound_rom_singles_11_regs_after_power      = (check::memory,  "../roms/test/blargg/dmg_sound/rom_singles/11-regs after power.gb");
    dmg_sound_rom_singles_12_wave_write_while_on   = (check::memory,  "../roms/test/blargg/dmg_sound/rom_singles/12-wave write while on.gb");
    halt_bug                                       = (check::console, "../roms/test/blargg/halt_bug.gb");
    instr_timing_instr_timing                      = (check::console, "../roms/test/blargg/instr_timing/instr_timing.gb");
    interrupt_time_interrupt_time                  = (check::console, "../roms/test/blargg/interrupt_time/interrupt_time.gb");
    mem_timing_individual_01_read_timing           = (check::console, "../roms/test/blargg/mem_timing/individual/01-read_timing.gb");
    mem_timing_individual_02_write_timing          = (check::console, "../roms/test/blargg/mem_timing/individual/02-write_timing.gb");
    mem_timing_individual_03_modify_timing         = (check::console, "../roms/test/blargg/mem_timing/individual/03-modify_timing.gb");
    mem_timing_mem_timing                          = (check::console, "../roms/test/blargg/mem_timing/mem_timing.gb");
    mem_timing_2_mem_timing                        = (check::memory,  "../roms/test/blargg/mem_timing-2/mem_timing.gb");
    mem_timing_2_rom_singles_01_read_timing        = (check::memory,  "../roms/test/blargg/mem_timing-2/rom_singles/01-read_timing.gb");
    mem_timing_2_rom_singles_02_write_timing       = (check::memory,  "../roms/test/blargg/mem_timing-2/rom_singles/02-write_timing.gb");
    mem_timing_2_rom_singles_03_modify_timing      = (check::memory,  "../roms/test/blargg/mem_timing-2/rom_singles/03-modify_timing.gb");
    oam_bug_oam_bug                                = (check::memory,  "../roms/test/blargg/oam_bug/oam_bug.gb");
    oam_bug_rom_singles_1_lcd_sync                 = (check::memory,  "../roms/test/blargg/oam_bug/rom_singles/1-lcd_sync.gb");
    oam_bug_rom_singles_2_causes                   = (check::memory,  "../roms/test/blargg/oam_bug/rom_singles/2-causes.gb");
    oam_bug_rom_singles_3_non_causes               = (check::memory,  "../roms/test/blargg/oam_bug/rom_singles/3-non_causes.gb");
    oam_bug_rom_singles_4_scanline_timing          = (check::memory,  "../roms/test/blargg/oam_bug/rom_singles/4-scanline_timing.gb");
    oam_bug_rom_singles_5_timing_bug               = (check::memory,  "../roms/test/blargg/oam_bug/rom_singles/5-timing_bug.gb");
    oam_bug_rom_singles_6_timing_no_bug            = (check::memory,  "../roms/test/blargg/oam_bug/rom_singles/6-timing_no_bug.gb");
    oam_bug_rom_singles_7_timing_effect            = (check::memory,  "../roms/test/blargg/oam_bug/rom_singles/7-timing_effect.gb");
    oam_bug_rom_singles_8_instr_effect             = (check::memory,  "../roms/test/blargg/oam_bug/rom_singles/8-instr_effect.gb");
}
