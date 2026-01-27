# LibreDash

Raspberry Pi bare metal Dashboard Software with TS .dash format compatibility.

## Overview

LibreDash is a bare metal dashboard application for Raspberry Pi that runs directly on the hardware without an operating system. It provides a flexible framework for creating custom dashboards with various visual elements like gauges, graphs, and value displays.

## Features

- **Bare Metal Performance**: Runs directly on Raspberry Pi hardware (no OS overhead)
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

- ARM cross-compilation toolchain: `arm-none-eabi-gcc`
- GNU Make

On Ubuntu/Debian:
```bash
sudo apt-get install gcc-arm-none-eabi binutils-arm-none-eabi
```

On macOS:
```bash
brew install arm-none-eabi-gcc
```

### Build Instructions

```bash
make
```

This will generate `kernel8.img` which can be copied to your Raspberry Pi SD card.

To clean build artifacts:
```bash
make clean
```

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
- Raspberry Pi 4 Model B
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

## Architecture

- `src/boot.S` - ARM64 boot code and initialization
- `src/kernel.c` - Main kernel entry point and demo
- `src/framebuffer.c` - Framebuffer initialization and drawing primitives
- `src/dashboard.c` - Dashboard rendering and element management
- `include/` - Header files for all components
- `linker.ld` - Linker script for memory layout

## Development

The project uses a modular architecture:

1. **MMIO Layer**: Low-level hardware access (GPIO, mailbox, etc.)
2. **Framebuffer Layer**: Graphics primitives and display management
3. **Dashboard Layer**: High-level dashboard element rendering
4. **Application Layer**: Main kernel with dashboard setup

## License

This project is licensed under the GNU General Public License v2.0. See LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Acknowledgments

- Raspberry Pi documentation and community
- ARM architecture reference materials
- Bare metal programming resources
