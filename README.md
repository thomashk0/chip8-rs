# A Minimal Chip8 Emulator Written in Rust

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.txt)

Features:

* Implements Chip8 ISA
* `#[no_std]` implementation of the emulator in `packages/chip8`
* Simple OpenGL + SDL2 GUI for running the emulator in `app/chip8-emu`
* WebAssembly version of the emulator in `app/chip8-wasm`, for running chip8 in the browser :)

## TODOs

* C bindings + Qt Gui (as a replacement for the chip8-emu)?
* WASM:
    * Configurable keybindings
    * Show CPU state
    * A bit of retro styling?

## License

This project is under a [MIT license](./LICENSE.txt).