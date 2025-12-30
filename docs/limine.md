# Limine Bootloader Integration

## Overview

Rustrial OS now supports **Limine**, a modern x86/x86_64 bootloader that enables creating **bootable ISO images** for real hardware and virtual machines. The project maintains dual bootloader support:

- **Bootloader crate** (default): For quick testing with `cargo run`
- **Limine** (framework ready): For production ISO builds that boot on real hardware

## Current Status

Experimental — Limine support is provided as an experimental feature. The integration currently compiles and produces an ISO, but it has not been fully validated on real hardware and is not expected to work reliably yet. You may observe boot-time logs in the serial terminal (example below), but the OS may not display correctly or run to completion.

The Limine integration infrastructure includes:
- Feature flags and conditional compilation
- Entry point and boot protocol structure
- Memory initialization framework
- Build scripts for ISO creation
- Limine protocol requests and full memory map integration are still under development and testing

Example serial boot logs you may see even while the feature is experimental:

```
Limine kernel starting...
Got boot info
HHDM offset: 0xffff800000000000
VGA test written at 0xffff8000000b8000
VGA buffer initialized
Direct VGA write after init
About to print to VGA...
VGA print successful!
About to call rustrial_os::init()...
init() complete
```

## Why Limine?

- ISO Creation: Build bootable CD/DVD/USB images
- Real Hardware: Boot on actual computers, not just QEMU
- UEFI + BIOS: Support for both modern and legacy systems
- Modern Protocol: Clean boot interface with rich information
- Active Development: Well-documented and maintained
- No OS Required: Direct kernel loading

## Prerequisites

### Linux/macOS

```bash
# xorriso for ISO creation
sudo apt install xorriso    # Ubuntu/Debian
sudo pacman -S libisoburn   # Arch Linux
brew install xorriso        # macOS

# Git (for cloning Limine)
sudo apt install git

# Make (for building Limine)
sudo apt install build-essential
```

### Windows

