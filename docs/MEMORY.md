# Memory

The following is a detailed diagram of Game Boy's memory architecture [^sono].

```mermaid
graph TD
    CPU(("#nbsp;#nbsp;#nbsp;CPU#nbsp;#nbsp;#nbsp;"))
    PPU(("#nbsp;#nbsp;#nbsp;PPU#nbsp;#nbsp;#nbsp;"))
    DMA((DMA))

    IBus[Internal Bus]
    EBus[External Bus]

    subgraph MOBO[Motherboard]
        CART
        EBus
        SOC
        SRAM
    end

    subgraph SRAM[SRAM Area]
        VRAM([VRAM])
        WRAM([WRAM])
    end

    subgraph SOC[SoC]
        subgraph CPUs[CPU-Bound Area]
            IO[I/O]
            HRAM([HRAM])
            BOOT[Boot ROM]

            subgraph HIGH[I/O Area]
                IO
                HRAM
            end

            BOOT
            CPU
            IBus

            CPU <==> IBus
            BOOT --x CPU
            IBus <--> HIGH
        end

        subgraph PPUs[PPU Internal]
            VRAMIF[VRAM Interface]

            subgraph OAM[OAM]
                OAMIF[OAM Interface]
                OAMA([OAM A])
                OAMB([OAM B])
            end

            DMA
            PPU
            VRAMIF

            OAMA & OAMB x--x OAMIF
            OAMIF  ==> PPU
            VRAMIF ==> PPU
        end

        DMA ==> OAMIF
    end

    subgraph CART[Cartridge Slot]
        ROM([ROM])
        ERAM([SRAM])
    end

    IBus <==>  EBus
    IBus <-->  OAMIF

    EBus x--x  CART
    EBus x--x  WRAM
    EBus <-->  VRAMIF
    EBus <-.-> DMA

    VRAM x--x  VRAMIF
```

<!-- Footnotes -->
[^sono]: Adapted from private communications with [Sono]. Used with permission.

<!-- Reference-style links -->
[sono]: https://github.com/SonoSooS
