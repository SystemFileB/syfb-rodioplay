#!/bin/sh
cargo build --release

if [ $? -eq 0 ]; then
    mkdir -p build
    cp ./target/release/rodioplay ./build/rodioplay.x86_64
fi
