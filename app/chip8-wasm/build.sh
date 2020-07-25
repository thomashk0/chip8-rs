#!/bin/bash

set -euo pipefail

TARGET=wasm32-unknown-unknown
BINARY=target/$TARGET/release/chip8_wasm.wasm

cargo build --target $TARGET --release
wasm-snip --snip-rust-fmt-code \
    --snip-rust-panicking-code \
    -o $BINARY \
    $BINARY
wasm-strip $BINARY

[ -d www ] || mkdir -p www
wasm-opt -o www/chip8_wasm.wasm -Oz $BINARY
wasm2wat -o www/chip8_wasm.wat www/chip8_wasm.wasm
wasm-objdump -d www/chip8_wasm.wasm > www/chip8_wasm.wasm.txt

echo "Size of chip8_wasm.wasm: $(stat -c %s www/chip8_wasm.wasm) bytes"
