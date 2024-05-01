# Architecture

This document describes several aspects of the Game Boy's architecture,
highlighting key areas of focus used within accurate emulator development.

![][dmg.pcb][^rev6]

## Notation

Within this document, addresses are always represented in hexadecimal, such as
`FEA0`, with the hexadecimal prefix `0x` implied.

## Schematic

At a high level, the motherboard contains the following ASICs:

- Unit 1: Sharp LR35902: Main SoC, a custom silicon package containing (among
  other things):
  - Boot: 256 bytes of non-programmable boot ROM.
  - HRAM: 127 bytes of CPU-local embedded static RAM.
  - APU: Audio Processing Unit. Used to generate sound effects/music.
  - PPU: Picture Processing Unit. Used to drive the LCD display.
    - DMA: Direct memory access circuit. Used to quickly write to OMA.
    - OAM: Object attribute memory. Used to store sprite data.
  - SM83: Sharp CPU resembling a Z80/Intel 8080 hybrid.
    - PIC: Programmable interrupt controller.
- Unit 2: Sharp LH5164LN: Video RAM (8 KiB) for graphical rendering by PPU.
- Unit 3: Sharp LH5164LN: Work RAM (8 KiB) for general purposes.

```
           Main SoC                 VRAM
    ┌────────────────────┐       ┌────────┐   ┌──┐C
    │U1: DMG CPU  LR35902│       │U2: SRAM│   ├──┤a
    │ ┌──────┐  ┌──────┐ │ V-Bus │        │   ├──┤r
    │ │ APU  ◄──► PPU  │ │◄─────►│   64K  │   ├──┤t
    │ └▲─────┘  └▲─────┘ │       │(8K ✗ 8)│ ┌►├──┤r
    │  ├─────────┘ I-Bus │       │        │ │ ├──┤i
    │  │  ┌────────┐     │       │LH5164LN│ │ ├──┤d
    │  ├──►  SM83  │     │ E-Bus └────────┘ │ ├──┤g
    │  │  └────────┘     │◄──┬──────────────┘ └──┘e
    │  ├─────────┐       │   │   ┌──────────────┐
    │ ┌▼─────┐  ┌▼─────┐ │   │   │U3: SRAM      │
    │ │ Boot │  │ HRAM │ │   └──►│ 64K (8K ✗ 8) │
    │ └──────┘  └──────┘ │       │      LH5164LN│
    └────────────────────┘       └──────────────┘
                                       WRAM
```
[^gram]

> [!NOTE]
> See the [memory architecture](./ARCH.md) for a more precise diagram.

> [!NOTE]
> More research is needed to determine exactly how components are laid out
> within the LR36902. For example, the DMA and OAM may not be embedded within
> the PPU.

### Memory Map

The Game Boy's CPU uses the following 16-bit address decoding scheme:

|    Address    |  Size  | Name | Description      |   Chip   |    Bus    | Notes
| ------------- | ------:| ----:|:---------------- | -------- | --------- | -----
| `0000..=00FF` |  256 B | Boot | Boot ROM         | LR35902  | Internal  | [^boot]
| `0000..=7FFF` | 32 KiB | Cart | Cartridge ROM    | Variable | External  | [^cart]
| `8000..=9FFF` |  8 KiB | VRAM | Video RAM        | LH5164LN | Video     |
| `A000..=BFFF` |  8 KiB | ERAM | External RAM     | Variable | External  | [^cart]
| `C000..=DFFF` |  8 KiB | WRAM | Work RAM         | LH5164LN | External  |
| `E000..=FDFF` | 7680 B | Echo | Echo RAM         | LH5164LN | External  | [^nuse][^eram]
| `FE00..=FE9F` |  160 B |  OAM | Object memory    | LR35902  | Internal  |
| `FEA0..=FEFF` |   96 B |      | Unmapped         |          | Internal  | [^nuse]
| `FF00..=FF7F` |  128 B |  Reg | I/O registers    | LR35902  | Internal  |
| `FF80..=FFFE` |  127 B | HRAM | High RAM         | LR35902  | Internal  |
| `FFFF..=FFFF` |    1 B |   IE | Interrupt enable | LR35902  | Internal  |

> [!WARNING]
> Performing a read to an unmapped region in memory will always
> return the `0xFF` byte. More generally, unmapped wires have a pull-up resistor
> that, when read, yields as logic-high.

### Memory Bus

Buses can be thought of as a tri-state buffer, allowing for shared (but single
driver) use to request or submit data to another device.

