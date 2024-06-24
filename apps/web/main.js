import { Cartridge, GameBoy } from "./pkg/rugby.js";

const msg = document.getElementById("msg");
const ctx = document.getElementById("lcd").getContext("2d");

// Construct an emulator instance
const emu = new GameBoy();
console.log("constructed a new console");

// Insert the game cartridge
emu.insert(new Cartridge());
console.log("inserted a game cartridge");

let cycle,
    frame = 0;
for (cycle = 0; frame < 60; cycle++) {
    emu.cycle();

    if (emu.vsync()) {
        // Extract frame data from the emulator
        const fdata = emu.frame();
        // Create an image to draw to the canvas
        const image = ctx.createImageData(160, 144);
        // Draw extracted frame data into the canvas
        for (let idx = 0; idx < 160 * 144; idx++) {
            image.data[4 * idx + 0] = 0;
            image.data[4 * idx + 1] = 0;
            image.data[4 * idx + 2] = 0;
            image.data[4 * idx + 3] = 32 * fdata[idx];
        }
        // Display the image in the canvas
        ctx.putImageData(image, 0, 0);
        // Increment rendered frames
        frame++;
    }
}

msg.innerHTML = `drew ${frame} frames in ${cycle} cycles`;
