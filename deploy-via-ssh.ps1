<#
.SYNOPSIS
    Deploy LibreDash kernel to running Raspberry Pi via SSH
.DESCRIPTION
    Builds kernel locally, copies via SSH to /boot on target Pi,
    and optionally reboots the device.
.PARAMETER Target
    SSH target in format user@host or user@ip (e.g., pi@raspberrypi.local)
.PARAMETER NoReboot
    Skip automatic reboot after deployment
.PARAMETER Verbose
    Show detailed output
.EXAMPLE
    .\deploy-via-ssh.ps1 -Target pi@raspberrypi.local
    .\deploy-via-ssh.ps1 -Target pi@192.168.1.100 -NoReboot
#>

param(
    [Parameter(Mandatory=$true)]
    [string]$Target,
    [switch]$NoReboot = $false,
    [switch]$Verbose = $false
)

$ErrorActionPreference = "Stop"

# Configuration
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$KernelImage = Join-Path $ProjectRoot "kernel8.img"
$LogFile = Join-Path $env:USERPROFILE ".libredash" "deploy-log.txt"

function Write-Status {
    param([string]$Message)
    Write-Host "[*] $Message" -ForegroundColor Cyan
    if (Test-Path $LogFile) {
        Add-Content -Path $LogFile -Value "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') : $Message"
    }
}

function Write-Success {
    param([string]$Message)
    Write-Host "[✓] $Message" -ForegroundColor Green
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "[✗] ERROR: $Message" -ForegroundColor Red
    exit 1
}

# Validation
if (-not (Test-Path $KernelImage)) {
    Write-Error-Custom "kernel8.img not found at $KernelImage. Run build first."
}

Write-Host "`n=== LibreDash SSH Deployment ===" -ForegroundColor Yellow

# Verify SSH connectivity
Write-Status "Checking SSH connectivity to $Target..."
try {
    ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=accept-new $Target "echo 'SSH OK'" | Out-Null
    Write-Success "SSH connected"
} catch {
    Write-Error-Custom "Cannot connect to $Target. Verify SSH is enabled and network is up."
}

# Build kernel
Write-Status "Building LibreDash for hardware..."
try {
    Push-Location $ProjectRoot
    & cargo +nightly build --release --features hardware 2>&1 | Where-Object {
        $_ -match "error|Finished"
    } | ForEach-Object { Write-Host "    $_" }
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error-Custom "Cargo build failed"
    }
    
    & cargo objcopy --release -- -O binary kernel8.img
    if ($LASTEXITCODE -ne 0) {
        Write-Error-Custom "Objcopy failed"
    }
    
    Pop-Location
    Write-Success "Build complete"
} catch {
    Write-Error-Custom "Build failed: $_"
}

# Copy kernel
Write-Status "Copying kernel8.img to $Target:/boot/..."
try {
    scp -q $KernelImage "${Target}:/tmp/kernel8.img"
    Write-Success "Kernel transferred"
} catch {
    Write-Error-Custom "SCP transfer failed: $_"
}

# Install kernel on target
Write-Status "Installing kernel on target Pi..."
try {
    $InstallScript = @"
echo 'Installing kernel...'
sudo cp /tmp/kernel8.img /boot/kernel8.img.bak 2>/dev/null
sudo cp /tmp/kernel8.img /boot/kernel8.img
echo 'Kernel installed successfully'
"@
    ssh $Target $InstallScript
    Write-Success "Kernel installed"
} catch {
    Write-Error-Custom "Installation failed: $_"
}

# Optional reboot
if (-not $NoReboot) {
    Write-Status "Rebooting target Pi..."
    ssh $Target "sudo shutdown -r now" | Out-Null
    
    Write-Host "`nWaiting for Pi to come back online..." -ForegroundColor Cyan
    $MaxWait = 60
    $Waited = 0
    $Online = $false
    
    while ($Waited -lt $MaxWait) {
        Start-Sleep -Seconds 3
        $Waited += 3
        
        try {
            if (Test-Connection -ComputerName ($Target.Split('@')[1]) -Count 1 -Quiet) {
                Write-Success "Pi is back online"
                $Online = $true
                break
            }
        } catch {
            # Still offline
        }
        
        Write-Host "." -NoNewline -ForegroundColor Gray
    }
    
    if (-not $Online) {
        Write-Host ""
        Write-Host "Warning: Pi did not come back online within $MaxWait seconds. Check manually." -ForegroundColor Yellow
    }
} else {
    Write-Host ""
    Write-Host "Kernel installed. Run 'sudo shutdown -r now' on the Pi to reboot with new kernel." -ForegroundColor Cyan
}

Write-Host ""
Write-Host "=== SSH Deployment Complete ===" -ForegroundColor Green
Write-Host ""
