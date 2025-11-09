#!/usr/bin/env pwsh
# ============================================================================
# RustrialOS Custom Bootloader Build Script - WINDOWS (PowerShell)
# ============================================================================
# This script is designed for Windows systems using PowerShell.
# For Linux/macOS, use build-custom.sh instead.
# 
# Complete custom bootloader build script for RustrialOS

Write-Host "╔════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  Building RustrialOS with Custom Bootloader  ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════╝" -ForegroundColor Cyan

# Step 1: Build bootloader
Write-Host "`n[1/4] Building custom bootloader..." -ForegroundColor Yellow
Push-Location boot
.\build.ps1
if (-not $?) {
    Write-Host "✗ Bootloader build failed!" -ForegroundColor Red
    Pop-Location
    exit 1
}
Pop-Location
Write-Host "✓ Bootloader built successfully" -ForegroundColor Green

# Step 2: Build kernel
Write-Host "`n[2/4] Building kernel (release mode)..." -ForegroundColor Yellow
cargo build --target x86_64-rustrial_os.json --release
if (-not $?) {
    Write-Host "✗ Kernel build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Kernel built successfully" -ForegroundColor Green

# Step 3: Convert kernel to binary
Write-Host "`n[3/4] Converting kernel to flat binary..." -ForegroundColor Yellow
rust-objcopy target/x86_64-rustrial_os/release/rustrial_os -O binary kernel.bin
if (-not $?) {
    Write-Host "✗ Kernel conversion failed!" -ForegroundColor Red
    Write-Host "   Make sure cargo-binutils is installed: cargo install cargo-binutils" -ForegroundColor Yellow
    exit 1
}
Write-Host "✓ Kernel converted to flat binary" -ForegroundColor Green

# Step 4: Create disk image
Write-Host "`n[4/4] Creating bootable disk image..." -ForegroundColor Yellow

# Create 10MB disk image
$diskSize = 20000 * 512
$zeros = New-Object byte[] $diskSize
[System.IO.File]::WriteAllBytes("disk.img", $zeros)

# Write bootloader at sector 0
$bootloader = [System.IO.File]::ReadAllBytes("boot\bootloader.img")
$disk = [System.IO.File]::ReadAllBytes("disk.img")
[Array]::Copy($bootloader, 0, $disk, 0, $bootloader.Length)

# Write kernel at sector 66 (0x4200 bytes offset)
$kernel = [System.IO.File]::ReadAllBytes("kernel.bin")
[Array]::Copy($kernel, 0, $disk, 33280, $kernel.Length)

[System.IO.File]::WriteAllBytes("disk.img", $disk)

$diskSizeMB = [math]::Round($diskSize / 1MB, 2)
$kernelSizeKB = [math]::Round($kernel.Length / 1KB, 2)
Write-Host "✓ Disk image created" -ForegroundColor Green
Write-Host "   - Total size: $diskSizeMB MB" -ForegroundColor Gray
Write-Host "   - Kernel size: $kernelSizeKB KB" -ForegroundColor Gray

Write-Host "`n╔════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║              Build Complete!                  ║" -ForegroundColor Green
Write-Host "╚════════════════════════════════════════════════╝" -ForegroundColor Green

Write-Host "`nRun with:" -ForegroundColor Cyan
Write-Host "  qemu-system-x86_64 -drive format=raw,file=disk.img" -ForegroundColor White
Write-Host "`nOr with serial output:" -ForegroundColor Cyan
Write-Host "  qemu-system-x86_64 -drive format=raw,file=disk.img -serial stdio" -ForegroundColor White
