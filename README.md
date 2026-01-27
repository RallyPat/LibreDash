# LibreDash

Raspberry Pi bare metal Dashboard Software written in Rust, fully compatible with TunerStudio/MegaSquirt .dash and .gauge formats.

## Overview

LibreDash is a bare metal dashboard application for Raspberry Pi written in Rust that runs directly on the hardware without an operating system. It is designed to display TunerStudio/MegaSquirt dashboards exactly as they appear in TunerStudio and Shadow Dash, providing a dedicated, high-performance dashboard display for automotive ECU tuning and monitoring.

## Features

- **Pure Rust Implementation**: Safe, modern systems programming with Rust
- **TunerStudio Compatible**: Full support for TunerStudio .dash and gauge formats
- **INI Gauge Parser**: Reads gauge configurations from mainController.ini files
- **Authentic Rendering**: Gauges render exactly like TunerStudio/Shadow Dash
- **Multiple Gauge Styles**:
  - Horizontal bar gauges
  - Vertical bar gauges  
  - Circular needle gauges (analog)
  - Digital numeric displays
- **Color-Coded Thresholds**: Automatic green/yellow/red coloring based on warning/danger zones
- **Bare Metal Performance**: Runs directly on Raspberry Pi hardware (no OS overhead)
- **No Standard Library**: Uses `#![no_std]` for minimal footprint
- **Framebuffer Graphics**: Hardware-accelerated display output
- **Real-time Updates**: Smooth animation and value updates

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

## TunerStudio Compatibility

LibreDash is designed to be a drop-in replacement for TunerStudio/Shadow Dash displays.

### Supported Gauge Configuration Format

LibreDash reads gauge definitions from TunerStudio INI files (`mainController.ini`):

```ini
[GaugeConfigurations]
tachometer = rpm, "Engine Speed", "RPM", 0, 8000, 300, 600, 7000, 7500, 0, 0
```

**Format**: `name = var, "title", "units", lo, hi, loD, loW, hiW, hiD, vd, ld`

- **name**: Internal gauge identifier
- **var**: Variable from ECU OutputChannels
- **title**: Display title on gauge
- **units**: Units label (e.g., "RPM", "PSI", "°F")
- **lo/hi**: Scale minimum/maximum
- **loD/loW**: Low danger/warning thresholds
- **hiW/hiD**: High warning/danger thresholds
- **vd**: Value decimal places
- **ld**: Label decimal places

### Gauge Styles

LibreDash supports all standard TunerStudio gauge styles:

1. **Horizontal Bar**: Classic left-to-right fill gauge
2. **Vertical Bar**: Bottom-to-top fill gauge
3. **Circular/Analog**: Needle gauge with arc scale
4. **Digital**: Large numeric display with color-coded border

### Color Coding

Gauges automatically change color based on thresholds:
- **Green**: Normal operating range
- **Yellow**: Warning zone (approaching limits)
- **Red**: Danger zone (critical values)

### .dash File Format

### .dash File Format

The TunerStudio .dash file is a binary format that includes:
- Dashboard layout coordinates
- Gauge positions and sizes
- Gauge style selections
- Embedded fonts and images
- Background graphics

**Status**: LibreDash currently parses INI gauge configurations. Full binary .dash parsing is in development. See `example.ini` for the supported gauge configuration format.

## Example Gauge Configuration

See `example.ini` for a complete sample INI file with various gauge types:

```ini
[GaugeConfigurations]
; Engine gauges
tachometer = rpm, "Engine Speed", "RPM", 0, 8000, 300, 600, 7000, 7500, 0, 0
oil_pressure = oilPressure, "Oil Pressure", "PSI", 0, 100, 10, 20, 80, 90, 1, 0
coolant_temp = coolantTemp, "Coolant Temp", "°F", 0, 250, 50, 80, 220, 240, 0, 0

; Speed and performance
speed = vehicleSpeed, "Speed", "MPH", 0, 200, 0, 0, 180, 190, 0, 0
boost = boostPressure, "Boost", "PSI", -15, 30, -5, 0, 25, 28, 1, 0

; Fuel system
afr = airFuelRatio, "Air/Fuel Ratio", "AFR", 10, 20, 10.5, 12, 16, 18, 2, 1
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
