# gameboy

<p align="center">
  <img width="100" height="100" src="./docs/assets/img/gameboy.svg"/>
</p>

<p align="center">
  <q>
    A delayed game is eventually good, but a rushed game is forever bad.
  </q>
  &mdash;
  <i>
    Shigeru Miyamoto
  </i>
</p>

---

A cycle accurate emulator of the original 1989 Nintendo Game Boy.

## Progress

### Core

- [ ] Implement audio
- [ ] Implement cartridges
  - [ ] Save RAM to disk
  - [ ] Support MBCs
    - [x] MBC1
    - [ ] MBC3
- [x] Implement CPU
  - [x] Instruction correctness
  - [x] Cycle accuracy
  - [x] Timed memory accesses
- [x] Implement interrupts
- [x] Implement joypad
- [x] Implement PPU
  - [x] Background drawing
  - [x] Window drawing
  - [x] Sprite rendering
- [ ] Implement timer
  - [x] Functional correctness
  - [ ] Implementation accuracy
- [ ] Performance enhancements
  - [ ] Benchmark tests

### Application

- [x] Basic app
  - [x] Screen pixel buffer window
  - [x] Configurable palette
  - [x] Static debug mode
- [ ] Full GUI
  - [ ] Dynamic cycle speed modifiers
  - [ ] Re-mappable joypad
  - [ ] Interactive debug menu
