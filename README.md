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

## Overview

**Rustrial OS** is an educational x86_64 operating system kernel written entirely in Rust. This project demonstrates low-level systems programming concepts including memory management, interrupt handling, hardware interfacing, and heap allocation—all without relying on the standard library or operating system abstractions.

Built as a learning journey through bare-metal programming, Rustrial OS implements core operating system features from scratch, providing hands-on experience with computer architecture fundamentals.

## Features

### ✅ Implemented

- **🖥️ VGA Text Mode Driver** - Direct VGA buffer manipulation for text output with color support
- **⌨️ Keyboard Input** - PS/2 keyboard driver with scancode translation (US layout)
- **⚡ Interrupt Handling** - Complete IDT setup with hardware interrupt support (timer, keyboard)
- **🛡️ Exception Handling** - CPU exception handlers including:
  - Breakpoint exceptions
  - Double fault handler with separate stack (IST)
  - Page fault handler with detailed error reporting
- **🧮 Memory Management**
  - Physical frame allocator
  - Virtual memory with page table management
  - 4-level page table walking and address translation
  - OffsetPageTable-based mapper
- **📦 Heap Allocation**
  - Dynamic memory allocation using linked-list allocator
  - 100 KiB heap starting at `0x_4444_4444_0000`
  - Support for `Box`, `Vec`, `Rc` and other heap-based types
- **🔧 GDT & TSS** - Global Descriptor Table and Task State Segment configuration
- **⏰ Timer Interrupts** - Periodic timer ticks via PIC (Programmable Interrupt Controller)
- **🔌 Serial Port Output** - UART communication for debugging (COM1)
- **🧪 Testing Framework** - Custom test framework with integration and unit tests

## Architecture

### Project Structure

```
rustrial_os/
├── src/
│   ├── main.rs           # Kernel entry point
│   ├── lib.rs            # Library exports and initialization
│   ├── vga_buffer.rs     # VGA text mode driver
│   ├── serial.rs         # UART serial port driver
│   ├── interrupts.rs     # IDT and interrupt handlers
│   ├── gdt.rs           # Global Descriptor Table setup
│   ├── memory.rs        # Memory management and paging
│   └── allocator.rs     # Heap allocator implementation
├── tests/
│   ├── basic_boot.rs    # Basic boot test
│   ├── heap_allocation.rs  # Heap allocation tests
│   ├── should_panic.rs  # Panic handling test
│   └── stack_overflow.rs   # Stack overflow test
├── Cargo.toml           # Project dependencies
├── config.toml          # Cargo build configuration
└── x86_64-rustrial_os.json  # Custom target specification
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

## Known Limitations

- Single-core only (no SMP support)
- No filesystem implementation
- Limited device driver support
- No user-space programs
- Basic scheduler (none implemented yet)
- Text mode only (no graphics)

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

This project is open source and available under the MIT License.

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