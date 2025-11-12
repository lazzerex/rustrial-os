# Rustrial OS

<img width="1334" height="421" alt="Rustrial OS Banner" src="https://github.com/user-attachments/assets/6c85e6bc-03b3-49c7-a90b-c38535edd7c1" />

<p align="center">
  <img src="https://img.shields.io/badge/Rust-%23000000.svg?style=flat&logo=rust&logoColor=white"/>
  <img src="https://img.shields.io/badge/x86__64-blue?style=flat&logo=intel&logoColor=white"/>
  <img src="https://img.shields.io/badge/QEMU-orange?style=flat&logo=qemu&logoColor=white"/>
  <img src="https://img.shields.io/badge/Bootloader-grey?style=flat"/>
  <img src="https://img.shields.io/badge/LLVM-%23000000.svg?style=flat&logo=llvm&logoColor=white"/>
  <img src="https://img.shields.io/badge/Bare%20Metal-purple?style=flat"/>
  <img src="https://img.shields.io/badge/VGA%20Buffer-green?style=flat"/>
</p>

<p align="center">
  <img src="https://img.shields.io/github/stars/lazzerex/rustrial-os?style=flat&logo=github"/>
  <img src="https://img.shields.io/github/forks/lazzerex/rustrial-os?style=flat&logo=github"/>
  <img src="https://img.shields.io/github/contributors/lazzerex/rustrial-os?style=flat"/>
  <img src="https://img.shields.io/github/issues-pr-raw/lazzerex/rustrial-os?label=pull%20requests&style=flat&color=yellow"/>
  <img src="https://img.shields.io/github/issues/lazzerex/rustrial-os?label=issues&style=flat&color=red"/>
  <img src="https://img.shields.io/crates/v/cargo.svg?label=cargo&logo=rust&style=flat&color=orange"/>
  <img src="https://img.shields.io/badge/rustup-nightly-blue?style=flat&logo=rust"/>
</p>

---

# Rustrial OS

A hobby x86-64 bare-metal operating system kernel written in Rust, built from scratch to explore low-level systems programming, memory management, and async concurrency on real hardware.

## Overview

Rustrial OS is an educational operating system project that demonstrates modern systems programming techniques using Rust. The kernel boots from bare metal on x86-64 architecture and provides core OS functionality including memory management, interrupt handling, task scheduling, and keyboard input processing.

You can refer to ```src/rustrial_script/docs``` to know more about how RustrialScripts is implemented.
Also checkout ```boot/custombootloader.md``` if you want to use a custom bootloader instead of the prebuilt bootloader crate.

## Showcase

<details markdown="1"> <summary>Menu Screen & Filesystem</summary>

<img width="722" height="389" alt="os-1" src="https://github.com/user-attachments/assets/5bf0b6a0-b3e6-4494-85ad-4b16e5f1779d" />

<img width="699" height="368" alt="os-2" src="https://github.com/user-attachments/assets/d36d6064-ecb4-41c6-ba16-d1fd1c10af1e" />

<img width="722" height="386" alt="os-3" src="https://github.com/user-attachments/assets/7b7c1e10-fe0e-43a1-a7f9-d67ad33ef67c" />

<img width="723" height="389" alt="os-4" src="https://github.com/user-attachments/assets/40df0234-8f02-4e79-b71e-a3db7b4e83e5" />

<img width="459" height="311" alt="os-5" src="https://github.com/user-attachments/assets/8b5b62d4-ebc0-4bcf-bc76-2639bfd9d39f" />

</details>


<details markdown="1"> <summary>Running Some Unit Tests</summary>
  
<img width="909" height="144" alt="image" src="https://github.com/user-attachments/assets/ea1a4b66-5351-47c8-9e56-83366e27fb87" />

<img width="906" height="140" alt="image" src="https://github.com/user-attachments/assets/90d5622b-060a-41a2-a25c-d617b8e69903" />

</details>


## Features

### Core Kernel Features
- **Bare Metal x86-64 Architecture**: Custom bootloader integration using the Bootloader crate, supporting both real hardware and QEMU emulation
- **Interrupt Handling**: Complete IDT (Interrupt Descriptor Table) setup with handlers for:
  - Breakpoint exceptions
  - Double fault handling with dedicated stack
  - Timer interrupts (PIC 8259 chained controller)
  - Keyboard input interrupts
  - Page fault exceptions with detailed error reporting