#### External Dispatch

Collectively, there are two memory buses external to the main SoC; these are the
external bus (E-Bus) and video bus (V-Bus). When a memory request is dispatched
from the main SoC, it will appear on precisely _one_ of the external buses,
depending on the address. While the V-Bus will always route to the VRAM, the
E-Bus will further decode the address to drive either the cartridge slot or
WRAM.

|    Address    |    Bus    | Destination |  Select  |
| ------------- | --------- | ----------- | -------- |
| `0000..=7FFF` | External  | Cartridge   | `0b0xxx` |
| `8000..=9FFF` | Video     | Video RAM   | `0b100x` |
| `A000..=BFFF` | External  | Cartridge   | `0b101x` |
| `C000..=FFFF` | External  | Work RAM    | `0b11xx` |

From the above, we can see how the chip select for the destination is calculated
using address bits A15-A13.

##### Echo RAM

For addresses of the form `0b11xx` (using bits A15-A14), WRAM is selected as the
destination device. However, as WRAM contains 8 KiB ($8192 = 2^{13}$ bytes),
only 13 addresses bits A12-A0 are needed. As such, address bit A13  is ignored
when dispatching to WRAM. This has the side effect of causing WRAM to appear
mapped twice, both at `C000..DFFF` and `E000..FFFF`. This second mapping is the
source of the Echo RAM. However, its use is discouraged by Nintendo due to the
upper addresses `FE00..FFFF` mapping to the internal bus.

#### Internal Dispatch

For addresses that map to internal devices, neither of the external memory buses
will show dispatch signals. Instead, these addresses map directly to the
corresponding device. (This can be thought of as a separate internal bus, or
I-Bus.) Furthermore, **internal bus requests are only visible to the CPU**. The
biggest implication of this is that if a DMA source address is selected with a
value ≥`FE00` it will not read from the I-Bus, rather it will appear on the
E-Bus (in this instance mapping to Echo RAM).

#### Conflicts

Multiple devices attempting concurrent access to the same bus is called a [bus
conflict][conflict]. In the Game Boy, the devices with bus access are:

- CPU: Addressable access to the entire [memory map](#memory-map).
- DMA: Priority access to OAM and both E-Bus and V-Bus.
- PPU: Direct access to OAM and V-Bus.

As a result, there are situations where multiple devices both attempt to use a
bus.

##### During DMA

The most common way to cause a bus conflict is with a DMA. While a DMA is
occurring, it has priority access to the OAM and the target address bus. In the
event that the CPU performs a memory access that would use the same bus as the
DMA's target, the CPU will always "lose" the access to the DMA [^dma]. The
result is that the CPU will read the same value that happens to be copied in the
corresponding DMA cycle. A consequence of this is that even CPU instruction
fetches can fail in this way, causing execution on the DMA's data.

As a workaround, Nintendo recommends copying a small subroutine to HRAM, which
the CPU always has exclusive access to (via the I-Bus), to execute until the DMA
is complete.

<!-- Footnotes -->
[^rev6]: "Game Boy (Rev 6) Motherboard" by [Joonas Javanainen][gekkio] is
         licensed under [CC BY 4.0][lic.cc4].
[^boot]: Boot ROM will become unmapped automatically upon completion, after
         which it is no longer accessible until a reset.
[^cart]: Cartridge slot memory traffic is handled by the Memory Bank Controller
         (MBC) used by included in the physical cartridge.
[^gram]: ASCII diagram is based upon the detailed [schematic][dmg.sch] by
         [Joonas Javanainen][gekkio] licensed under [CC BY 4.0][lic.cc4].
[^nuse]: Nintendo says use of this area is prohibited.
[^eram]: Echo RAM is simply a mirror of WRAM, accessible due to a quirk in how
         addresses are decoded. See [above](#echo-ram) for more details.
[^dma]:  See the discussion at Gekkio/mooneye-gb#39.

<!--
  Reference-style links
-->

[dmg.pcb]:  https://github.com/Gekkio/gb-schematics/blob/main/DMG-CPU-06/DMG-CPU-06.jpg

<!-- Schematic -->
[conflict]: https://en.wikipedia.org/wiki/Bus_contention

<!-- Footnotes -->
[dmg.sch]:  https://github.com/Gekkio/gb-schematics/blob/main/DMG-CPU-06/schematic/DMG-CPU-06.pdf
[gekkio]:   https://github.com/Gekkio/gb-schematics
[lic.cc4]:  http://creativecommons.org/licenses/by/4.0/
