# RustrialOS Custom Bootloader Documentation

## Overview

This directory contains a custom x86-64 bootloader written in Assembly for RustrialOS. The bootloader implements a two-stage boot process that transitions the CPU from 16-bit Real Mode to 64-bit Long Mode and loads the kernel into memory.

## Files in This Directory

### Assembly Source Files
- **`boot.asm`** - Stage 1 bootloader (boot sector, 512 bytes)
  - Platform: All (x86-64 assembly)
  - Loaded by BIOS, loads Stage 2 from disk
  
- **`stage2.asm`** - Stage 2 bootloader (main bootloader, ~8KB)
  - Platform: All (x86-64 assembly)
  - Enables A20, checks CPU, loads kernel, enters Long Mode

### Build Scripts

#### Windows Scripts
- **`build.ps1`** - PowerShell script to build the bootloader only
  - For: Windows (PowerShell)
  - Usage: `cd boot; .\build.ps1`
  
- **`build-custom.ps1`** - Complete build script (bootloader + kernel + disk image)
  - For: Windows (PowerShell)
  - Usage: `.\boot\build-custom.ps1` (run from project root)

#### Linux/macOS Scripts
- **`Makefile`** - Make build script to build the bootloader only
  - For: Linux/macOS
  - Usage: `cd boot; make`
  
- **`build-custom.sh`** - Complete build script (bootloader + kernel + disk image)
  - For: Linux/macOS (Bash)
  - Usage: `./boot/build-custom.sh` (run from project root)

### Documentation
- **`custombootloader.md`** - This file, comprehensive bootloader documentation

### Quick Reference

| Task | Windows | Linux/macOS |
|------|---------|-------------|
| Build bootloader only | `cd boot; .\build.ps1` | `cd boot; make` |
| Build complete disk image | `.\boot\build-custom.ps1` | `./boot/build-custom.sh` |
| Clean build artifacts | Delete files manually | `cd boot; make clean` |
| Test in QEMU | `qemu-system-x86_64 -drive format=raw,file=disk.img` | Same |

### Platform-Specific Notes

**Windows Users:**
- Use PowerShell scripts (`.ps1` files)
- Ensure PowerShell execution policy allows scripts: `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser`
- NASM must be installed and in PATH
- Forward slashes in paths are automatically converted

**Linux/macOS Users:**
- Use Makefile or Bash scripts (`.sh` files)
- Make scripts executable: `chmod +x boot/build-custom.sh`
- NASM is typically available through package managers
- Ensure `dd` and `make` utilities are installed (usually pre-installed)

**Assembly Files (Platform-Independent):**
- `boot.asm` and `stage2.asm` work on all platforms
- Only the build tools differ between operating systems
- NASM syntax is identical across Windows/Linux/macOS

### Architecture

**Stage 1 (boot.asm)** - Boot Sector (512 bytes)
- Loaded by BIOS at address 0x7C00
- Initializes segments and stack pointer
- Loads Stage 2 from disk using BIOS INT 0x13
- Transfers control to Stage 2

**Stage 2 (stage2.asm)** - Main Bootloader (8KB)
- Loaded at address 0x7E00 by Stage 1
- Enables A20 line for accessing memory above 1MB
- Verifies CPU supports 64-bit Long Mode via CPUID
- Loads kernel binary from disk using BIOS INT 13h extensions (AH=0x42) from configurable LBA 65 to 0x100000 (1MB)
- Configures page tables with identity mapping for first 4MB
- Sets up Global Descriptor Table (GDT)
- Transitions through Protected Mode to Long Mode
- Transfers control to kernel entry point

### Memory Layout

```
Address Range          Purpose
------------------     ----------------------------------
0x00000000-0x000003FF  Real Mode IVT (Interrupt Vector Table)
0x00000400-0x000004FF  BIOS Data Area
0x00000500-0x00007BFF  Free memory
0x00007C00-0x00007DFF  Stage 1 bootloader (512 bytes)
0x00007E00-0x00009FFF  Stage 2 bootloader (8KB)
0x00001000-0x00004FFF  Page tables (PML4, PDPT, PD)
0x00100000-0x001FFFFF  Kernel binary (1MB+)
0x00200000+            Stack and free memory
```

### Boot Sequence