- **Memory Management**: 
  - Paging support with virtual to physical address translation
  - Configurable heap allocation at `0x_4444_4444_0000` with 100KB default size
  - Multiple allocator strategies available (bump allocator, linked list allocator, fixed-size block allocator)
  - Frame allocation from bootloader memory map
- **Global Descriptor Table (GDT)**: CPU segmentation setup with double fault IST (Interrupt Stack Table) support
- **Async Task System**: 
  - Futures-based async/await support with custom executor
  - Keyboard event handling as async tasks
  - Waker-based task scheduling


### VGA Buffer & I/O
- **VGA Text Output**: Direct memory-mapped VGA buffer (0xb8000) for console printing
- **Graphics System** (`src/graphics/`): Text-mode visual enhancements
  - Box drawing (single/double lines, shadow effects) using IBM PC box-drawing characters
  - Progress bars with percentage display and animations
  - UI components (message boxes, status bars, terminal windows, menus)
  - Splash screens with ASCII art and loading animations
  - Interactive graphics demo showcasing all features
  - VGA Mode 13h (320x200x256) pixel graphics framework (experimental)
- **Serial Port I/O**: UART 16550 serial communication for test output and debugging
- **Keyboard Input**: Async keyboard task for processing PS/2 keyboard scancodes

#### Graphics Quick Start
```rust
use rustrial_os::graphics::{text_graphics::*, demo::*, splash::*};
use rustrial_os::vga_buffer::Color;

// Box drawing
draw_box(10, 5, 40, 10, Color::Cyan, Color::Black);
draw_double_box(15, 8, 50, 8, Color::Yellow, Color::Blue);

// Progress bar
draw_progress_bar(10, 12, 50, 75, 100, Color::Green, Color::Black);

// UI elements
show_message_box("Success", "Operation complete!", 20, 10, 40);
show_status_bar("Rustrial OS v0.1.0 | F1=Help");

// Run full demo
run_graphics_demo();

// Splash screen
show_boot_splash();
```
See `src/graphics/README.md` for detailed API documentation and examples.

### Filesystem & Storage
- **In-Memory Filesystem (RAMfs)**: Complete virtual filesystem implementation with:
  - File creation, reading, writing, and deletion
  - Directory support and navigation
  - Virtual File System (VFS) abstraction layer
  - Embedded script loading at boot time
- **Script Loader**: Compile-time embedding of `.rscript` files into the kernel binary
- **Interactive Script Browser**: Navigate and execute scripts from the filesystem with keyboard controls

### RustrialScript Interpreter & Interactive Menu
- **RustrialScript Interpreter**: Minimal, stack-based scripting language for RustrialOS. Runs scripts directly in the OS, with no_std and alloc-only dependencies. Features integer, boolean, and nil types, arithmetic and control flow, and direct integration with VGA output.
- **Interactive Menu System**: Enhanced boot menu with nested navigation:
  - **Option 1**: Normal operation with system information and keyboard echo mode
  - **Option 2**: RustrialScript with two sub-options:
    - Run Demo: Execute pre-coded Fibonacci demonstration
    - Browse Scripts: Interactive filesystem browser with 6 embedded scripts
  - **Option 3**: Comprehensive help documentation
- **Script Browser Features**:
  - Navigate with arrow keys (↑/↓) or W/S
  - Visual selection indicator (→ symbol)
  - Press Enter to execute selected script
  - Real-time script execution with VGA output
  - Smart menu flow: returns to browser after execution
- **Integration**: The interpreter is modular, with lexer, parser, VM, and value modules. Example scripts (fibonacci, factorial, collatz, gcd, prime checker, sum of squares) are embedded at compile time and loaded into the filesystem. See `src/rustrial_script/docs/` for detailed documentation and `FILESYSTEM.md` for filesystem architecture.

## Project Structure

