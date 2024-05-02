# Testing

Outlined below is current testing results for various integration test suites.

## Integration

### Acid2

[![acid2 origin][acid2.git.badge]][acid2.git.hyper]
[![acid2 source][acid2.src.badge]][acid2.src.hyper]

A clever re-imagining of the [Web Standards Project's Acid2][wsp.acid2]
rendering test for the Game Boy's PPU.

#### Results

```
0 passed; 17 failed;
```

| Pass | Test                                                |
|:----:| --------------------------------------------------- |
|  ❌  | `failure_10_obj_limit`                              |
|  ❌  | `failure_8x16_obj_tile_index_bit_0`                 |
|  ❌  | `failure_bg_enable`                                 |
|  ❌  | `failure_bg_map`                                    |
|  ❌  | `failure_obj_enable`                                |
|  ❌  | `failure_obj_horizontal_flip`                       |
|  ❌  | `failure_obj_palette`                               |
|  ❌  | `failure_obj_priority_lower_x`                      |
|  ❌  | `failure_obj_priority_same_x`                       |
|  ❌  | `failure_obj_size`                                  |
|  ❌  | `failure_obj_to_bg_priority`                        |
|  ❌  | `failure_obj_vertical_flip`                         |
|  ❌  | `failure_tile_sel`                                  |
|  ❌  | `failure_win_enable`                                |
|  ❌  | `failure_win_line_counter`                          |
|  ❌  | `failure_win_map`                                   |
|  ❌  | `success`                                           |

---

### Blargg

[![blargg origin][blargg.git.badge]][blargg.git.hyper]
[![blargg source][blargg.src.badge]][blargg.src.hyper]

Shay Green's classic suite of Game Boy testing ROMs, checking a wide variety of
behaviours.

#### Results

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

---

### Mealybug

[![mealybug origin][mealybug.git.badge]][mealybug.git.hyper]
[![mealybug source][mealybug.src.badge]][mealybug.src.hyper]

Correctness tests focusing on runtime changes made to the PPU.

#### Results

```
0 passed; 24 failed;
```

| Pass | Test                                                |
|:----:| --------------------------------------------------- |
|  ❌  | `m2_win_en_toggle`                                  |
|  ❌  | `m3_bgp_change`                                     |
|  ❌  | `m3_bgp_change_sprites`                             |
|  ❌  | `m3_lcdc_bg_en_change`                              |
|  ❌  | `m3_lcdc_bg_map_change`                             |
|  ❌  | `m3_lcdc_obj_en_change`                             |
|  ❌  | `m3_lcdc_obj_en_change_variant`                     |
|  ❌  | `m3_lcdc_obj_size_change`                           |
|  ❌  | `m3_lcdc_obj_size_change_scx`                       |
|  ❌  | `m3_lcdc_tile_sel_change`                           |
|  ❌  | `m3_lcdc_tile_sel_win_change`                       |
|  ❌  | `m3_lcdc_win_en_change_multiple`                    |
|  ❌  | `m3_lcdc_win_en_change_multiple_wx`                 |
|  ❌  | `m3_lcdc_win_map_change`                            |
|  ❌  | `m3_obp0_change`                                    |
|  ❌  | `m3_scx_high_5_bits`                                |
|  ❌  | `m3_scx_low_3_bits`                                 |
|  ❌  | `m3_scy_change`                                     |
|  ❌  | `m3_window_timing`                                  |
|  ❌  | `m3_window_timing_wx_0`                             |
|  ❌  | `m3_wx_4_change`                                    |
|  ❌  | `m3_wx_4_change_sprites`                            |
|  ❌  | `m3_wx_5_change`                                    |
|  ❌  | `m3_wx_6_change`                                    |

---

### Mooneye

[![mooneye origin][mooneye.git.badge]][mooneye.git.hyper]
[![mooneye source][mooneye.src.badge]][mooneye.src.hyper]

Detailed tests on precise characteristics easily verifiable on actual hardware.

#### Results

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

---

## Attribution

Attribution of all included open-source software is listed in the project's
[README][attrib].

<!--
  Reference-style links
-->

<!-- Badges -->
[acid2.git.badge]:    https://img.shields.io/badge/acid2-origin-2188a7?logo=github
[acid2.git.hyper]:    https://github.com/mattcurrie/dmg-acid2
[acid2.src.badge]:    https://img.shields.io/badge/acid2-source-a72145?logo=rust
[acid2.src.hyper]:    /tests/acid2.rs
[blargg.git.badge]:   https://img.shields.io/badge/blargg-origin-2188a7?logo=github
[blargg.git.hyper]:   https://github.com/retrio/gb-test-roms
[blargg.src.badge]:   https://img.shields.io/badge/blargg-source-a72145?logo=rust
[blargg.src.hyper]:   /tests/blargg.rs
[mealybug.git.badge]: https://img.shields.io/badge/mealybug-origin-2188a7?logo=github
[mealybug.git.hyper]: https://github.com/mattcurrie/mealybug-tearoom-tests
[mealybug.src.badge]: https://img.shields.io/badge/mealybug-source-a72145?logo=rust
[mealybug.src.hyper]: /tests/mealybug.rs
[mooneye.git.badge]:  https://img.shields.io/badge/mooneye-origin-2188a7?logo=github
[mooneye.git.hyper]:  https://github.com/Gekkio/mooneye-test-suite
[mooneye.src.badge]:  https://img.shields.io/badge/mooneye-source-a72145?logo=rust
[mooneye.src.hyper]:  /tests/mooneye.rs

<!-- Integration -->
[wsp.acid2]: https://webstandards.org/files/acid2/test.html

<!-- Attribution -->
[attrib]: /README.md#attribution