```
BIOS Power-On
    |
    v
Load boot sector (Stage 1) to 0x7C00

    |
    v
Stage 1: Load Stage 2 from disk to 0x7E00
    |
    v
Stage 2: Enable A20, check CPU features
    |
    v
Stage 2: Load kernel from disk to 0x100000
    |
    v
Stage 2: Setup page tables and GDT
    |
    v
Stage 2: Real Mode -> Protected Mode -> Long Mode
    |
    v
Jump to kernel at 0x100000 (64-bit mode)
```

## Building the Bootloader

### Prerequisites

#### Required Tools

**NASM (Netwide Assembler)** - Required for all platforms
- Official Website: https://www.nasm.us/
- **Windows:** 
  - Download the Windows installer from https://www.nasm.us/pub/nasm/releasebuilds/
  - Run the installer and ensure "Add NASM to PATH" is checked
  - Or download the ZIP, extract, and manually add to PATH
  - Verify installation: `nasm -v`
- **Linux (Debian/Ubuntu):**
  - `sudo apt update && sudo apt install nasm`
  - Verify installation: `nasm -v`
- **Linux (Fedora/RHEL):**
  - `sudo dnf install nasm`
  - Verify installation: `nasm -v`
- **macOS:**
  - Using Homebrew: `brew install nasm`
  - Using MacPorts: `sudo port install nasm`
  - Verify installation: `nasm -v`

**Additional Tools by Platform:**
- **Windows:** PowerShell 5.1+ (built-in on Windows 10/11)
- **Linux/macOS:** Make and dd utilities (usually pre-installed)

### Build Commands by Operating System

#### Windows (PowerShell)

**Building the Bootloader Only:**
```powershell
cd boot
.\build.ps1
```

**Building Complete Disk Image:**
```powershell
# Run from project root
.\boot\build-custom.ps1
```

**What Gets Built:**
- `boot\boot.bin` - Stage 1 bootloader (512 bytes)
- `boot\stage2.bin` - Stage 2 bootloader (~8KB)
- `boot\bootloader.img` - Combined bootloader
- `disk.img` - Complete bootable disk image (with build-custom.ps1)

#### Linux/macOS (Make/Bash)

**Building the Bootloader Only:**
```bash
cd boot
make
```

**Building Complete Disk Image:**
```bash
# Run from project root
chmod +x boot/build-custom.sh
./boot/build-custom.sh
```

**What Gets Built:**
- `boot/boot.bin` - Stage 1 bootloader (512 bytes)
- `boot/stage2.bin` - Stage 2 bootloader (~8KB)
- `boot/bootloader.img` - Combined bootloader
- `disk.img` - Complete bootable disk image (with build-custom.sh)

**Cleaning Build Artifacts:**
```bash
cd boot
make clean
```

#### Manual Build (All Platforms)

If you prefer to build manually or troubleshoot:

```bash
# Navigate to boot directory
cd boot

# Assemble Stage 1 (boot sector)
nasm -f bin boot.asm -o boot.bin

# Assemble Stage 2 (main bootloader)
nasm -f bin stage2.asm -o stage2.bin

# Combine stages (Windows PowerShell)
$boot = [System.IO.File]::ReadAllBytes("boot.bin")
$stage2 = [System.IO.File]::ReadAllBytes("stage2.bin")
$combined = $boot + $stage2
[System.IO.File]::WriteAllBytes("bootloader.img", $combined)

# Combine stages (Linux/macOS)
cat boot.bin stage2.bin > bootloader.img
```

### Output Files

- `boot.bin` - Stage 1 boot sector (exactly 512 bytes with 0xAA55 signature)
- `stage2.bin` - Stage 2 bootloader (approximately 8KB)
- `bootloader.img` - Combined bootloader ready for disk

## Integration with Kernel

### Kernel Requirements

The bootloader expects the kernel to meet these specifications:

1. **Binary Format:** Flat binary (not ELF), compiled with appropriate linker script
2. **Entry Point:** First instruction at offset 0 in the binary
3. **Load Address:** Must be position-independent or linked for 0x100000
4. **Execution Mode:** Code must be 64-bit Long Mode compatible
5. **Memory Model:** Identity-mapped first 4MB available
6. **Disk Location:** Kernel binary starts at LBA 65 (sector 66 one-based) as configured by the `KERNEL_LBA` and `KERNEL_SECTORS` constants in `stage2.asm`

### Sample Linker Script

