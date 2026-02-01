<#
.SYNOPSIS
    Deploy LibreDash kernel to Raspberry Pi 3B+ SD card
.DESCRIPTION
    Builds LibreDash for real hardware, downloads official firmware,
    optionally backs up SD card, customizes config, and copies files.
.PARAMETER NoBackup
    Skip SD card backup
.PARAMETER Device
    Specify SD card drive letter (e.g., "E:"). Auto-detects if not specified.
.PARAMETER Verbose
    Show all commands and detailed output
.EXAMPLE
    .\deploy-to-pi.ps1
    .\deploy-to-pi.ps1 -NoBackup
    .\deploy-to-pi.ps1 -Device "E:" -Verbose
#>

param(
    [switch]$NoBackup = $false,
    [string]$Device = "",
    [switch]$Verbose = $false
)

$ErrorActionPreference = "Stop"
$WarningPreference = "Continue"

# Configuration
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$LibreDashHome = Join-Path $env:USERPROFILE ".libredash"
$FirmwareDir = Join-Path $LibreDashHome "rpi-firmware"
$BackupDir = Join-Path $LibreDashHome "backups"
$LogFile = Join-Path $LibreDashHome "deploy-log.txt"
$ConfigFile = Join-Path $LibreDashHome "deploy-config.json"
$BuildDir = Join-Path $ProjectRoot "target" "aarch64-unknown-linux-gnu" "release"
$KernelImage = Join-Path $ProjectRoot "kernel8.img"

# Firmware URLs
$FirmwareBaseUrl = "https://github.com/raspberrypi/firmware/raw/master/boot/"
$FirmwareFiles = @("bootcode.bin", "start.elf", "fixup.dat", "checksums.txt")

function Write-Log {
    param([string]$Message)
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    "$Timestamp : $Message" | Tee-Object -Append -FilePath $LogFile
}

function Write-Status {
    param([string]$Message)
    Write-Host "[*] $Message" -ForegroundColor Cyan
    Write-Log $Message
}

function Write-Success {
    param([string]$Message)
    Write-Host "[✓] $Message" -ForegroundColor Green
    Write-Log $Message
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "[✗] ERROR: $Message" -ForegroundColor Red
    Write-Log "ERROR: $Message"
    exit 1
}

# Initialize environment
Write-Host "`n=== LibreDash SD Card Deployment ===" -ForegroundColor Yellow
Write-Status "Checking environment..."

# Create directories
if (-not (Test-Path $LibreDashHome)) {
    New-Item -ItemType Directory -Path $LibreDashHome | Out-Null
    Write-Status "Created ~/.libredash directory"
}
if (-not (Test-Path $BackupDir)) {
    New-Item -ItemType Directory -Path $BackupDir | Out-Null
}

# Initialize log file
if (-not (Test-Path $LogFile)) {
    New-Item -ItemType File -Path $LogFile | Out-Null
}

Write-Log "=== Deployment started at $(Get-Date) ==="
Write-Success "Environment ready"

# Check Cargo
Write-Status "Checking Cargo installation..."
try {
    $CargoVersion = & cargo --version 2>&1
    Write-Success "Cargo found: $CargoVersion"
} catch {
    Write-Error-Custom "Cargo not found. Install Rust from https://rustup.rs/"
}

# Download and verify firmware
Write-Status "Checking Raspberry Pi firmware..."
$NeedDownload = $false

foreach ($file in $FirmwareFiles) {
    $FilePath = Join-Path $FirmwareDir $file
    if (-not (Test-Path $FilePath)) {
        $NeedDownload = $true
        break
    }
}

