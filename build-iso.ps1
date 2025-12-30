@echo off
setlocal enabledelayedexpansion

echo ==========================================
echo   Rustrial OS - Limine ISO Builder
echo ==========================================

REM Check for required tools
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: cargo not found
    exit /b 1
)

REM Configuration
set KERNEL_NAME=rustrial_os
set TARGET=x86_64-rustrial_os.json
set ISO_NAME=rustrial_os.iso
set BUILD_DIR=target\iso_build
set LIMINE_VERSION=v8.0.17

echo.
echo Step 1/6: Building kernel with Limine support...
cargo build --target %TARGET% --release --no-default-features --features limine

echo.
echo Step 2/6: Setting up build directory...
if exist %BUILD_DIR% rmdir /s /q %BUILD_DIR%
mkdir %BUILD_DIR%\boot
mkdir %BUILD_DIR%\EFI\BOOT

echo.
echo Step 3/6: Downloading Limine bootloader...
if not exist "limine" (
    echo Cloning Limine repository...
    git clone https://github.com/limine-bootloader/limine.git --branch=%LIMINE_VERSION% --depth=1
    cd limine
    REM Note: Windows users will need to manually download prebuilt binaries
    echo Please download prebuilt Limine binaries from:
    echo https://github.com/limine-bootloader/limine/releases
    cd ..
) else (
    echo Limine directory exists, skipping clone...
)

echo.
echo Step 4/6: Copying kernel and bootloader files...
copy target\x86_64-rustrial_os\release\%KERNEL_NAME%.exe %BUILD_DIR%\boot\kernel.elf
copy limine.conf %BUILD_DIR%\boot\limine.conf
copy limine\limine-bios.sys %BUILD_DIR%\boot\
copy limine\limine-bios-cd.bin %BUILD_DIR%\boot\
copy limine\limine-uefi-cd.bin %BUILD_DIR%\boot\
copy limine\BOOTX64.EFI %BUILD_DIR%\EFI\BOOT\
copy limine\BOOTIA32.EFI %BUILD_DIR%\EFI\BOOT\

echo.
echo Step 5/6: Creating ISO image...
echo Note: You'll need xorriso or a similar tool for Windows
echo Consider using WSL or downloading mkisofs for Windows
echo.
echo Manual ISO creation steps:
echo 1. Use a tool like ImgBurn or mkisofs to create an ISO
echo 2. Include all files from %BUILD_DIR%
echo 3. Set boot sector to boot\limine-bios-cd.bin
echo.

echo.
echo ==========================================
echo   Build preparation complete!
echo ==========================================
echo.
echo Next steps:
echo 1. Install xorriso or use WSL to create the ISO
echo 2. Or manually create ISO using Windows tools
echo.
echo To test with QEMU on Windows:
echo   qemu-system-x86_64 -cdrom %ISO_NAME% -m 256M -serial stdio
echo.

pause
