# RustrialOS Custom Bootloader Build Script (PowerShell)
# Requires: nasm.exe in PATH or in current directory

Write-Host "RustrialOS Custom Bootloader Build Script" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# Check if nasm is available
$nasm = Get-Command nasm -ErrorAction SilentlyContinue
if (-not $nasm) {
    Write-Host "ERROR: nasm.exe not found in PATH!" -ForegroundColor Red
    Write-Host "Please install NASM from: https://www.nasm.us/" -ForegroundColor Yellow
    Write-Host "Or place nasm.exe in this directory" -ForegroundColor Yellow
    exit 1
}

Write-Host "Found NASM: $($nasm.Source)" -ForegroundColor Green
Write-Host ""

# Clean previous builds
Write-Host "Cleaning previous build..." -ForegroundColor Yellow
Remove-Item -Path "boot.bin" -ErrorAction SilentlyContinue
Remove-Item -Path "stage2.bin" -ErrorAction SilentlyContinue
Remove-Item -Path "bootloader.img" -ErrorAction SilentlyContinue

# Assemble Stage 1 (boot sector)
Write-Host "Assembling Stage 1 (boot.asm)..." -ForegroundColor Yellow
& nasm -f bin boot.asm -o boot.bin

if (-not $?) {
    Write-Host "ERROR: Failed to assemble Stage 1!" -ForegroundColor Red
    exit 1
}

$stage1Size = (Get-Item "boot.bin").Length
Write-Host "Stage 1 assembled: boot.bin ($stage1Size bytes)" -ForegroundColor Green

if ($stage1Size -ne 512) {
    Write-Host "WARNING: Stage 1 should be exactly 512 bytes!" -ForegroundColor Red
    Write-Host "Current size: $stage1Size bytes" -ForegroundColor Red
}

Write-Host ""

# Assemble Stage 2
Write-Host "Assembling Stage 2 (stage2.asm)..." -ForegroundColor Yellow
& nasm -f bin stage2.asm -o stage2.bin

if (-not $?) {
    Write-Host "ERROR: Failed to assemble Stage 2!" -ForegroundColor Red
    exit 1
}

$stage2Size = (Get-Item "stage2.bin").Length
Write-Host "Stage 2 assembled: stage2.bin ($stage2Size bytes)" -ForegroundColor Green
Write-Host ""

# Combine Stage 1 and Stage 2
Write-Host "Creating bootloader image..." -ForegroundColor Yellow

# Read both files and combine
$stage1 = [System.IO.File]::ReadAllBytes("boot.bin")
$stage2 = [System.IO.File]::ReadAllBytes("stage2.bin")
$combined = $stage1 + $stage2
[System.IO.File]::WriteAllBytes("bootloader.img", $combined)

$totalSize = (Get-Item "bootloader.img").Length
Write-Host "Bootloader image created: bootloader.img ($totalSize bytes)" -ForegroundColor Green
Write-Host ""

Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Files created:" -ForegroundColor Cyan
Write-Host "  boot.bin        - Stage 1 boot sector (512 bytes)" -ForegroundColor White
Write-Host "  stage2.bin      - Stage 2 bootloader (~8KB)" -ForegroundColor White
Write-Host "  bootloader.img  - Combined bootloader image" -ForegroundColor White
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Build your Rust kernel: cargo build" -ForegroundColor White
Write-Host "  2. Create disk image with bootloader + kernel" -ForegroundColor White
Write-Host "  3. Test in QEMU: qemu-system-x86_64 -drive format=raw,file=disk.img" -ForegroundColor White