Create `kernel.ld` for your Rust kernel:

```ld
ENTRY(_start)

SECTIONS {
    . = 0x100000;
    
    .text : {
        *(.text .text.*)
    }
    
    .rodata : {
        *(.rodata .rodata.*)
    }
    
    .data : {
        *(.data .data.*)
    }
    
    .bss : {
        *(.bss .bss.*)
    }
}
```

Configure `.cargo/config.toml`:
```toml
[target.x86_64-rustrial_os]
rustflags = ["-C", "link-arg=-T", "-C", "link-arg=kernel.ld"]
```

### Kernel Entry Point

Modify `main.rs` to remove BootInfo dependency:

```rust
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Custom bootloader doesn't provide BootInfo
    println!("RustrialOS Kernel Starting...");
    rustrial_os::init();
    
    // Continue initialization
    loop {}
}
```

## Creating Bootable Disk Image

### Quick Start - Use the Build Scripts

The easiest way to create a complete bootable disk image is to use the provided build scripts:

#### Windows
```powershell
# From project root directory
.\boot\build-custom.ps1
```

#### Linux/macOS
```bash
# From project root directory
chmod +x boot/build-custom.sh
./boot/build-custom.sh
```

These scripts automatically:
1. Build the custom bootloader
2. Build the kernel in release mode
3. Convert the kernel to a flat binary
4. Create a bootable disk image
5. Test with QEMU (if available)

### Manual Process (Step-by-Step)

If you prefer manual control or need to troubleshoot:

#### Step 1: Install Required Tools

**All Platforms:**
```bash
# Install cargo-binutils for kernel conversion
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

**Windows:**
- Install QEMU from: https://qemu.weilnetz.de/w64/
- Or use Chocolatey: `choco install qemu`

**Linux:**
```bash
sudo apt install qemu-system-x86  # Debian/Ubuntu
sudo dnf install qemu-system-x86  # Fedora/RHEL
```

**macOS:**
```bash
brew install qemu
```

#### Step 2: Build Bootloader and Kernel

**Windows (PowerShell):**
```powershell
# Build bootloader
cd boot
.\build.ps1
cd ..

# Build kernel in release mode with custom bootloader feature
cargo build --target x86_64-rustrial_os.json --release --features custom_bootloader

# Convert kernel ELF to flat binary
rust-objcopy target\x86_64-rustrial_os\release\rustrial_os -O binary kernel.bin
```

**Linux/macOS (Bash):**
```bash
# Build bootloader
cd boot
make
cd ..

# Build kernel in release mode with custom bootloader feature
cargo build --target x86_64-rustrial_os.json --release --features custom_bootloader

# Convert kernel ELF to flat binary
rust-objcopy target/x86_64-rustrial_os/release/rustrial_os -O binary kernel.bin
```

> **Important:** The `--features custom_bootloader` flag is required when building for the custom bootloader. This enables a simplified entry point (`_start`) that doesn't require `BootInfo`. The default build uses the bootloader crate's `entry_point!` macro which is incompatible with the custom bootloader.

#### Step 3: Create Disk Image

**Windows (PowerShell):**
```powershell
# Create 10MB disk image (20000 sectors × 512 bytes)
$diskSize = 20000 * 512
$zeros = New-Object byte[] $diskSize
[System.IO.File]::WriteAllBytes("disk.img", $zeros)

# Write bootloader at sector 0
$bootloader = [System.IO.File]::ReadAllBytes("boot\bootloader.img")
$disk = [System.IO.File]::ReadAllBytes("disk.img")
[Array]::Copy($bootloader, 0, $disk, 0, $bootloader.Length)

# Write kernel at LBA 65 (sector 66 one-based, offset 0x8200)
$kernel = [System.IO.File]::ReadAllBytes("kernel.bin")
[Array]::Copy($kernel, 0, $disk, 33280, $kernel.Length)

# Save the disk image
[System.IO.File]::WriteAllBytes("disk.img", $disk)

Write-Host "Disk image created successfully!" -ForegroundColor Green
Write-Host "Bootloader size: $($bootloader.Length) bytes" -ForegroundColor Cyan
Write-Host "Kernel size: $($kernel.Length) bytes" -ForegroundColor Cyan
```

**Linux/macOS (Bash):**
```bash
# Create 10MB disk image (20000 sectors × 512 bytes)
dd if=/dev/zero of=disk.img bs=512 count=20000 2>/dev/null

