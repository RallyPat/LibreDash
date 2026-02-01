#!/bin/bash
#
# LibreDash SD Card Deployment Script for Linux
# 
# Usage:
#   ./deploy-to-pi.sh                    # Interactive SD card selection
#   ./deploy-to-pi.sh --device /dev/sdc  # Specific device
#   ./deploy-to-pi.sh --no-backup        # Skip backup
#   ./deploy-to-pi.sh --verbose          # Show detailed output
#

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIBREDASH_HOME="${HOME}/.libredash"
FIRMWARE_DIR="${LIBREDASH_HOME}/rpi-firmware"
BACKUP_DIR="${LIBREDASH_HOME}/backups"
LOG_FILE="${LIBREDASH_HOME}/deploy-log.txt"
CONFIG_FILE="${LIBREDASH_HOME}/deploy-config.json"
KERNEL_IMAGE="${SCRIPT_DIR}/kernel8.img"

# Firmware URLs
FIRMWARE_BASE_URL="https://github.com/raspberrypi/firmware/raw/master/boot/"
FIRMWARE_FILES=("bootcode.bin" "start.elf" "fixup.dat" "checksums.txt")

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Parse arguments
NO_BACKUP=false
DEVICE=""
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-backup)
            NO_BACKUP=true
            shift
            ;;
        --device)
            DEVICE="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Functions
log() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "$timestamp : $message" >> "$LOG_FILE"
}

status() {
    echo -e "${CYAN}[*]${NC} $1"
    log "$1"
}

success() {
    echo -e "${GREEN}[✓]${NC} $1"
    log "$1"
}

error() {
    echo -e "${RED}[✗] ERROR: $1${NC}" >&2
    log "ERROR: $1"
    exit 1
}

# Initialize environment
echo ""
echo -e "${YELLOW}=== LibreDash SD Card Deployment ===${NC}"
status "Checking environment..."

# Create directories
mkdir -p "$LIBREDASH_HOME" "$BACKUP_DIR" || error "Failed to create directories"

# Initialize log
log "=== Deployment started at $(date) ==="
success "Environment ready"

# Check required tools
for cmd in cargo lsblk dd; do
    if ! command -v $cmd &> /dev/null; then
        error "$cmd not found. Please install it."
    fi
done
success "All required tools found"

# Download and verify firmware
status "Checking Raspberry Pi firmware..."
if [[ ! -f "$FIRMWARE_DIR/bootcode.bin" ]]; then
    status "Downloading Raspberry Pi firmware (first time)..."
    mkdir -p "$FIRMWARE_DIR" || error "Failed to create firmware directory"

    # Download checksums first
    status "  Downloading checksums.txt..."
    if ! wget -q -O "$FIRMWARE_DIR/checksums.txt" "${FIRMWARE_BASE_URL}checksums.txt"; then
        error "Failed to download checksums.txt"
    fi
    success "  checksums.txt downloaded"

    # Download and verify each firmware file
    for file in "${FIRMWARE_FILES[@]}"; do
        if [[ "$file" == "checksums.txt" ]]; then
            continue
        fi

        status "  Downloading $file..."
        if ! wget -q -O "$FIRMWARE_DIR/$file" "${FIRMWARE_BASE_URL}$file"; then
            error "Failed to download $file"
        fi
        success "    $file downloaded"

        # Verify checksum
        status "  Verifying SHA256 for $file..."
        cd "$FIRMWARE_DIR"
        local_hash=$(sha256sum "$file" | awk '{print $1}')
        expected_hash=$(grep "$file" checksums.txt | awk '{print $1}')
        cd - > /dev/null

        if [[ "$local_hash" == "$expected_hash" ]]; then
            success "    Checksum verified: $file"
        else
            error "Checksum mismatch for $file"
        fi
    done
else
    success "Firmware cache valid"
fi

# Build kernel
status "Building LibreDash for hardware..."
status "  Running: cargo +nightly build --release --features hardware"
cd "$SCRIPT_DIR"
if ! cargo +nightly build --release --features hardware 2>&1 | grep -E "error|warning|Finished"; then
    error "Cargo build failed"
fi

status "  Creating kernel8.img..."
if ! cargo objcopy --release -- -O binary kernel8.img; then
    error "Objcopy failed"
fi

local kernel_size=$(stat -f%z "$KERNEL_IMAGE" 2>/dev/null || stat -c%s "$KERNEL_IMAGE" 2>/dev/null)
success "Build complete: kernel8.img ($kernel_size bytes)"

# Detect SD card
status "Detecting SD cards..."
if ! command -v lsblk &> /dev/null; then
    error "lsblk not found. Please install util-linux."
fi

# Find removable devices (typically /dev/sdc, /dev/sdd, etc.)
sd_devices=()
while IFS= read -r device; do
    sd_devices+=("$device")
done < <(lsblk -d -n -o NAME,TYPE,SIZE | grep disk | awk '{print "/dev/" $1}')

