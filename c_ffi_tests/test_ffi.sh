#!/bin/bash
set -e

echo "1- Building Rust staticlib"
cargo build --release

echo "2- Compiling C FFI test"
gcc -o ./out/test_ffi test_ffi.c \
    -Iinclude \
    ../target/release/librtp_midi_netsync.a \
    -lpthread -ldl -lm

echo "3- Running FFI test"
./out/test_ffi && echo "Test passed"