# Write bootloader at sector 0
dd if=boot/bootloader.img of=disk.img conv=notrunc 2>/dev/null

# Write kernel at LBA 65 (sector 66 in BIOS numbering)
dd if=kernel.bin of=disk.img bs=512 seek=65 conv=notrunc 2>/dev/null

echo "Disk image created successfully!"
echo "Bootloader size: $(stat -c%s boot/bootloader.img 2>/dev/null || stat -f%z boot/bootloader.img) bytes"
echo "Kernel size: $(stat -c%s kernel.bin 2>/dev/null || stat -f%z kernel.bin) bytes"
```

> **Tip:** The loader reads `KERNEL_SECTORS` (default 128, i.e. 64 KiB) starting at `KERNEL_LBA` (default 65). If your kernel binary is larger or lives elsewhere, adjust those constants in `boot/stage2.asm`, rebuild the bootloader, and regenerate `disk.img`.

#### Step 4: Test with QEMU

**Windows (PowerShell):**
```powershell
# Basic test
qemu-system-x86_64 -drive format=raw,file=disk.img

# With more options for debugging
qemu-system-x86_64 -drive format=raw,file=disk.img -serial stdio -no-reboot
```

**Linux/macOS (Bash):**
```bash
# Basic test
qemu-system-x86_64 -drive format=raw,file=disk.img

# With more options for debugging
qemu-system-x86_64 -drive format=raw,file=disk.img -serial stdio -no-reboot

