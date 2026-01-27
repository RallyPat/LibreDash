#!/bin/bash
# Build script for LibreDash

set -e

echo "Building LibreDash..."

# Build the project
cargo build --release

# Convert ELF to binary image
rust-objcopy --strip-all -O binary target/aarch64-rpi/release/libredash kernel8.img

echo "Build complete! kernel8.img is ready."
echo "Copy kernel8.img to your Raspberry Pi SD card."
