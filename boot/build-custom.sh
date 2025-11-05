#!/bin/bash
# Complete custom bootloader build script for RustrialOS

set -e  # Exit on any error

echo -e "\e[36m╔════════════════════════════════════════════════╗\e[0m"
echo -e "\e[36m║  Building RustrialOS with Custom Bootloader  ║\e[0m"
echo -e "\e[36m╚════════════════════════════════════════════════╝\e[0m"

# Step 1: Build bootloader
echo -e "\n\e[33m[1/4] Building custom bootloader...\e[0m"
cd boot && make && cd .. || {
    echo -e "\e[31m✗ Bootloader build failed!\e[0m"
    exit 1
}
echo -e "\e[32m✓ Bootloader built successfully\e[0m"

# Step 2: Build kernel
echo -e "\n\e[33m[2/4] Building kernel (release mode)...\e[0m"
cargo build --target x86_64-rustrial_os.json --release || {
    echo -e "\e[31m✗ Kernel build failed!\e[0m"
    exit 1
}
echo -e "\e[32m✓ Kernel built successfully\e[0m"

# Step 3: Convert kernel to binary
echo -e "\n\e[33m[3/4] Converting kernel to flat binary...\e[0m"
rust-objcopy target/x86_64-rustrial_os/release/rustrial_os -O binary kernel.bin || {
    echo -e "\e[31m✗ Kernel conversion failed!\e[0m"
    echo -e "\e[33m   Make sure cargo-binutils is installed: cargo install cargo-binutils\e[0m"
    exit 1
}
echo -e "\e[32m✓ Kernel converted to flat binary\e[0m"

# Step 4: Create disk image
echo -e "\n\e[33m[4/4] Creating bootable disk image...\e[0m"

# Create 10MB disk image
dd if=/dev/zero of=disk.img bs=512 count=20000 2>/dev/null

# Write bootloader at sector 0
dd if=boot/bootloader.img of=disk.img conv=notrunc 2>/dev/null

# Write kernel at sector 66
dd if=kernel.bin of=disk.img bs=512 seek=65 conv=notrunc 2>/dev/null

# Get sizes for display
DISK_SIZE=$(du -h disk.img | cut -f1)
KERNEL_SIZE=$(du -h kernel.bin | cut -f1)

echo -e "\e[32m✓ Disk image created\e[0m"
echo -e "\e[90m   - Total size: $DISK_SIZE\e[0m"
echo -e "\e[90m   - Kernel size: $KERNEL_SIZE\e[0m"

echo -e "\n\e[32m╔════════════════════════════════════════════════╗\e[0m"
echo -e "\e[32m║              Build Complete!                  ║\e[0m"
echo -e "\e[32m╚════════════════════════════════════════════════╝\e[0m"

echo -e "\n\e[36mRun with:\e[0m"
echo -e "\e[97m  qemu-system-x86_64 -drive format=raw,file=disk.img\e[0m"
echo -e "\n\e[36mOr with serial output:\e[0m"
echo -e "\e[97m  qemu-system-x86_64 -drive format=raw,file=disk.img -serial stdio\e[0m"
