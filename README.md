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
- **Serial Port I/O**: UART 16550 serial communication for test output and debugging
- **Keyboard Input**: Async keyboard task for processing PS/2 keyboard scancodes

## Project Structure

```
src/
├── main.rs                 # Kernel entry point and main function
├── lib.rs                  # Kernel library with core initialization and test infrastructure
├── vga_buffer.rs          # VGA text mode buffer and print macros
├── serial.rs              # Serial port (UART) I/O implementation
├── gdt.rs                 # Global Descriptor Table setup
├── interrupts.rs          # Interrupt Descriptor Table and handlers
├── memory.rs              # Paging, page tables, and memory mapping
├── allocator/             # Memory allocator implementations
│   ├── mod.rs             # Allocator interface and Locked wrapper
│   ├── bump.rs            # Simple bump allocator
│   ├── linked_list.rs     # Linked list allocator
│   └── fixed_size_block.rs # Fixed-size block allocator (active by default)
└── task/                  # Async task system
    ├── mod.rs             # Task structure and TaskId
    ├── executor.rs        # Waker-based task executor
    ├── simple_executor.rs # Basic single-threaded executor
    └── keyboard.rs        # Keyboard input processing

tests/
├── basic_boot.rs          # Tests kernel boots without crashing
├── heap_allocation.rs     # Tests heap allocation with Box, Vec, and Rc
├── should_panic.rs        # Tests panic handling
└── stack_overflow.rs      # Tests stack overflow exception handling

x86_64-rustrial_os.json    # Custom build target configuration
Cargo.toml                 # Project manifest with dependencies
```

### Memory Layout

- **VGA Buffer**: `0xb8000` - Text mode framebuffer
- **Heap**: `0x_4444_4444_0000` - `0x_4444_4445_9000` (100 KiB)
- **Serial Port**: `0x3F8` (COM1)
- **IST Stack**: 20 KiB dedicated stack for double fault handler

## Prerequisites

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

### Quick Start

```bash
# Clone the repository
git clone https://github.com/lazzerex/rustrial-os.git
cd rustrial-os

# Build and run in QEMU (all-in-one)
cargo run
```

### Manual Build Steps

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
# Run all tests
cargo test

# Run specific test
cargo test --test heap_allocation

# Run with test output
cargo test -- --nocapture

```

## Key Implementation Details

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
- Single-core only (no SMP support)
- No filesystem implementation
- Limited device driver support
- No user-space programs
- Basic scheduler (none implemented yet)
- Text mode only (no graphics)
- No pre-emptive multitasking (only cooperative via async/await)
- Limited memory: 100KB heap by default
- No file system
- No network support
- Minimal error handling in many areas

### Potential Improvements
- Multi-core CPU support
- Pre-emptive task scheduling
- Larger heap with paging improvements
- File system implementation (FAT32 or ext2)
- Network stack
- Enhanced error types and Result-based error handling
- Dynamic task creation and management

## Roadmap

### Planned Features

- [ ] Multitasking and process scheduler
- [ ] User-space programs
- [ ] System calls interface
- [ ] Filesystem support (FAT32)
- [ ] Network stack (basic TCP/IP)
- [ ] ACPI support
- [ ] USB driver framework
- [ ] Graphics mode support

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

## Resources & Learning

This project is heavily inspired by and follows guidance from:

- **[Writing an OS in Rust](https://os.phil-opp.com/)** by Philipp Oppermann - Comprehensive blog series on OS development
- **[OSDev Wiki](https://wiki.osdev.org/)** - Extensive operating system development resources
- **[The Rust Book](https://doc.rust-lang.org/book/)** - Official Rust programming guide
- **[Rust Embedded Book](https://rust-embedded.github.io/book/)** - Embedded Rust programming

## License

This project is provided as-is for educational purposes.

## Acknowledgments

Special thanks to:
- **Philipp Oppermann** for the excellent "Writing an OS in Rust" blog series
- The **Rust Embedded Working Group** for embedded Rust tools
- The **OSDev community** for OS development resources
- All **contributors** to this project

---



<p align="center">
  <a href="https://github.com/lazzerex/rustrial-os/issues">Report Bug</a>
  ·
  <a href="https://github.com/lazzerex/rustrial-os/issues">Request Feature</a>
  ·
  <a href="https://github.com/lazzerex/rustrial-os/pulls">Submit PR</a>
</p>