if ($NeedDownload) {
    Write-Status "Downloading Raspberry Pi firmware (first time)..."
    if (-not (Test-Path $FirmwareDir)) {
        New-Item -ItemType Directory -Path $FirmwareDir | Out-Null
    }

    # Download checksums first
    $ChecksumsUrl = $FirmwareBaseUrl + "checksums.txt"
    $ChecksumsPath = Join-Path $FirmwareDir "checksums.txt"
    Write-Status "  Downloading checksums.txt..."
    try {
        Invoke-WebRequest -Uri $ChecksumsUrl -OutFile $ChecksumsPath -ErrorAction Stop | Out-Null
        Write-Success "  checksums.txt downloaded"
    } catch {
        Write-Error-Custom "Failed to download checksums.txt: $_"
    }

    # Download and verify each firmware file
    foreach ($file in $FirmwareFiles | Where-Object {$_ -ne "checksums.txt"}) {
        $FileUrl = $FirmwareBaseUrl + $file
        $FilePath = Join-Path $FirmwareDir $file
        
        Write-Status "  Downloading $file..."
        try {
            Invoke-WebRequest -Uri $FileUrl -OutFile $FilePath -ErrorAction Stop | Out-Null
            Write-Success "    $file downloaded"
        } catch {
            Write-Error-Custom "Failed to download $file : $_"
        }

        # Verify checksum
        Write-Status "  Verifying SHA256 for $file..."
        $LocalHash = (Get-FileHash -Path $FilePath -Algorithm SHA256).Hash
        $ChecksumsContent = Get-Content $ChecksumsPath -Raw
        $ExpectedHash = $null
        
        foreach ($line in $ChecksumsContent -split "`n") {
            if ($line -match "^([a-f0-9]+)\s+\*?$file") {
                $ExpectedHash = $Matches[1]
                break
            }
        }

        if ($ExpectedHash -and $LocalHash -eq $ExpectedHash) {
            Write-Success "    Checksum verified: $file"
        } else {
            Write-Error-Custom "Checksum mismatch for $file"
        }
    }
} else {
    Write-Success "Firmware cache valid"
}

# Build kernel
Write-Status "Building LibreDash for hardware..."
Write-Status "  Running: cargo +nightly build --release --features hardware"
try {
    Push-Location $ProjectRoot
    & cargo +nightly build --release --features hardware 2>&1 | Where-Object {
        $_ -match "error|warning|Finished"
    } | ForEach-Object { Write-Host "    $_" }
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error-Custom "Cargo build failed"
    }
    
    Write-Status "  Creating kernel8.img..."
    & cargo objcopy --release -- -O binary kernel8.img
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error-Custom "Objcopy failed"
    }
    
    $KernelSize = (Get-Item $KernelImage).Length
    Write-Success "Build complete: kernel8.img ($KernelSize bytes)"
    Pop-Location
} catch {
    Write-Error-Custom "Build failed: $_"
}

# Detect SD card
Write-Status "Detecting SD cards..."
$Volumes = Get-Volume | Where-Object {
    $_.DriveType -eq "Removable" -and $_.FileSystem -eq "FAT32" -and $_.Size -gt 1GB
} | Select-Object -First 10

