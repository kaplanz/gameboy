# rugby

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

[![dependency status][deps.badge]][deps.hyper]

A cycle accurate emulator of the original 1989 Nintendo Game Boy.

> [!IMPORTANT]
>
> Consider this to be pre-alpha software. I make no guarantee of the stability
> of any documented APIs until the first official release.

## Goal

This project aims to provide high-accuracy emulation of all major components in
the SM83-based Nintendo Game Boy family of consoles. Each component is modular,
enabling them to be easily substituted for one another. In turn, this leads to
emulator instances supporting multiple implementations of core components with
different accuracy/performance margins.

> [!NOTE]
>
> At this time, there are no plans to support the later Game Boy Advance, which
> has an entirely different architecture.

## Organization

In accordance with the preference for modularity outlined above, the project is
partitioned into the [core](./core), and various emulator [frontends](./apps).
Useful supporting [crates](./crates) are independently available as well. Also
packaged in this repository are several open-source [ROMs](./roms). These are
used internally for testing and to demo project functionality.

### Workspace

Cargo — Rust's package manager — allows for a workspace of several crates to be
specified within its [manifest](./Cargo.toml). Within this project, workspace
crates are used with the structure as follows:

```
./
├── Cargo.lock       # cargo lockfile
├── Cargo.toml       # cargo manifest
├── Justfile         # useful development commands
├── README.md        # this document
├── ...
├── apps/            # frontend apps
├── core/            # emulation core
├── crates/          # support crates
├── docs/            # documentation
├── roms/            # open-source ROMs
├── src/             # emulation library
└── tests/           # integration tests
```

> [!TIP]
>
> For downstream library users, I recommend using the top-level `rugby` crate
> rather than directly using `rugby-core`, as it better structured for end
> users and includes useful supporting modules.

## Progress

### Core

- [ ] Implementation
  - [ ] Audio (APU)
  - [x] Cartridges
    - [ ] Save RAM to disk
    - [ ] Support hardware
      - [x] MBC1
      - [ ] MBC3
      - [x] MBC5
  - [x] Interrupts (PIC)
  - [x] Graphics (PPU)
    - [x] Functional correctness
      - [x] Background drawing
      - [x] Window overlay
      - [x] Sprite rendering
    - [ ] Implementation accuracy
  - [x] Joypad
  - [x] Processor (CPU)
    - [x] Functional correctness
    - [ ] Implementation accuracy
      - [x] Multi-cycle instructions
      - [x] Timed memory accesses
      - [ ] Timing precision
  - [x] Timer
    - [x] Functional correctness
    - [x] Implementation accuracy
- [ ] Performance enhancements
  - [x] Real-time emulation
  - [ ] Benchmark tests
- [ ] [Testing][docs.testing]
  - [ ] Blargg (17/45)
  - [ ] Mooneye (23/69)

### Library

- [x] Color palette API
- [x] Debugger API
  - [x] Breakpoints
    - [x] Instruction address (PC)
    - [ ] Instruction opcode
    - [ ] I/O operation (read/write)
    - [ ] Softbreak (`ld b, b`)
    - [ ] Dynamic condition
  - [x] Memory peek/poke
  - [x] Register manipulation
    - [ ] APU
    - [x] CPU
    - [x] Interrupts
    - [x] PPU
    - [x] Serial
    - [x] Timer
  - [x] Peripheral control
    - [x] Serial interface
- [x] Exposed core API
- [x] Frontend API

### Frontend

- [x] Command-line (CLI)
  - [x] Configuration file
    - [ ] Re-mappable joypad
  - [x] Debugging support
    - [x] Interactive debugging
    - [x] CPU logging
    - [x] Memory rendering
    - [ ] Dynamic speed changing
- [ ] Cross platform (SDL)
- [ ] Native
  - [ ] iOS
  - [ ] macOS
- [ ] Web (WASM)

## References

This project would not have been possible without the countless Game Boy
community resources. Of these, I would like to specifically recognize the [Game
Boy Development community][gbdev].

See the list of resources (in no particular order) used as research for this
project below.

### Documentation

- [Pan Docs][pandocs]: Go-to community resource documenting the inner workings
  of the Game Boy. "The single most comprehensive technical reference to Game
  Boy available to the public."
- [Game Boy Architecture by Rodrigo Copetti][gbarch]: High-level practical
  analysis of the Game Boy.
  - Includes a helpful introduction to the PPU rendering pipeline.
- [Game Boy: Complete Technical Reference][gbctr]: Summation of [Gekkio]'s
  comprehensive Game Boy research.
  - Used for exact instruction timing breakdown.
- [The Gameboy Emulator Development Guide][gbedg]: Documentation intended to
  assist with development of emulators for the original DMG Game Boy.
  - Used extensively for the initial PPU and timer implementations.
- [Nitty Gritty Gameboy Cycle Timing][nitty]: Down and dirty timing of the Game
  Boy's video hardware.

### Hardware

- [Emu-Russia's DMG-01 SM83 Core Research][dmgcpu]: Verilog model with
  invaluable accompanying diagrams of the SM83 core.

### Software

- [Test ROMs by Shay Green (a.k.a. Blargg)][blargg]: Classic introductory
  collection of test ROMs.
- [Mooneye Test Suite][mooneye]: Comprehensive test suite for individual niche
  behaviours.
- [dmg-acid2 by Matt Curie][dmg-acid2]: Clever re-imagining of the Web Standards
  Project's Acid2 rendering test for the Game Boy's PPU.

## License

This project is dual-licensed under both [MIT License][license.mit] and [Apache
License 2.0][license.ap2]. You have permission to use this code under the
conditions of either license pursuant to the rights granted by the chosen
license.

<!-- Reference-style badges -->
[deps.badge]: https://deps.rs/repo/github/kaplanz/rugby/status.svg
[deps.hyper]: https://deps.rs/repo/github/kaplanz/rugby

<!-- Reference-style files -->
[license.ap2]:  ./LICENSE-APACHE
[license.mit]:  ./LICENSE-MIT
[docs.testing]: ./docs/TESTING.md

<!-- Reference-style links -->
[blargg]:    https://github.com/retrio/gb-test-roms
[dmg-acid2]: https://github.com/mattcurrie/dmg-acid2
[dmgcpu]:    https://github.com/emu-russia/dmgcpu
[gbarch]:    https://www.copetti.org/writings/consoles/game-boy
[gbctr]:     https://gekkio.fi/files/gb-docs/gbctr.pdf
[gbdev]:     https://gbdev.io
[gbedg]:     https://hacktix.github.io/GBEDG/
[gekkio]:    https://gekkio.fi
[mooneye]:   https://github.com/Gekkio/mooneye-test-suite
[nitty]:     http://blog.kevtris.org/blogfiles/Nitty%20Gritty%20Gameboy%20VRAM%20Timing.txt
[pandocs]:   https://gbdev.io/pandocs/