```
src/
├── main.rs                  # Kernel entry point and initialization
├── lib.rs                   # Kernel library with core setup and test infrastructure
├── vga_buffer.rs            # VGA text mode buffer and print macros
├── serial.rs                # Serial port (UART) I/O for debugging
├── gdt.rs                   # Global Descriptor Table configuration
├── interrupts.rs            # Interrupt Descriptor Table and exception handlers
├── memory.rs                # Paging, page tables, and memory translation
├── allocator.rs             # Heap allocator interface
├── rustrial_menu.rs         # Interactive menu system with script browser
├── script_loader.rs         # Compile-time script embedding and loading
├── graphics/                # Graphics system (text-mode and VGA)
│   ├── mod.rs               # Graphics module interface
│   ├── text_graphics.rs     # Box drawing, progress bars, UI components
│   ├── vga_graphics.rs      # VGA Mode 13h pixel graphics (experimental)
│   ├── splash.rs            # Splash screens and fancy UI elements
│   ├── demo.rs              # Interactive graphics demonstrations
│   └── README.md            # Graphics API documentation and examples
├── fs/                      # Filesystem implementation
│   ├── mod.rs               # Filesystem initialization and mounting
│   ├── vfs.rs               # Virtual File System abstraction layer
│   └── ramfs.rs             # RAM-based in-memory filesystem
├── rustrial_script/         # RustrialScript interpreter modules
│   ├── mod.rs               # Main interpreter interface
│   ├── lexer.rs             # Tokenizer for script parsing
│   ├── parser.rs            # Bytecode compiler
│   ├── vm.rs                # Stack-based virtual machine
│   ├── value.rs             # Value types (int, bool, nil)
│   ├── examples/            # Example scripts (*.rscript files)
│   └── docs/                # Language and integration documentation
├── allocator/               # Memory allocator implementations
│   ├── bump.rs              # Simple bump allocator
│   ├── linked_list.rs       # Linked list allocator
│   └── fixed_size_block.rs  # Fixed-size block allocator (default)
└── task/                    # Async task execution system
    ├── mod.rs               # Task structure and TaskId
    ├── executor.rs          # Waker-based task executor
    ├── simple_executor.rs   # Basic single-threaded executor
    └── keyboard.rs          # Async keyboard input processing

tests/                       # Integration tests
├── basic_boot.rs            # Kernel boot verification
├── heap_allocation.rs       # Heap allocator tests
├── should_panic.rs          # Panic handling tests
└── stack_overflow.rs        # Stack overflow exception tests

boot/                        # Custom bootloader (optional)
├── boot.asm                 # Stage 1 bootloader (512 bytes)
├── stage2.asm               # Stage 2 bootloader (Long Mode setup)
├── build.ps1                # Windows build script
├── Makefile                 # Linux/macOS build script
└── custombootloader.md      # Bootloader documentation

Cargo.toml                   # Project dependencies and configuration
x86_64-rustrial_os.json      # Custom target specification
FILESYSTEM.md                # Filesystem architecture documentation
```
### Memory Layout
- **VGA Buffer**: `0xb8000` - Text mode framebuffer
- **Heap**: `0x_4444_4444_0000` - `0x_4444_4445_9000` (100 KiB)
- **Serial Port**: `0x3F8` (COM1)
- **IST Stack**: 20 KiB dedicated stack for double fault handler

## Prerequisites
You should first have ```rustup``` and ```cargo``` installed on your machine. After that, installed all the required tools below. For Linux, you might want to use ```Rustup``` to avoid problems relating to the cargo versions conflicting.
### Required Tools

1. **Rust Nightly Toolchain**
   ```bash
   rustup install nightly
   rustup override set nightly
   rustup component add rust-src llvm-tools-preview
   ```

2. **Bootimage Tool**
   ```bash
   cargo install bootimage
   ```

