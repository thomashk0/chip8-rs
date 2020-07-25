async function init() {
    const keymap = {
        "0": 0,
        "1": 1,
        "2": 2,
        "3": 3,
        "4": 4,
        "5": 5,
        "6": 6,
        "7": 7,
        "8": 8,
        "9": 9,
        "a": 0xA,
        "b": 0xB,
        "c": 0xC,
        "d": 0xD,
        "e": 0xE,
        "f": 0xF
    }
    const {instance} = await WebAssembly.instantiateStreaming(fetch("./chip8_wasm.wasm"));
    const exports = instance.exports;

    exports.chip8_init();

    const canvas_w = exports.chip8_fb_width();
    const canvas_h = exports.chip8_fb_height();
    const canvas = document.getElementById("demo-canvas");
    canvas.width = canvas_w;
    canvas.height = canvas_h;

    const framebuffer = exports.chip8_fb();
    const image = new ImageData(
        new Uint8ClampedArray(
            exports.memory.buffer,
            framebuffer,
            4 * canvas_w * canvas_h,
        ),
        canvas_w,
    );

    document.addEventListener("keydown", event => {
        const key = event.key;
        console.log(`Key down: ${key}`);
        if (key in keymap) {
            exports.chip8_key_down(keymap[key]);
        }
    });

    document.addEventListener("keyup", event => {
        const key = event.key;
        console.log(`Key up: ${key}`);
        if (key in keymap) {
            exports.chip8_key_up(keymap[key]);
        }
    });
    const runButton = document.getElementById("reset");
    runButton.addEventListener("click", () => {
        exports.chip8_reset();
    });

    const ctx = canvas.getContext("2d");
    let start;
    const render = (timestamp) => {
        if (start === undefined)
            start = timestamp;
        const elapsed = timestamp - start;
        start = timestamp;
        exports.chip8_advance_ms(elapsed);
        ctx.putImageData(image, 0, 0);
        requestAnimationFrame(render);
    };
    render();
}

init();