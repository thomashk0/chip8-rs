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

out=www
[ -d ${out} ] || mkdir -p ${out}
wasm-opt -o ${out}/chip8_wasm.wasm -Oz $BINARY
wasm2wat -o ${out}/chip8_wasm.wat ${out}/chip8_wasm.wasm
wasm-objdump -d ${out}/chip8_wasm.wasm > ${out}/chip8_wasm.wasm.txt
echo "info: size of chip8_wasm.wasm: $(stat -c %s ${out}/chip8_wasm.wasm) bytes"

echo "info: regenerating ${out}/roms/index.json"
[ -d ${out}/roms ] || mkdir -p ${out}/roms
./scripts/make-index.py -o ${out}/roms/index.json ../../assets/roms
