#!/bin/bash
# Create bootable ISO image from RustrialOS
# Requires: grub-mkrescue, xorriso

set -e

ISO_NAME="rustrial-os.iso"
BOOTIMAGE="target/x86_64-rustrial_os/debug/bootimage-rustrial_os.bin"
ISO_DIR="target/iso"

echo "Creating bootable ISO: $ISO_NAME"

# Check dependencies
if ! command -v grub-mkrescue &> /dev/null; then
    echo "Error: grub-mkrescue not found. Install grub-pc-bin or grub-efi-amd64-bin"
    exit 1
fi

if ! command -v xorriso &> /dev/null; then
    echo "Error: xorriso not found. Install xorriso package"
    exit 1
fi

# Check if bootimage exists
if [ ! -f "$BOOTIMAGE" ]; then
    echo "Bootimage not found. Building..."
    cargo bootimage --target x86_64-rustrial_os.json
fi

# Create ISO directory structure
mkdir -p "$ISO_DIR/boot/grub"

# Copy kernel
cp "$BOOTIMAGE" "$ISO_DIR/boot/kernel.bin"

# Create GRUB config
cat > "$ISO_DIR/boot/grub/grub.cfg" << 'EOF'
set timeout=0
set default=0

menuentry "RustrialOS" {
    multiboot2 /boot/kernel.bin
    boot
}
EOF

# Create ISO
grub-mkrescue -o "$ISO_NAME" "$ISO_DIR"

# Clean up
rm -rf "$ISO_DIR"

SIZE=$(du -h "$ISO_NAME" | cut -f1)
echo "ISO created: $ISO_NAME ($SIZE)"
echo "Test with: qemu-system-x86_64 -cdrom $ISO_NAME"