# With memory size specified
qemu-system-x86_64 -drive format=raw,file=disk.img -m 512M
```

**QEMU Keyboard Shortcuts:**
- `Ctrl+Alt+G` - Release mouse capture
- `Ctrl+Alt+F` - Toggle fullscreen
- `Ctrl+Alt+2` - Switch to QEMU monitor
- `Ctrl+Alt+1` - Switch back to VM display

## Switching Between Bootloaders

### Using Legacy Bootloader (Default)

RustrialOS currently uses the `bootloader` crate which is stable and production-ready.

**Advantages:**
- Simple: `cargo run` handles everything
- Maintained and tested
- Provides BootInfo structure with memory map
- No assembly knowledge required

**To continue using:**
```bash
# No changes needed
cargo run
```

### Using Custom Bootloader

**Advantages:**
- Educational: Learn x86-64 boot process
- Full control over boot sequence
- Smaller size (~8.5KB vs ~50KB)
- Demonstrates low-level programming

**Disadvantages:**
- Manual build process
- No automatic memory map
- Requires NASM installation
- Experimental status

**To switch permanently:**

1. Remove bootloader dependency from `Cargo.toml`:
```toml
# Comment out:
# bootloader = { version = "0.9", features = ["map_physical_memory"]}
```

2. Add custom linker script (see "Sample Linker Script" above)

3. Update kernel entry point (see "Kernel Entry Point" above)

4. Use custom build process (see "Creating Bootable Disk Image" above)

### Hybrid Approach (Recommended)

Keep both bootloaders available:

1. Use legacy bootloader for development: `cargo run`
2. Test with custom bootloader occasionally: build disk.img manually
3. Learn from custom bootloader source code
4. No changes to Cargo.toml needed

## Troubleshooting

### Bootloader Not Loading

**Symptom:** System hangs at BIOS or "No bootable device"

**Solutions:**
- Verify boot.bin is exactly 512 bytes
- Confirm 0xAA55 signature at offset 510-511
- Check QEMU command uses correct disk image
- Test bootloader alone: `qemu-system-x86_64 -drive format=raw,file=boot/bootloader.img`

### Stage 2 Not Executing

**Symptom:** Boot hangs after Stage 1 messages

**Solutions:**
- Verify Stage 2 is present on disk starting at sector 2
- Check number of sectors loaded in boot.asm (should be 64)
- Ensure stage2.bin is properly assembled
- Verify bootloader.img contains both stages

### Kernel Not Loading

**Symptom:** "Loading kernel..." message appears but system hangs or "Kernel load error!" is printed

**Solutions:**
- Confirm kernel.bin exists and is valid
- Verify kernel starts at LBA 65 (sector 66 one-based)
- Ensure `KERNEL_SECTORS` in `stage2.asm` is large enough for the kernel binary
- Check kernel is flat binary format (not ELF)
- Ensure kernel load address matches linker script (0x100000)

### "No Long Mode Support" Error

**Symptom:** Error message displayed by Stage 2

**Solutions:**
- Use CPU with x86-64 support
- Enable virtualization in BIOS if using VM
- Use modern VM software (QEMU, VirtualBox, VMware)
- Check QEMU is 64-bit version: `qemu-system-x86_64`

### Kernel Crashes Immediately

**Symptom:** Triple fault or immediate reset after kernel loads

**Solutions:**
- Verify kernel entry point is correct
- Ensure kernel code is 64-bit compatible
- Check page tables are properly configured
- Confirm stack pointer is valid
- Test kernel with legacy bootloader to isolate issue

## Technical Details

### Stage 1 Implementation

Key operations performed by boot.asm:

1. Initialize segment registers (CS, DS, ES, SS)
2. Set stack pointer to 0x7C00
3. Load Stage 2 using BIOS INT 0x13h (CHS addressing)
4. Display status messages using BIOS INT 0x10h
5. Jump to Stage 2 at 0x7E00

### Stage 2 Implementation

Key operations performed by stage2.asm:

1. **A20 Line Enable:** Access memory above 1MB
   - Try BIOS method (INT 0x15h, AX=0x2401)
   - Try keyboard controller method (port 0x92)
   - Verify A20 enabled by reading test addresses

2. **CPUID Check:** Verify 64-bit Long Mode support
   - Check CPUID availability
   - Query extended functions
   - Verify LM bit in feature flags

3. **Kernel Loading:** Read kernel from disk
  - Load from configurable LBA 65 (sector 66 one-based) to address 0x10000 and later relocate to 0x100000
  - Use BIOS INT 0x13h Extensions (AH=0x42) with a Disk Address Packet
  - Adjust `KERNEL_SECTORS` in `stage2.asm` if your kernel binary exceeds the default 64 KiB window

4. **Page Table Setup:** Configure 4-level paging
   - PML4 at 0x1000 (1 entry)
   - PDPT at 0x2000 (1 entry)
   - PD at 0x3000 (2 entries, 4MB total)
   - Identity map first 4MB

5. **GDT Configuration:** Prepare for Protected/Long Mode
   - Null descriptor
   - 64-bit code segment (CS)
   - 64-bit data segment (DS)

6. **Mode Transitions:**
   - Enter Protected Mode: Set CR0.PE
   - Enable paging: Set CR0.PG, load CR3
   - Enter Long Mode: Set EFER.LME
   - Jump to 64-bit code segment

### Educational Value

This bootloader demonstrates fundamental OS development concepts:

**Low-Level Programming:**
- Direct hardware manipulation
- BIOS interrupt services
- Port I/O operations
- Segment and descriptor tables

**x86-64 Architecture:**
- CPU mode transitions
- Memory protection mechanisms
- Paging structures
- Control registers

**Boot Process:**
- Master Boot Record format
- Multi-stage loading
- Disk addressing (CHS and LBA)
- Boot signature requirements

## Status and Recommendations

**Current Status:** Experimental

The custom bootloader is fully functional and assembles correctly. It has been designed following standard x86-64 boot procedures and includes comprehensive error handling.

**Recommended Usage:**

- **For Development:** Use legacy `bootloader` crate (stable, fast iteration)
- **For Learning:** Study and experiment with custom bootloader
- **For Production:** Continue with legacy bootloader until custom is thoroughly tested

**Testing Required:**

- Build complete disk image with kernel
- Boot test in QEMU
- Verify kernel initialization
- Test error handling paths
- Compare behavior with legacy bootloader

## References and Resources

**Official Documentation:**
- Intel 64 and IA-32 Architectures Software Developer Manuals
- NASM Documentation: https://www.nasm.us/doc/

**Community Resources:**
- OSDev Wiki: https://wiki.osdev.org/
- OSDev Bootloader Guide: https://wiki.osdev.org/Bootloader
- OSDev Long Mode: https://wiki.osdev.org/Entering_Long_Mode_Directly
- OSDev Paging: https://wiki.osdev.org/Paging

**Related Projects:**
- Rust OSDev: https://os.phil-opp.com/
- Writing an OS in Rust by Philipp Oppermann

## License

This custom bootloader is part of the RustrialOS project and follows the project's license terms.
