# Testing

Outlined below is current testing results for various integration test suites.

## Integration

### [Blargg](/tests/blargg.rs)

```
17 passed; 28 failed;
```

| Pass | Test                                                |
|:----:| --------------------------------------------------- |
|  ✅  | `cpu_instrs/cpu_instrs.gb`                          |
|  ✅  | `cpu_instrs/individual/01-special.gb`               |
|  ✅  | `cpu_instrs/individual/02-interrupts.gb`            |
|  ✅  | `cpu_instrs/individual/03-op sp,hl.gb`              |
|  ✅  | `cpu_instrs/individual/04-op r,imm.gb`              |
|  ✅  | `cpu_instrs/individual/05-op rp.gb`                 |
|  ✅  | `cpu_instrs/individual/06-ld r,r.gb`                |
|  ✅  | `cpu_instrs/individual/07-jr,jp,call,ret,rst.gb`    |
|  ✅  | `cpu_instrs/individual/08-misc instrs.gb`           |
|  ✅  | `cpu_instrs/individual/09-op r,r.gb`                |
|  ✅  | `cpu_instrs/individual/10-bit ops.gb`               |
|  ✅  | `cpu_instrs/individual/11-op a,(hl).gb`             |
|  ❌  | `dmg_sound/dmg_sound.gb`                            |
|  ❌  | `dmg_sound/rom_singles/01-registers.gb`             |
|  ❌  | `dmg_sound/rom_singles/02-len ctr.gb`               |
|  ❌  | `dmg_sound/rom_singles/03-trigger.gb`               |
|  ❌  | `dmg_sound/rom_singles/04-sweep.gb`                 |
|  ❌  | `dmg_sound/rom_singles/05-sweep details.gb`         |
|  ❌  | `dmg_sound/rom_singles/06-overflow on trigger.gb`   |
|  ❌  | `dmg_sound/rom_singles/07-len sweep period sync.gb` |
|  ❌  | `dmg_sound/rom_singles/08-len ctr during power.gb`  |
|  ❌  | `dmg_sound/rom_singles/09-wave read while on.gb`    |
|  ❌  | `dmg_sound/rom_singles/10-wave trigger while on.gb` |
|  ❌  | `dmg_sound/rom_singles/11-regs after power.gb`      |
|  ❌  | `dmg_sound/rom_singles/12-wave write while on.gb`   |
|  ❌  | `halt_bug.gb`                                       |
|  ✅  | `instr_timing/instr_timing.gb`                      |
|  ❌  | `interrupt_time/interrupt_time.gb`                  |
|  ✅  | `mem_timing/individual/01-read_timing.gb`           |
|  ✅  | `mem_timing/individual/02-write_timing.gb`          |
|  ✅  | `mem_timing/individual/03-modify_timing.gb`         |
|  ✅  | `mem_timing/mem_timing.gb`                          |
|  ❌  | `mem_timing-2/mem_timing.gb`                        |
|  ❌  | `mem_timing-2/rom_singles/01-read_timing.gb`        |
|  ❌  | `mem_timing-2/rom_singles/02-write_timing.gb`       |
|  ❌  | `mem_timing-2/rom_singles/03-modify_timing.gb`      |
|  ❌  | `oam_bug/oam_bug.gb`                                |
|  ❌  | `oam_bug/rom_singles/1-lcd_sync.gb`                 |
|  ❌  | `oam_bug/rom_singles/2-causes.gb`                   |
|  ❌  | `oam_bug/rom_singles/3-non_causes.gb`               |
|  ❌  | `oam_bug/rom_singles/4-scanline_timing.gb`          |
|  ❌  | `oam_bug/rom_singles/5-timing_bug.gb`               |
|  ❌  | `oam_bug/rom_singles/6-timing_no_bug.gb`            |
|  ❌  | `oam_bug/rom_singles/7-timing_effect.gb`            |
|  ❌  | `oam_bug/rom_singles/8-instr_effect.gb`             |

### [Mooneye](/tests/mooneye.rs)

```
23 passed; 46 failed;
```