3. **QEMU Emulator**
   - **Linux**: `sudo apt install qemu-system-x86` (Debian/Ubuntu) or `sudo pacman -S qemu` (Arch)
   - **macOS**: `brew install qemu`
   - **Windows**: Download from [qemu.org](https://www.qemu.org/download/)

### System Requirements

- Rust 1.70+ (nightly)
- LLVM tools (included with Rust toolchain)
- At least 2 GB RAM for building
- Internet connection for dependency downloads

## Build & Run

### Quick Start (Using Bootloader Crate)

This is the recommended method for development and testing:

```bash
# Clone the repository
git clone https://github.com/lazzerex/rustrial-os.git
cd rustrial-os

# Build and run in QEMU (all-in-one command)
cargo run
```

The OS will boot and display an interactive menu where you can:
- Press **1** for normal operation mode
- Press **2** to access RustrialScript (demo or script browser)
- Press **3** to view help documentation

### Using Custom Bootloader (Advanced)

The custom bootloader is written in x86-64 assembly and provides educational insight into the boot process. See `boot/custombootloader.md` for detailed documentation.

#### Prerequisites for Custom Bootloader

1. **NASM (Netwide Assembler)**
   - Windows: Download from [nasm.us](https://www.nasm.us/) and add to PATH
   - Linux: `sudo apt install nasm` (Debian/Ubuntu) or `sudo pacman -S qemu` (Arch)
   - macOS: `brew install nasm`

2. **Verify NASM installation:**
   ```bash
   nasm -v
   # Should output: NASM version 2.xx.xx or higher
   ```

#### Building with Custom Bootloader

**Step 1: Build the Custom Bootloader**

```powershell
# Windows (PowerShell)
cd boot
.\build.ps1
cd ..
```

```bash
# Linux/macOS
cd boot
make
cd ..
```

This creates:
- `boot/boot.bin` - Stage 1 bootloader (512 bytes)
- `boot/stage2.bin` - Stage 2 bootloader (~8KB)
- `boot/bootloader.img` - Combined bootloader image

**Step 2: Build the Kernel**

```bash
# Build kernel for release (optimized)
cargo build --target x86_64-rustrial_os.json --release

# Convert ELF to flat binary
cargo install cargo-binutils
rustup component add llvm-tools-preview
rust-objcopy target/x86_64-rustrial_os/release/rustrial_os -O binary kernel.bin
```

**Step 3: Create Bootable Disk Image**

```powershell
# Windows (PowerShell)
# Create 10MB disk image filled with zeros
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

# Run in QEMU
qemu-system-x86_64 -drive format=raw,file=disk.img
```

```bash
# Linux/macOS
# Create 10MB disk image
dd if=/dev/zero of=disk.img bs=512 count=20000

# Write bootloader at sector 0
dd if=boot/bootloader.img of=disk.img conv=notrunc

# Write kernel at sector 66
dd if=kernel.bin of=disk.img bs=512 seek=65 conv=notrunc

# Run in QEMU
qemu-system-x86_64 -drive format=raw,file=disk.img
```

**Why Sector 66?**
- Sector 0: Boot signature (Stage 1)
- Sectors 1-64: Stage 2 bootloader
- Sector 65+: Reserved
- Sector 66+: Kernel binary

**Complete Build Script (PowerShell)**

Create `build-custom.ps1` in the project root:

```powershell
#!/usr/bin/env pwsh
# Complete custom bootloader build script

Write-Host "Building RustrialOS with Custom Bootloader..." -ForegroundColor Cyan

# Step 1: Build bootloader
Write-Host "`n[1/4] Building custom bootloader..." -ForegroundColor Yellow
cd boot
.\build.ps1
if (-not $?) { exit 1 }
cd ..

# Step 2: Build kernel
Write-Host "`n[2/4] Building kernel..." -ForegroundColor Yellow
cargo build --target x86_64-rustrial_os.json --release
if (-not $?) { exit 1 }

# Step 3: Convert kernel to binary
Write-Host "`n[3/4] Converting kernel to flat binary..." -ForegroundColor Yellow
rust-objcopy target/x86_64-rustrial_os/release/rustrial_os -O binary kernel.bin

# Step 4: Create disk image
Write-Host "`n[4/4] Creating bootable disk image..." -ForegroundColor Yellow
$diskSize = 20000 * 512
$zeros = New-Object byte[] $diskSize
[System.IO.File]::WriteAllBytes("disk.img", $zeros)

$bootloader = [System.IO.File]::ReadAllBytes("boot\bootloader.img")
$disk = [System.IO.File]::ReadAllBytes("disk.img")
[Array]::Copy($bootloader, 0, $disk, 0, $bootloader.Length)

$kernel = [System.IO.File]::ReadAllBytes("kernel.bin")
[Array]::Copy($kernel, 0, $disk, 33280, $kernel.Length)

[System.IO.File]::WriteAllBytes("disk.img", $disk)

Write-Host "`nBuild complete! disk.img created." -ForegroundColor Green
Write-Host "Run with: qemu-system-x86_64 -drive format=raw,file=disk.img" -ForegroundColor Cyan
```

Then run:
```powershell
.\build-custom.ps1
```

**Complete Build Script (Bash)**

Create `build-custom.sh` in the project root:

```bash
#!/bin/bash
# Complete custom bootloader build script

echo -e "\e[36mBuilding RustrialOS with Custom Bootloader...\e[0m"

# Step 1: Build bootloader
echo -e "\n\e[33m[1/4] Building custom bootloader...\e[0m"
cd boot && make && cd .. || exit 1

# Step 2: Build kernel
echo -e "\n\e[33m[2/4] Building kernel...\e[0m"
cargo build --target x86_64-rustrial_os.json --release || exit 1

# Step 3: Convert kernel to binary
echo -e "\n\e[33m[3/4] Converting kernel to flat binary...\e[0m"
rust-objcopy target/x86_64-rustrial_os/release/rustrial_os -O binary kernel.bin

# Step 4: Create disk image
echo -e "\n\e[33m[4/4] Creating bootable disk image...\e[0m"
dd if=/dev/zero of=disk.img bs=512 count=20000 2>/dev/null
dd if=boot/bootloader.img of=disk.img conv=notrunc 2>/dev/null
dd if=kernel.bin of=disk.img bs=512 seek=65 conv=notrunc 2>/dev/null

echo -e "\n\e[32mBuild complete! disk.img created.\e[0m"
echo -e "\e[36mRun with: qemu-system-x86_64 -drive format=raw,file=disk.img\e[0m"
```

Make it executable and run:
```bash
chmod +x build-custom.sh
./build-custom.sh
```

### Manual Build Steps (Bootloader Crate)

```bash
# Build the kernel
cargo build --target x86_64-rustrial_os.json

# Create bootable image
cargo bootimage --target x86_64-rustrial_os.json

# Run in QEMU manually
qemu-system-x86_64 -drive format=raw,file=target/x86_64-rustrial_os/debug/bootimage-rustrial_os.bin
```

### Testing

```bash
# Run all integration tests
cargo test

# Run specific test
cargo test --test heap_allocation

# Run with verbose output
cargo test -- --nocapture
```

### QEMU Options

```bash
# Run with serial output to terminal
cargo run -- -serial stdio

# Run with more memory
qemu-system-x86_64 -m 256M -drive format=raw,file=disk.img

# Run with debugging enabled
qemu-system-x86_64 -s -S -drive format=raw,file=disk.img
# Then in another terminal: gdb target/x86_64-rustrial_os/debug/rustrial_os
```

## Key Implementation Details

### Filesystem Architecture

The OS implements a complete virtual filesystem layer:

**RAMfs (In-Memory Filesystem)**:
- Files stored in heap-allocated data structures
- Fast access (no disk I/O)
- Directory support with path-based navigation
- Full VFS abstraction for future filesystem implementations

**Script Loading System**:
- Scripts embedded at compile time using `include_bytes!`
- Automatically mounted to `/scripts/` directory at boot
- 6 example scripts available: fibonacci, factorial, collatz, gcd, prime_checker, sum_of_squares
- Can be extended by modifying `src/script_loader.rs`

**VFS API Example**:
```rust
use rustrial_os::fs::{self, FileSystem};

// Get filesystem reference
if let Some(fs) = fs::root_fs() {
    let mut fs = fs.lock();
    
    // Create a file
    fs.create_file("/test.txt", b"Hello, World!")?;
    
    // Read the file
    let content = fs.read_file_to_string("/test.txt")?;
    
    // List directory
    let files = fs.list_dir("/scripts")?;
}
```

See `FILESYSTEM.md` for detailed architecture documentation.

### Memory Management
The kernel implements a flexible allocator system:
- **Bump Allocator**: Simple, fast allocation but no deallocation
- **Linked List Allocator**: Maintains free list of deallocated blocks
- **Fixed-Size Block Allocator** (Current): Pre-allocated block sizes for fast allocation with reduced fragmentation

### Interrupt Handling
- **Timer Interrupt**: Triggered by hardware timer for periodic tasks
- **Keyboard Interrupt**: Reads PS/2 scancodes and queues them for processing
- **Page Fault Handler**: Displays fault address and error code before halting
- **Double Fault Handler**: Dedicated stack prevents triple faults

### Async Tasks
The kernel supports async/await using a custom executor:
- Tasks are represented as pinned futures
- Executor polls tasks via `Context` and `Waker`
- Keyboard input is processed asynchronously as individual scancodes arrive

### VGA Buffer
- Direct memory access to VGA text buffer at `0xb8000`
- Implements `Write` trait for `print!` and `println!` macros
- Supports color attributes (foreground/background)

### RustrialScript Interpreter
- Stack-based VM with fixed-size stack (256 values)
- Integer, boolean, and nil types
- Simple syntax: variable assignment, arithmetic, control flow (if/while)
- Example scripts demonstrate Fibonacci, factorials, GCD, prime checking, etc.
- No_std, alloc-only: runs in bare-metal OS with no dependencies
- Direct integration with VGA buffer for output
- Modular design: separate lexer, parser, VM, and value modules
- Bytecode compilation for efficient execution
- See `src/rustrial_script/docs/ARCHITECTURE.md` for implementation details
- See `src/rustrial_script/docs/scriptdocs.md` for language reference

### Interactive Menu System
- Async keyboard-driven navigation
- Nested menu structure for better organization
- Visual feedback with selection indicators
- Smart state management for menu transitions
- Script browser with real-time file listing from RAMfs
- Keyboard controls:
  - Number keys: Select menu options
  - Arrow keys (↑/↓) or W/S: Navigate in script browser
  - Enter: Execute selected item
  - ESC: Return to previous menu/screen
  - Backspace: Delete character (in echo mode)
- See `src/rustrial_script/docs/INTERACTIVE_MENU.md` for user experience details

### Custom Bootloader (Optional)
- Two-stage boot process written in x86-64 assembly
- Stage 1: 512-byte boot sector loaded by BIOS
- Stage 2: ~8KB bootloader handling Long Mode transition
- Features:
  - A20 line enable for >1MB memory access
  - CPUID checks for 64-bit CPU support
  - 4-level paging setup with identity mapping
  - GDT configuration for Protected/Long Mode
  - Real Mode → Protected Mode → Long Mode transition
- Educational resource for learning boot process
- See `boot/custombootloader.md` for complete documentation
- Production use: stick with the stable `bootloader` crate

## Dependencies

Key external crates:
- `bootloader`: Handles x86-64 bare-metal boot process
- `x86_64`: x86-64 specific CPU instructions and structures
- `volatile`: Memory access utilities to prevent compiler optimizations
- `uart_16550`: UART serial communication
- `pc-keyboard`: PS/2 keyboard decoding
- `spin`: Spinlock for synchronization
- `lazy_static`: Static initialization for IDT and PICS
- `pic8259`: Programmable Interrupt Controller
- `crossbeam-queue`: Lock-free concurrent queue (unused features available)
- `futures-util`: Futures primitives for async


## Configuration

### Custom Target Specification

The OS uses a custom target (`x86_64-rustrial_os.json`) with:
- **Architecture**: x86_64
- **OS**: none (bare metal)
- **Linker**: rust-lld
- **Features**: Software floating-point, red-zone disabled
- **Panic strategy**: Abort

### Build Configuration

Configured in `.cargo/config.toml`:
- Custom runner using `bootimage`
- Build-std for `core`, `compiler_builtins`, and `alloc`
- Panic-abort for tests

## Development

### Adding New Features

1. **New Interrupt Handler**: Add to `interrupts.rs` IDT initialization
2. **Memory Mapping**: Use the `Mapper` trait in `memory.rs`
3. **Device Drivers**: Follow the pattern in `serial.rs` or `vga_buffer.rs`
4. **Tests**: Create integration tests in `tests/` directory

### Debugging

```bash
# Enable serial output for debugging
cargo run -- -serial stdio

# Run with GDB
qemu-system-x86_64 -s -S -drive format=raw,file=target/.../bootimage-rustrial_os.bin
# In another terminal: gdb target/.../rustrial_os
```

### Known Limitations
- Single-core only (no SMP support yet)
- Filesystem is in-memory only (no persistence across reboots)
- No block device drivers (ATA/AHCI not implemented)
- Limited device driver support (VGA text mode, PS/2 keyboard, serial only)
- No user-space programs or process isolation
- No pre-emptive multitasking (cooperative async/await only)
- Limited memory: 100KB heap by default (configurable)
- VGA graphics mode experimental (320x200 only, may not work on all hardware)
- No network support
- Scripts are read-only (can't edit at runtime)
- Minimal error handling in some areas

### Potential Improvements
- Multi-core CPU support with proper synchronization
- Pre-emptive task scheduling with time slicing
- Larger heap with on-demand paging
- Block device drivers (ATA PIO/DMA, AHCI)
- Persistent filesystem (FAT32 or custom FS)
- File caching and buffering layer
- Network stack (basic TCP/IP)
- Script editor and REPL interface
- Enhanced error types and Result-based error handling
- Dynamic task creation and management
- ACPI support for power management
- USB driver framework
- Higher resolution graphics modes (VESA/VBE support)
- Font rendering and bitmap image support
- Window management system

## Roadmap

### Planned Features

- [x] ~~Basic filesystem support (RAMfs)~~ ✅ **Completed!**
- [x] ~~Minimal scripting interpreter (RustrialScript)~~ ✅ **Completed!**
- [x] ~~Interactive boot menu system~~ ✅ **Completed!**
- [x] ~~Text-mode graphics enhancements~~ ✅ **Completed!**
- [ ] Block device abstraction layer
- [ ] FAT32 filesystem implementation
- [ ] Read files from bootloader disk
- [ ] Multitasking and process scheduler
- [ ] User-space programs
- [ ] System calls interface
- [ ] Network stack (basic TCP/IP)
- [ ] ACPI support
- [ ] USB driver framework
- [ ] Higher resolution graphics (VESA/VBE)

## Contributing

Contributions are welcome! Whether you're:
- Fixing bugs
- Adding new features
- Improving documentation
- Writing tests

Please feel free to open an issue or submit a pull request.

### Development Guidelines

1. Follow Rust naming conventions and style guidelines
2. Add tests for new functionality
3. Update documentation for API changes
4. Ensure `cargo test` passes before submitting PRs
5. Use meaningful commit messages

## Troubleshooting

### Build Issues

**Error: "target 'x86_64-rustrial_os' not found"**
```bash
# Ensure rust-src is installed
rustup component add rust-src
```

**Error: "bootimage not found"**
```bash
# Install bootimage tool
cargo install bootimage
```

**Error: "NASM not found" (Custom Bootloader)**
- Windows: Download from [nasm.us](https://www.nasm.us/), install, and add to PATH
- Linux: `sudo apt install nasm`
- macOS: `brew install nasm`

### Runtime Issues

**QEMU doesn't start or shows black screen**
- Verify QEMU installation: `qemu-system-x86_64 --version`
- Check disk image exists: `ls -l disk.img` or `dir disk.img`
- Try with verbose output: `cargo run -- -serial stdio`

**Kernel panic or triple fault**
- Check serial output for panic messages
- Verify heap is properly initialized in `main.rs`
- Run tests to isolate issues: `cargo test`

**Script browser shows no files**
- Ensure filesystem is initialized in `main.rs`
- Verify scripts are loaded: check for "[FS]" messages on boot
- Confirm `script_loader.rs` includes all scripts

**Custom bootloader hangs**
- Verify boot.bin is exactly 512 bytes with 0xAA55 signature
- Check stage2.bin is present and assembled correctly
- Ensure kernel.bin is at correct sector (66) on disk
- Test with: `qemu-system-x86_64 -drive format=raw,file=boot/bootloader.img`

### Testing Issues

**Tests fail with "exit code 0x10"**
- This is actually success! Exit code 0x10 is our success code
- Check output for actual test results

**Integration tests timeout**
- Increase timeout in `Cargo.toml`: `test-timeout = 600`
- Run single test: `cargo test --test heap_allocation`

## Resources & Learning

This project is heavily inspired by and follows guidance from:

- **[Writing an OS in Rust](https://os.phil-opp.com/)** by Philipp Oppermann - Comprehensive blog series on OS development
- **[OSDev Wiki](https://wiki.osdev.org/)** - Extensive operating system development resources
- **[The Rust Book](https://doc.rust-lang.org/book/)** - Official Rust programming guide
- **[Rust Embedded Book](https://rust-embedded.github.io/book/)** - Embedded Rust programming

## License


## Documentation

### Main Documentation
- **[FILESYSTEM.md](FILESYSTEM.md)** - Complete filesystem architecture, design decisions, and API reference
- **[boot/custombootloader.md](boot/custombootloader.md)** - Custom x86-64 bootloader documentation and boot process

### RustrialScript Documentation
Located in `src/rustrial_script/docs/`:
- **[INTEGRATION.md](src/rustrial_script/docs/INTEGRATION.md)** - How to integrate and use the interpreter
- **[INTERACTIVE_MENU.md](src/rustrial_script/docs/INTERACTIVE_MENU.md)** - Menu system user guide
- **[ARCHITECTURE.md](src/rustrial_script/docs/ARCHITECTURE.md)** - Interpreter internals and design
- **[GETTING_STARTED.md](src/rustrial_script/docs/GETTING_STARTED.md)** - Quick start guide for RustrialScript
- **[scriptdocs.md](src/rustrial_script/docs/scriptdocs.md)** - Complete language reference and syntax

### Example Scripts
Located in `src/rustrial_script/examples/`:
- `fibonacci.rscript` - Fibonacci sequence generator
- `factorial.rscript` - Factorial calculator
- `collatz.rscript` - Collatz conjecture demonstration
- `gcd.rscript` - Greatest common divisor calculator
- `prime_checker.rscript` - Prime number tester
- `sum_of_squares.rscript` - Sum of squares calculator

This project is provided as-is for educational purposes.

## Acknowledgments

Special thanks to:
- **Philipp Oppermann** for the excellent "Writing an OS in Rust" blog series
- The **Rust Embedded Working Group** for embedded Rust tools
- The **OSDev community** for OS development resources
- All **contributors** to this project

---

## Quick Reference

### Boot Menu Navigation
```
Main Menu:
  [1] Normal Operation     → System info + keyboard echo
  [2] RustrialScript       → Sub-menu for scripts
      [1] Run Demo         → Fibonacci demonstration
      [2] Browse Scripts   → Interactive file browser
  [3] Graphics Demo        → Interactive graphics showcase
  [4] Help                 → Documentation

Script Browser:
  ↑/↓ or W/S  - Navigate scripts
  Enter       - Execute selected script
  ESC         - Return to previous menu
  
Graphics Demo:
  Automatic   - Shows box drawing, progress bars, UI elements
  Any Key     - Return to main menu after demo
```

### Common Commands
```bash
# Quick start
cargo run

# Build with custom bootloader
./build-custom.sh          # Linux/macOS
.\build-custom.ps1         # Windows

# Run tests
cargo test

# Build for release
cargo build --target x86_64-rustrial_os.json --release

# Run with serial output
cargo run -- -serial stdio
```

### File Locations
- Kernel source: `src/`
- Scripts: `src/rustrial_script/examples/`
- Bootloader: `boot/`
- Tests: `tests/`
- Documentation: `*.md` and `src/rustrial_script/docs/`

### Memory Map
```
0x0000_0000 - 0x0000_7BFF   Free/BIOS
0x0000_7C00 - 0x0000_7DFF   Boot sector (Stage 1)
0x0000_7E00 - 0x0000_9FFF   Stage 2 bootloader
0x0000_1000 - 0x0000_4FFF   Page tables
0x0010_0000 - 0x001F_FFFF   Kernel binary
0x4444_4444_0000            Heap start (100KB)
0xB800_0000                 VGA buffer
```

---

<p align="center">
  <a href="https://github.com/lazzerex/rustrial-os/issues">Report Bug</a>
  ·
  <a href="https://github.com/lazzerex/rustrial-os/issues">Request Feature</a>
  ·
  <a href="https://github.com/lazzerex/rustrial-os/pulls">Submit PR</a>
</p>
