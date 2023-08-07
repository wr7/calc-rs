#!/bin/sh
cargo build --release
cp ./target/thumbv6m-none-eabi/release/calc-rs ./calc-rs.elf
