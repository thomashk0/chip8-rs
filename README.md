# A Minimal Chip8 Emulator Written in Rust

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.txt)

See the [Online demo of the emulator](https://thomashk0.github.io/chip8_rs_demo.html).

## Overview

Project features:

* Emulates the CHIP8 only (not the Super/Mega variants)
* `#[no_std]` and lightweight implementation of the emulator is provided in the Rust crate `packages/chip8`. The crate is designed to be easily cross-compiled on very constrained platforms.
* WebAssembly version of the emulator in `app/chip8-wasm`, for running chip8 in a Web browser. The WASM version of the chip8 interpreter fits in less than 4K bytes!
* Simple OpenGL + SDL2 GUI for running the emulator in `app/chip8-emu`

## TODOs

A random list of possible future improvements of the project:

* Build instruction
* API clean-up and documentation
* Support CHIP8 variants
* (Not sure) C bindings + Qt Gui (as a replacement for chip8-emu)?
* WASM:
    * Configurable keybindings
    * Show CPU state
    * A bit of retro styling?

## License

This project is under a [MIT license](./LICENSE.txt).
