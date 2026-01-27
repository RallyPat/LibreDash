# LibreDash

Raspberry Pi bare metal Dashboard Software written in Rust with TS .dash format compatibility.

## Overview

LibreDash is a bare metal dashboard application for Raspberry Pi written in Rust that runs directly on the hardware without an operating system. It provides a flexible framework for creating custom dashboards with various visual elements like gauges, graphs, and value displays.

## Features

- **Pure Rust Implementation**: Safe, modern systems programming with Rust
- **Bare Metal Performance**: Runs directly on Raspberry Pi hardware (no OS overhead)
- **No Standard Library**: Uses `#![no_std]` for minimal footprint
- **Framebuffer Graphics**: Hardware-accelerated display output
- **Multiple Dashboard Elements**:
  - Gauges (horizontal bars with percentage fill)
  - Value displays (with color-coded indicators)
  - Graphs (with grid lines for data visualization)
  - Labels (text display areas)
- **Real-time Updates**: Smooth animation and value updates
- **.dash Format Support**: Compatible with TS dashboard format (JSON-based)

## Building

### Prerequisites

- Rust toolchain (stable or nightly)
- `rust-src` component
- `llvm-tools-preview` component

Install Rust from [rustup.rs](https://rustup.rs), then install components:

```bash
rustup component add rust-src llvm-tools-preview
```

### Build Instructions

Use the provided build script:

```bash
./build.sh
```

Or build manually:

```bash
cargo build --release
rust-objcopy --strip-all -O binary target/aarch64-rpi/release/libredash kernel8.img
```

This will generate `kernel8.img` which can be copied to your Raspberry Pi SD card.

## Installation

1. Format an SD card as FAT32
2. Copy the following Raspberry Pi firmware files to the SD card:
   - `bootcode.bin`
   - `start.elf`
   - `fixup.dat`
   - `config.txt` (optional, for configuration)
3. Copy the compiled `kernel8.img` to the SD card
4. Insert the SD card into your Raspberry Pi and power it on

## Hardware Support

- Raspberry Pi 3 Model B/B+
- Raspberry Pi 4 Model B (may require config.txt adjustments)
- Display output via HDMI

## Dashboard Configuration

LibreDash supports the `.dash` format for dashboard configuration. See `example.dash` for a sample configuration file.

### .dash Format

The .dash format is a JSON-based configuration that defines dashboard elements:

```json
{
  "dashboard": {
    "name": "My Dashboard",
    "elements": [
      {
        "type": "gauge",
        "x": 50,
        "y": 50,
        "width": 400,
        "height": 60,
        "color": "#00FF00",
        "min_value": 0,
        "max_value": 100
      }
    ]
  }
}
```

## Project Structure

- `src/main.rs` - Main kernel entry point and demo
- `src/boot.rs` - ARM64 boot code and initialization
- `src/framebuffer.rs` - Framebuffer initialization and drawing primitives
- `src/dashboard.rs` - Dashboard rendering and element management
- `src/mmio.rs` - Memory-mapped I/O operations
- `linker.ld` - Linker script for memory layout
- `aarch64-rpi.json` - Custom target specification for Raspberry Pi
- `.cargo/config.toml` - Cargo build configuration

## Architecture

The project uses a modular Rust architecture:

1. **MMIO Layer** (`mmio.rs`): Low-level hardware access via memory-mapped I/O
2. **Framebuffer Layer** (`framebuffer.rs`): Graphics primitives and display management
3. **Dashboard Layer** (`dashboard.rs`): High-level dashboard element rendering
4. **Application Layer** (`main.rs`): Main kernel with dashboard setup and animation loop

All code runs in `#![no_std]` mode without the Rust standard library, using only `core` library features.

## Development

### Code Structure

The codebase follows Rust best practices for embedded/bare metal development:

- Uses `#![no_std]` and `#![no_main]` attributes
- Custom panic handler (infinite loop)
- Inline assembly for boot code
- Volatile memory operations for MMIO
- Type-safe abstractions over hardware

### Safety

While some unsafe code is necessary for hardware access, it's isolated to:
- MMIO operations (memory-mapped I/O)
- Framebuffer buffer access
- Boot assembly code

## License

This project is licensed under the GNU General Public License v2.0. See LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Acknowledgments

- Raspberry Pi documentation and community
- Rust embedded working group
- ARM architecture reference materials
- Bare metal programming resources
