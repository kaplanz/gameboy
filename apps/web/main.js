import { Emulator } from "./pkg/rugby.js";

const emu = new Emulator();

emu.reset();

for (let i = 0, len = 100; i < len; i++) {
    if (emu.ready()) {
        emu.cycle();
    }
}

console.log(emu.toString());