| Pass | Test                                                |
|:----:| --------------------------------------------------- |
|  ❌  | `acceptance/add_sp_e_timing.gb`                     |
|  ✅  | `acceptance/bits/mem_oam.gb`                        |
|  ✅  | `acceptance/bits/reg_f.gb`                          |
|  ❌  | `acceptance/bits/unused_hwio-GS.gb`                 |
|  ❌  | `acceptance/boot_div-dmg0.gb`                       |
|  ❌  | `acceptance/boot_div-dmgABCmgb.gb`                  |
|  ❌  | `acceptance/boot_hwio-dmg0.gb`                      |
|  ❌  | `acceptance/boot_hwio-dmgABCmgb.gb`                 |
|  ❌  | `acceptance/boot_regs-dmg0.gb`                      |
|  ✅  | `acceptance/boot_regs-dmgABC.gb`                    |
|  ❌  | `acceptance/call_cc_timing.gb`                      |
|  ❌  | `acceptance/call_cc_timing2.gb`                     |
|  ❌  | `acceptance/call_timing.gb`                         |
|  ❌  | `acceptance/call_timing2.gb`                        |
|  ✅  | `acceptance/di_timing-GS.gb`                        |
|  ✅  | `acceptance/div_timing.gb`                          |
|  ✅  | `acceptance/ei_sequence.gb`                         |
|  ✅  | `acceptance/ei_timing.gb`                           |
|  ✅  | `acceptance/halt_ime0_ei.gb`                        |
|  ✅  | `acceptance/halt_ime0_nointr_timing.gb`             |
|  ✅  | `acceptance/halt_ime1_timing.gb`                    |
|  ✅  | `acceptance/halt_ime1_timing2-GS.gb`                |
|  ✅  | `acceptance/if_ie_registers.gb`                     |
|  ✅  | `acceptance/instr/daa.gb`                           |
|  ❌  | `acceptance/interrupts/ie_push.gb`                  |
|  ✅  | `acceptance/intr_timing.gb`                         |
|  ❌  | `acceptance/jp_cc_timing.gb`                        |
|  ❌  | `acceptance/jp_timing.gb`                           |
|  ❌  | `acceptance/ld_hl_sp_e_timing.gb`                   |
|  ✅  | `acceptance/oam_dma/basic.gb`                       |
|  ✅  | `acceptance/oam_dma/reg_read.gb`                    |
|  ❌  | `acceptance/oam_dma_restart.gb`                     |
|  ✅  | `acceptance/oam_dma/sources-GS.gb`                  |
|  ❌  | `acceptance/oam_dma_start.gb`                       |
|  ❌  | `acceptance/oam_dma_timing.gb`                      |
|  ✅  | `acceptance/pop_timing.gb`                          |
|  ❌  | `acceptance/ppu/hblank_ly_scx_timing-GS.gb`         |
|  ✅  | `acceptance/ppu/intr_1_2_timing-GS.gb`              |
|  ❌  | `acceptance/ppu/intr_2_0_timing.gb`                 |
|  ❌  | `acceptance/ppu/intr_2_mode0_timing.gb`             |
|  ❌  | `acceptance/ppu/intr_2_mode0_timing_sprites.gb`     |
|  ❌  | `acceptance/ppu/intr_2_mode3_timing.gb`             |
|  ❌  | `acceptance/ppu/intr_2_oam_ok_timing.gb`            |
|  ❌  | `acceptance/ppu/lcdon_timing-GS.gb`                 |
|  ❌  | `acceptance/ppu/lcdon_write_timing-GS.gb`           |
|  ❌  | `acceptance/ppu/stat_irq_blocking.gb`               |
|  ❌  | `acceptance/ppu/stat_lyc_onoff.gb`                  |
|  ❌  | `acceptance/ppu/vblank_stat_intr-GS.gb`             |
|  ❌  | `acceptance/push_timing.gb`                         |
|  ✅  | `acceptance/rapid_di_ei.gb`                         |
|  ❌  | `acceptance/ret_cc_timing.gb`                       |
|  ❌  | `acceptance/ret_timing.gb`                          |
|  ❌  | `acceptance/reti_intr_timing.gb`                    |
|  ❌  | `acceptance/reti_timing.gb`                         |
|  ❌  | `acceptance/rst_timing.gb`                          |
|  ❌  | `acceptance/serial/boot_sclk_align-dmgABCmgb.gb`    |
|  ✅  | `acceptance/timer/div_write.gb`                     |
|  ✅  | `acceptance/timer/rapid_toggle.gb`                  |
|  ❌  | `acceptance/timer/tim00.gb`                         |
|  ❌  | `acceptance/timer/tim00_div_trigger.gb`             |
|  ❌  | `acceptance/timer/tim01.gb`                         |
|  ❌  | `acceptance/timer/tim01_div_trigger.gb`             |
|  ❌  | `acceptance/timer/tim10.gb`                         |
|  ❌  | `acceptance/timer/tim10_div_trigger.gb`             |
|  ❌  | `acceptance/timer/tim11.gb`                         |
|  ❌  | `acceptance/timer/tim11_div_trigger.gb`             |
|  ❌  | `acceptance/timer/tima_reload.gb`                   |
|  ❌  | `acceptance/timer/tima_write_reloading.gb`          |
|  ✅  | `acceptance/timer/tma_write_reloading.gb`           |