if [[ ${#sd_devices[@]} -eq 0 ]]; then
    error "No removable SD cards detected. Insert SD card and try again."
fi

if [[ -z "$DEVICE" ]]; then
    echo -e "${CYAN}Detected SD cards:${NC}"
    for i in "${!sd_devices[@]}"; do
        dev="${sd_devices[$i]}"
        size=$(lsblk -d -n -o SIZE "$dev" 2>/dev/null || echo "unknown")
        label=$(lsblk -n -o LABEL "${dev}1" 2>/dev/null | head -1)
        echo "  $((i+1))) $dev - $size - \"$label\""
    done

    read -p "Select SD card [1]: " selection
    selection=${selection:-1}

    if ! [[ "$selection" =~ ^[0-9]+$ ]] || [[ $selection -lt 1 ]] || [[ $selection -gt ${#sd_devices[@]} ]]; then
        error "Invalid selection"
    fi

    DEVICE="${sd_devices[$((selection-1))]}"
fi

# Verify device exists
if [[ ! -b "$DEVICE" ]]; then
    error "Device $DEVICE not found or is not a block device"
fi

# Safety check
if [[ "$DEVICE" == "/dev/sda" ]] || [[ "$DEVICE" == "/dev/nvme0n1" ]]; then
    error "Cannot deploy to system drive $DEVICE"
fi

success "Selected device: $DEVICE"

# Unmount device
status "Unmounting $DEVICE..."
for partition in ${DEVICE}*; do
    if mountpoint -q "$partition" 2>/dev/null; then
        sudo umount "$partition" || status "  Warning: Could not unmount $partition"
    fi
done

# Create backup
if [[ "$NO_BACKUP" != true ]]; then
    timestamp=$(date +%Y-%m-%d-%H-%M)
    backup_file="${BACKUP_DIR}/sd-backup-${timestamp}.img"

    status "Creating backup of $DEVICE (this may take 2-3 minutes)..."
    echo "  Backup will be saved to: $backup_file"

    # Get device size
    device_size=$(lsblk -b -d -n -o SIZE "$DEVICE")
    device_size_gb=$((device_size / 1024 / 1024 / 1024))

    if ! sudo dd if="$DEVICE" of="$backup_file" bs=4M status=progress; then
        status "Warning: Backup failed, but continuing deployment"
    else
        success "Backup saved: $backup_file (${device_size_gb}GB)"
    fi
fi

# Create config.txt if needed
config_path="$FIRMWARE_DIR/config.txt"
if [[ ! -f "$config_path" ]]; then
    cat > "$config_path" << 'EOF'
# LibreDash Configuration for Raspberry Pi 3B+
arm_64bit=1
kernel_address=0x80000
enable_uart=1

# HDMI Display Settings
hdmi_force_hotplug=1
hdmi_group=2
hdmi_mode=85

# Safe mode (use if display doesn't work)
# hdmi_safe=1

# UART Debug (GPIO 14/15)
# Connect USB-to-serial adapter for 115200 baud output
EOF
fi

# Customize config.txt
read -p "Customize config.txt? [Y/n]: " customize
if [[ "$customize" != "n" && "$customize" != "N" ]]; then
    status "Opening config.txt in editor..."
    ${EDITOR:-nano} "$config_path"
    status "Continuing deployment..."
fi

# Copy files to SD card
status "Copying files to $DEVICE..."

# Mount device if not mounted
mount_point="/tmp/libredash-sd-$$"
if ! mountpoint -q "$mount_point" 2>/dev/null; then
    mkdir -p "$mount_point"
    if ! sudo mount "${DEVICE}1" "$mount_point"; then
        error "Failed to mount $DEVICE"
    fi
fi

try_copy() {
    local src="$1"
    local dest="$2"
    local name=$(basename "$src")
    
    if ! sudo cp "$src" "$dest"; then
        error "Failed to copy $name"
    fi
    sudo chmod 644 "$dest" 2>/dev/null
    echo -e "    - $name... ${GREEN}✓${NC}"
}

# Copy firmware files
for file in "${FIRMWARE_FILES[@]}"; do
    if [[ "$file" != "checksums.txt" ]]; then
        try_copy "$FIRMWARE_DIR/$file" "$mount_point/$file"
    fi
done

# Copy kernel
try_copy "$KERNEL_IMAGE" "$mount_point/kernel8.img"

# Copy config
try_copy "$config_path" "$mount_point/config.txt"

success "All files copied"

# Sync and unmount
status "Syncing and unmounting..."
sudo sync
sudo umount "$mount_point" || error "Failed to unmount device"
rmdir "$mount_point"
success "Device ejected safely"

# Summary
echo ""
echo -e "${GREEN}=== Deployment Complete ===${NC}"
echo -e "${GREEN}✓ Ready! Insert SD card into Raspberry Pi and power on.${NC}"
echo ""
echo -e "${CYAN}Optional: Connect UART for debug output (115200 baud):${NC}"
echo -e "  - GPIO 14 (TXD) → USB adapter RX"
echo -e "  - GPIO 15 (RXD) → USB adapter TX"
echo -e "  - GND → USB adapter GND"
echo ""

log "Deployment completed successfully"
