import { acid2, Cartridge, GameBoy } from "./pkg/rugby.js";

// Construct an emulator instance
const emu = new GameBoy();
console.log("constructed a new console");

// Insert the game cartridge
emu.insert(new Cartridge(acid2()));
console.log("inserted a game cartridge");

// Construct the application
const app = {
    play: true,
};

// Construct application state
const gui = {
    cartridge: document.querySelector("input[name=rom]"),
    joypad: {
        dpad: {
            up: document.querySelector("#up"),
            lf: document.querySelector("#lf"),
            rt: document.querySelector("#rt"),
            dn: document.querySelector("#dn"),
        },
        game: {
            a: document.querySelector("#a"),
            b: document.querySelector("#b"),
        },
        keys: {
            start: document.querySelector("#start"),
            select: document.querySelector("#select"),
        },
    },
    screen: document.querySelector("#screen").getContext("2d"),
};

// Add event listeners
addEventListener("keydown", ({ key }) => emu.keydown(key));
addEventListener("keyup", ({ key }) => emu.keyup(key));

// Handle cartridge input
gui.cartridge.onchange = function () {
    let reader = new FileReader();
    reader.readAsArrayBuffer(this.files[0]);
    reader.onload = async function (event) {
        const data = new Uint8Array(event.target.result);
        const cart = new Cartridge(data);
        emu.insert(cart);
        emu.reset();
    };
};

// Perform asynchronous emulation
const FRQ = 4194304;
const DIV = 32;
setInterval(() => {
    for (let cycle = 0; cycle < FRQ / DIV; cycle++) {
        // Check if the console is paused
        if (!app.play) {
            break;
        }

        // Emulate a single cycle
        emu.cycle();

        // Redraw on vsync
        if (emu.vsync()) {
            // Extract frame data from the emulator
            const fdata = emu.frame();
            // Create an image to draw to the canvas
            const image = gui.screen.createImageData(160, 144);
            // Draw extracted frame data into the canvas
            for (let idx = 0; idx < 160 * 144; idx++) {
                image.data[4 * idx + 0] = 0;
                image.data[4 * idx + 1] = 0;
                image.data[4 * idx + 2] = 0;
                image.data[4 * idx + 3] = 32 * fdata[idx];
            }
            // Display the image in the canvas
            gui.screen.putImageData(image, 0, 0);
        }
    }
}, 1000 / DIV);
