#!/bin/bash
#
# LibreDash SSH Deployment Script for Linux
#
# Builds kernel locally and deploys to a running Raspberry Pi via SSH.
#
# Usage:
#   ./deploy-via-ssh.sh pi@raspberrypi.local
#   ./deploy-via-ssh.sh pi@192.168.1.100 --no-reboot
#   ./deploy-via-ssh.sh pi@raspberrypi.local --verbose
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KERNEL_IMAGE="${SCRIPT_DIR}/kernel8.img"
LOG_FILE="${HOME}/.libredash/deploy-log.txt"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

# Parse arguments
TARGET="$1"
NO_REBOOT=false
VERBOSE=false

if [[ -z "$TARGET" ]]; then
    echo "Usage: $0 <user@host> [--no-reboot] [--verbose]"
    echo "Example: $0 pi@raspberrypi.local"
    exit 1
fi

shift
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-reboot)
            NO_REBOOT=true
            shift
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
status() {
    echo -e "${CYAN}[*]${NC} $1"
    if [[ -f "$LOG_FILE" ]]; then
        echo "$(date '+%Y-%m-%d %H:%M:%S') : $1" >> "$LOG_FILE"
    fi
}

success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

error() {
    echo -e "${RED}[✗] ERROR: $1${NC}" >&2
    exit 1
}

# Validation
if [[ ! -f "$KERNEL_IMAGE" ]]; then
    error "kernel8.img not found at $KERNEL_IMAGE. Run build first."
fi

echo ""
echo -e "${CYAN}=== LibreDash SSH Deployment ===${NC}"

# Verify SSH connectivity
status "Checking SSH connectivity to $TARGET..."
if ! ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=accept-new "$TARGET" "echo 'SSH OK'" > /dev/null 2>&1; then
    error "Cannot connect to $TARGET. Verify SSH is enabled and network is up."
fi
success "SSH connected"

# Build kernel
status "Building LibreDash for hardware..."
cd "$SCRIPT_DIR"

if ! cargo +nightly build --release --features hardware 2>&1 | grep -E "error|Finished"; then
    error "Cargo build failed"
fi

if ! cargo objcopy --release -- -O binary kernel8.img; then
    error "Objcopy failed"
fi

success "Build complete"

# Copy kernel
status "Copying kernel8.img to $TARGET:/tmp/..."
if ! scp -q "$KERNEL_IMAGE" "${TARGET}:/tmp/kernel8.img"; then
    error "SCP transfer failed"
fi
success "Kernel transferred"

# Install kernel on target
status "Installing kernel on target Pi..."
install_script=$(cat <<'EOF'
echo 'Installing kernel...'
sudo cp /tmp/kernel8.img /boot/kernel8.img.bak 2>/dev/null || true
sudo cp /tmp/kernel8.img /boot/kernel8.img
echo 'Kernel installed successfully'
EOF
)

if ! ssh "$TARGET" "$install_script"; then
    error "Installation failed"
fi
success "Kernel installed"

# Optional reboot
if [[ "$NO_REBOOT" != true ]]; then
    status "Rebooting target Pi..."
    ssh "$TARGET" "sudo shutdown -r now" > /dev/null 2>&1
    
    echo -e "\nWaiting for Pi to come back online..." -NoNewline
    
    target_host=$(echo "$TARGET" | cut -d'@' -f2)
    max_wait=60
    waited=0
    online=false
    
    while [[ $waited -lt $max_wait ]]; do
        sleep 3
        waited=$((waited + 3))
        
        if ping -c 1 -W 1 "$target_host" > /dev/null 2>&1; then
            success "Pi is back online"
            online=true
            break
        fi
        
        echo -n "."
    done
    
    if [[ "$online" != true ]]; then
        echo ""
        echo -e "${CYAN}Warning: Pi did not come back online within $max_wait seconds. Check manually.${NC}"
    fi
else
    echo ""
    echo -e "${CYAN}Kernel installed. Run 'sudo shutdown -r now' on the Pi to reboot with new kernel.${NC}"
fi

echo ""
echo -e "${GREEN}=== SSH Deployment Complete ===${NC}"
echo ""
