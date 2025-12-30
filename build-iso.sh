#!/bin/bash
set -e

echo "=========================================="
echo "  Rustrial OS - Limine ISO Builder"
echo "=========================================="

# Check for required tools
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo not found"; exit 1; }
command -v xorriso >/dev/null 2>&1 || { echo "Error: xorriso not found. Install with: sudo apt install xorriso"; exit 1; }

# Configuration
KERNEL_NAME="rustrial_os"
TARGET="x86_64-rustrial_os.json"
ISO_NAME="rustrial_os.iso"
BUILD_DIR="target/iso_build"
LIMINE_VERSION="v8.x-binary"

echo ""
echo "Step 1/6: Building kernel with Limine support..."
cargo build --target $TARGET --release --no-default-features --features limine -Z unstable-options --config .cargo/config-limine.toml

echo ""
echo "Step 2/6: Setting up build directory..."
rm -rf $BUILD_DIR
mkdir -p $BUILD_DIR/boot $BUILD_DIR/EFI/BOOT

echo ""
echo "Step 3/6: Downloading Limine bootloader..."
if [ ! -d "limine" ]; then
    echo "Cloning Limine repository..."
    git clone https://github.com/limine-bootloader/limine.git --branch=$LIMINE_VERSION --depth=1
    cd limine
    make
    cd ..
else
    echo "Limine directory exists, skipping clone..."
fi

echo ""
echo "Step 4/6: Copying kernel and bootloader files..."
cp target/x86_64-rustrial_os/release/$KERNEL_NAME $BUILD_DIR/boot/kernel.elf
# limine.conf and limine-bios.sys must be in the same directory
cp limine.conf $BUILD_DIR/limine.conf
cp limine/limine-bios.sys $BUILD_DIR/limine-bios.sys  # Same dir as limine.conf
cp limine/limine-bios-cd.bin $BUILD_DIR/boot/
cp limine/limine-uefi-cd.bin $BUILD_DIR/boot/
cp limine/BOOTX64.EFI $BUILD_DIR/EFI/BOOT/
cp limine/BOOTIA32.EFI $BUILD_DIR/EFI/BOOT/

echo ""
echo "Step 5/6: Creating ISO image..."
xorriso -as mkisofs \
    -b boot/limine-bios-cd.bin \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    --efi-boot boot/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image \
    --protective-msdos-label \
    $BUILD_DIR -o $ISO_NAME

echo ""
echo "Step 6/6: Installing Limine bootloader to ISO..."
./limine/limine bios-install $ISO_NAME

echo ""
echo "=========================================="
echo "  ✓ ISO created successfully!"
echo "=========================================="
echo ""
echo "ISO location: $ISO_NAME"
echo "ISO size: $(du -h $ISO_NAME | cut -f1)"
echo ""
echo "To test with QEMU:"
echo "  qemu-system-x86_64 -cdrom $ISO_NAME -m 256M -serial stdio \\"
echo "    -netdev user,id=net0 -device rtl8139,netdev=net0"
echo ""
echo "To boot on real hardware:"
echo "  - Burn ISO to USB: sudo dd if=$ISO_NAME of=/dev/sdX bs=4M status=progress"
echo "  - Or burn to CD/DVD using your preferred tool"
echo ""