- **Git for Windows**: [https://git-scm.com/download/win](https://git-scm.com/download/win)
- **xorriso**: Use WSL (Windows Subsystem for Linux) or download mkisofs
- Alternatively, use the Linux instructions in WSL

## Building an ISO

### Quick Build (Linux/macOS)

```bash
# One-command ISO creation
./build-iso.sh
```

This script will:
1. Build the kernel with Limine support
2. Download and compile Limine bootloader
3. Set up the ISO directory structure
4. Copy kernel and bootloader files
5. Create the bootable ISO image
6. Install Limine to the ISO

### Windows Build

```powershell
# Run the PowerShell script
.\build-iso.ps1
```

**Note**: Windows users may need to manually download prebuilt Limine binaries from:
[https://github.com/limine-bootloader/limine/releases](https://github.com/limine-bootloader/limine/releases)

### Manual Build Steps

If you prefer to build manually:

```bash
# 1. Build kernel with Limine feature (disable default bootloader)
cargo build --target x86_64-rustrial_os.json --release --no-default-features --features limine

# 2. Clone and build Limine
# The project's `build-iso.sh` uses the `v8.x-binary` branch by default; you can
# use that or a specific stable tag such as `v8.0.17` if you need a fixed release.
git clone https://github.com/limine-bootloader/limine.git --branch=v8.x-binary --depth=1
cd limine && make && cd ..

# 3. Set up ISO structure
mkdir -p target/iso_build/boot target/iso_build/EFI/BOOT

# 4. Copy files
cp target/x86_64-rustrial_os/release/rustrial_os target/iso_build/boot/kernel.elf
# `limine.cfg` (or `limine.conf` depending on Limine version) must be placed
# next to the Limine binary files at the ISO root or boot directory used by Limine.
cp limine.cfg target/iso_build/boot/
cp limine/limine-bios.sys target/iso_build/boot/
cp limine/limine-bios-cd.bin target/iso_build/boot/
cp limine/limine-uefi-cd.bin target/iso_build/boot/
cp limine/BOOTX64.EFI target/iso_build/EFI/BOOT/
cp limine/BOOTIA32.EFI target/iso_build/EFI/BOOT/

# 5. Create ISO
xorriso -as mkisofs \
    -b boot/limine-bios-cd.bin \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    --efi-boot boot/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image \
    --protective-msdos-label \
    target/iso_build -o rustrial_os.iso

# 6. Install Limine bootloader
./limine/limine bios-install rustrial_os.iso
```

## Testing the ISO

### QEMU (Recommended for Testing)

```bash
# Basic test
qemu-system-x86_64 -cdrom rustrial_os.iso -m 256M

# With serial output and networking
qemu-system-x86_64 -cdrom rustrial_os.iso -m 256M -serial stdio \
    -netdev user,id=net0 -device rtl8139,netdev=net0

# With more memory and better graphics
qemu-system-x86_64 -cdrom rustrial_os.iso -m 512M -serial stdio \
    -vga std -display gtk \
    -netdev user,id=net0 -device rtl8139,netdev=net0
```

### VirtualBox

1. Create a new VM (Type: Linux, Version: Other Linux 64-bit)
2. Allocate at least 256 MB RAM
3. Go to Settings → Storage
4. Add Optical Drive and select `rustrial_os.iso`
5. Enable EFI in System settings (optional, for UEFI boot)
6. Start the VM

### VMware

1. Create a new VM (Guest OS: Other Linux 64-bit)
2. Allocate at least 256 MB RAM
3. Use ISO image → Browse to `rustrial_os.iso`
4. Configure network adapter (optional, for networking features)
5. Power on the VM

### Real Hardware

**USB Drive:**
```bash
# WARNING: This will erase all data on the USB drive!
sudo dd if=rustrial_os.iso of=/dev/sdX bs=4M status=progress sync
# Replace /dev/sdX with your USB device (check with lsblk)
```

**CD/DVD:**
- Use your operating system's built-in disc burning tool
- On Linux: `brasero`, `k3b`, or `xfburn`
- On Windows: Windows Media Player, ImgBurn, or CDBurnerXP
- On macOS: Disk Utility

## Boot Process

When booting the ISO, Limine will:

1. **Initialize**: Set up the boot environment
2. **Load Kernel**: Read `kernel.elf` from the ISO
3. **Provide Info**: Pass memory map, framebuffer, HHDM offset to kernel
4. **Transfer Control**: Jump to the kernel's `_start` function

The kernel then:
1. Initializes GDT, IDT, and interrupts
2. Sets up paging with Limine's HHDM offset
3. Initializes the heap allocator
4. Loads the desktop environment

## Configuration

Edit `limine.cfg` to customize the boot menu:

```
# Timeout in seconds
timeout: 5

# Graphics mode
graphics: yes

# Verbose output
verbose: yes

# Menu entry
/Rustrial OS
    protocol: limine
    kernel_path: boot():/kernel.elf
```

### Available Options

- `timeout`: Boot menu timeout in seconds (0 = no timeout)
- `graphics: yes/no`: Enable graphical boot menu
- `verbose: yes/no`: Show detailed boot information
- `resolution`: Set framebuffer resolution (e.g., `1024x768`)
- `kernel_cmdline`: Pass command-line arguments to kernel

## Dual Bootloader Support

The project supports three bootloader modes via Cargo features:

### 1. Default (Bootloader Crate)

```bash
# Quick testing with cargo run
cargo run
```

Uses the `bootloader = "0.9"` crate. Best for development and testing.

### 2. Limine (ISO)

```bash
# Build for ISO distribution
cargo build --no-default-features --features limine
./build-iso.sh
```

Uses Limine bootloader. Experimental support — see the "Current Status" section above.

### 3. Custom Bootloader (Legacy)

```bash
# Build with custom bootloader
cargo build --features custom_bootloader
```

Uses the legacy custom bootloader in `boot/`. Not recommended for new projects.

## Technical Details

### Memory Initialization

Limine provides memory information through the **Higher Half Direct Map (HHDM)**:

```rust
// Get physical memory offset from Limine
let boot_info = LimineBootInfo::get();
let phys_mem_offset = boot_info.physical_memory_offset()
    .expect("Limine did not provide HHDM offset");

// Initialize paging
let mut mapper = unsafe { memory::init(phys_mem_offset) };
```

### Memory Map Conversion

The frame allocator converts Limine's memory map entries:

```rust
// Limine entry types → Rustrial memory regions
USABLE → Usable (available for allocation)
BOOTLOADER_RECLAIMABLE → Usable (can be reused after boot)
KERNEL_AND_MODULES → InUse (kernel code/data)
RESERVED → Reserved (don't touch)
```

### Entry Point

Limine jumps to `_start` with C calling convention:

```rust
#[cfg(feature = "limine")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    kernel_main_limine()
}
```

## Troubleshooting

### ISO doesn't boot

- **Check BIOS/UEFI**: Try both boot modes
- **Verify ISO**: Test with QEMU first before real hardware
- **Re-run limine bios-install**: `./limine/limine bios-install rustrial_os.iso`

#### Capturing serial logs & testing in QEMU

When Limine boots but the screen stays black, serial logs are the most reliable way to see what the kernel did. Run QEMU with `-serial stdio` to forward the guest serial console to your terminal. Example commands:

```bash
# Basic serial output
qemu-system-x86_64 -cdrom rustrial_os.iso -m 256M -serial stdio

# Serial + VGA + GTK display
qemu-system-x86_64 -cdrom rustrial_os.iso -m 512M -serial stdio -vga std -display gtk

# Save serial output to a file
qemu-system-x86_64 -cdrom rustrial_os.iso -m 256M -serial file:serial.log
```

Tips:
- Ensure `limine.cfg` (or `limine.conf`) and `limine-bios.sys` are present in the ISO root/boot directory — Limine requires both.
- Enable verbose Limine output by setting `verbose: yes` (or `VERBOSE=yes` in your version of the config) so the bootloader prints handoff information.
- If the serial logs show HHDM offset and early VGA writes (see example logs in the "Current Status" section), but the screen is black, the likely causes are:
    - The kernel tried to write to VGA before the HHDM mapping was applied for all drawing code paths (we added `*_with_hhdm` initializers to help with that).
    - The frame buffer is in graphics mode instead of text mode; try `-vga std` or different QEMU VGA options.
    - The kernel's GDT/IDT/paging setup differs on real hardware; test on multiple VM backends (QEMU, VirtualBox).

If you need, I can add a small checklist that prints extra debug info very early (serial-only) to help isolate where rendering stops.

### Kernel panics on boot

- **Memory issues**: Limine's memory map might differ from bootloader crate
- **Check serial output**: Add `-serial stdio` to QEMU
- **Verify feature flag**: Ensure you built with `--features limine`

### Build errors

- **Missing xorriso**: Install with package manager
- **Limine clone failed**: Check internet connection or clone manually
- **Cargo build failed**: Ensure you have Rust nightly: `rustup override set nightly`

### ISO too large

The ISO includes:
- Kernel binary (~2-5 MB)
- Limine bootloader (~1 MB)
- Configuration files (~1 KB)

Total size is typically 3-10 MB depending on debug symbols.

To reduce size:
```bash
# Strip debug symbols
cargo build --target x86_64-rustrial_os.json --release --features limine
strip target/x86_64-rustrial_os/release/rustrial_os
```

## File Structure

After building, your workspace will contain:

```
rustrial-os/
├── limine.cfg              # Limine configuration
├── build-iso.sh            # Linux/macOS build script
├── build-iso.ps1           # Windows build script
├── rustrial_os.iso         # Bootable ISO (generated)
├── limine/                 # Limine bootloader (downloaded)
└── target/
    └── iso_build/          # Temporary ISO structure
        ├── boot/
        │   ├── kernel.elf
        │   ├── limine.cfg
        │   └── [bootloader files]
        └── EFI/
            └── BOOT/
                └── [UEFI bootloaders]
```

## Additional Resources

- **Limine Documentation**: [https://github.com/limine-bootloader/limine](https://github.com/limine-bootloader/limine)
- **Limine Protocol Spec**: [https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md](https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md)
- **OSDev Wiki**: [https://wiki.osdev.org/Limine](https://wiki.osdev.org/Limine)
- **Rust OSDev**: [https://os.phil-opp.com/](https://os.phil-opp.com/)


## License

The Limine bootloader is licensed under BSD-2-Clause. Rustrial OS integration is part of the main project license.