if (-not $Device) {
    if ($Volumes.Count -eq 0) {
        Write-Error-Custom "No removable FAT32 drives detected. Insert SD card and try again."
    }
    
    Write-Host "Detected SD cards:" -ForegroundColor Cyan
    for ($i = 0; $i -lt $Volumes.Count; $i++) {
        $Vol = $Volumes[$i]
        $DriveLetter = $Vol.DriveLetter
        $SizeGB = [math]::Round($Vol.Size / 1GB, 1)
        Write-Host "  $($i+1)) $DriveLetter : - $SizeGB GB - `"$($Vol.FileSystemLabel)`""
    }
    
    $Selection = Read-Host "Select SD card [1]"
    if ([string]::IsNullOrWhiteSpace($Selection)) { $Selection = "1" }
    
    if (-not ([int]::TryParse($Selection, [ref]$null)) -or $Selection -lt 1 -or $Selection -gt $Volumes.Count) {
        Write-Error-Custom "Invalid selection"
    }
    
    $SelectedVolume = $Volumes[$Selection - 1]
    $Device = $SelectedVolume.DriveLetter + ":"
} else {
    if (-not (Test-Path $Device)) {
        Write-Error-Custom "Device $Device not found"
    }
}

Write-Success "Selected device: $Device"

# Safety check
if ($Device -eq "C:" -or $Device -eq "D:") {
    Write-Error-Custom "Cannot deploy to system drive $Device"
}

# Create backup
if (-not $NoBackup) {
    $Timestamp = Get-Date -Format "yyyy-MM-dd-HH-mm"
    $BackupFile = Join-Path $BackupDir "sd-backup-$Timestamp.img"
    
    Write-Status "Creating backup of $Device (this may take 2-3 minutes)..."
    Write-Host "  Backup will be saved to: $BackupFile"
    
    try {
        # Note: Full SD card imaging requires admin. Using simpler file copy for now.
        $BackupSource = Get-ChildItem -Path $Device -Recurse -ErrorAction SilentlyContinue
        Write-Host "  Note: Using file-level backup (not raw disk image)"
        
        if (-not (Test-Path (Split-Path $BackupFile))) {
            New-Item -ItemType Directory -Path (Split-Path $BackupFile) | Out-Null
        }
        
        # Create a tar/zip backup instead
        $BackupZip = $BackupFile -replace '.img$', '.zip'
        Compress-Archive -Path "$Device\*" -DestinationPath $BackupZip -Force | Out-Null
        Write-Success "Backup saved: $BackupZip"
    } catch {
        Write-Host "Warning: Backup failed, but continuing deployment. Error: $_" -ForegroundColor Yellow
    }
}

# Customize config.txt
$ConfigPath = Join-Path $FirmwareDir "config.txt"
if (-not (Test-Path $ConfigPath)) {
    # Create default config.txt
    @"
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
"@ | Out-File -Encoding UTF8 $ConfigPath
}

$CustomizeConfig = Read-Host "Customize config.txt? [Y/n]"
if ($CustomizeConfig -ne "n" -and $CustomizeConfig -ne "N") {
    Write-Status "Opening config.txt in Notepad..."
    & notepad $ConfigPath
    Write-Status "Continuing deployment..."
}

# Copy files to SD card
Write-Status "Copying files to $Device ..."
try {
    foreach ($file in $FirmwareFiles | Where-Object {$_ -ne "checksums.txt"}) {
        $Source = Join-Path $FirmwareDir $file
        $Dest = Join-Path $Device $file
        Copy-Item -Path $Source -Destination $Dest -Force | Out-Null
        Write-Host "    - $file... ✓"
    }
    
    Copy-Item -Path $KernelImage -Destination (Join-Path $Device "kernel8.img") -Force | Out-Null
    Write-Host "    - kernel8.img... ✓"
    
    Copy-Item -Path $ConfigPath -Destination (Join-Path $Device "config.txt") -Force | Out-Null
    Write-Host "    - config.txt... ✓"
    
    Write-Success "All files copied"
} catch {
    Write-Error-Custom "Failed to copy files: $_"
}

# Safe eject
Write-Status "Ejecting $Device ..."
try {
    $Volume = Get-Volume | Where-Object {$_.DriveLetter -eq $Device[0]}
    if ($Volume) {
        $Volume | Dismount-Volume -Confirm:$false | Out-Null
    }
    Write-Success "Device ejected safely"
} catch {
    Write-Host "Warning: Could not eject device programmatically. Manually eject from system tray." -ForegroundColor Yellow
}

# Summary
Write-Host "`n=== Deployment Complete ===" -ForegroundColor Green
Write-Host "✓ Ready! Insert SD card into Raspberry Pi and power on." -ForegroundColor Green
Write-Host ""
Write-Host "Optional: Connect UART for debug output (115200 baud):" -ForegroundColor Cyan
Write-Host "  - GPIO 14 (TXD) → USB adapter RX" -ForegroundColor Gray
Write-Host "  - GPIO 15 (RXD) → USB adapter TX" -ForegroundColor Gray
Write-Host "  - GND → USB adapter GND" -ForegroundColor Gray
Write-Host ""
Write-Log "Deployment completed successfully"
Write-Host ""
