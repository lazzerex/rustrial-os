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

## Star History

<a href="https://www.star-history.com/#lazzerex/rustrial-os&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=lazzerex/rustrial-os&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=lazzerex/rustrial-os&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=lazzerex/rustrial-os&type=date&legend=top-left" />
 </picture>
</a>

## Overview

Rustrial OS is an educational operating system project that demonstrates modern systems programming techniques using Rust. The kernel boots from bare metal on x86-64 architecture and provides core OS functionality including memory management, interrupt handling, task scheduling, hardware detection, PS/2 mouse support and keyboard input processing.

**Quick Links:**
- Shell Documentation: `docs/shell.md`
- **Networking Stack**: `docs/net.md`
- RustrialScript Documentation: `docs/scriptdocs.md`
- Custom Bootloader Guide: `docs/custombootloader.md`
- Graphics API: `docs/graphicsdemo.md`
- Filesystem Architecture: Documented in `src/fs/`
- Native Hardware Detection: `docs/native.md`

## Showcase

**Menu Screen & Filesystem**

<img width="722" height="389" alt="os-1" src="https://github.com/user-attachments/assets/5bf0b6a0-b3e6-4494-85ad-4b16e5f1779d" />

<img width="724" height="400" alt="rustde" src="https://github.com/user-attachments/assets/3e36ad2f-10fa-4e9c-928a-e7e07006ee53" />

<img width="699" height="368" alt="os-2" src="https://github.com/user-attachments/assets/d36d6064-ecb4-41c6-ba16-d1fd1c10af1e" />

<img width="722" height="386" alt="os-3" src="https://github.com/user-attachments/assets/7b7c1e10-fe0e-43a1-a7f9-d67ad33ef67c" />

<img width="609" height="310" alt="os-1-new" src="https://github.com/user-attachments/assets/c7848646-9612-41c9-a692-c7a20f14fb63" />

<img width="555" height="239" alt="os-2-new" src="https://github.com/user-attachments/assets/c24ecfb6-82be-43ca-8d75-b606a86a3d7a" />

<img width="624" height="340" alt="os-2-1-new" src="https://github.com/user-attachments/assets/f61e9d80-7899-49e8-acb6-b22e6ac2be90" />

<img width="656" height="349" alt="os-3-new" src="https://github.com/user-attachments/assets/207436f1-c7bd-49a7-b955-088cb61d304b" />


<img width="723" height="389" alt="os-4" src="https://github.com/user-attachments/assets/40df0234-8f02-4e79-b71e-a3db7b4e83e5" />

<img width="459" height="311" alt="os-5" src="https://github.com/user-attachments/assets/8b5b62d4-ebc0-4bcf-bc76-2639bfd9d39f" />

</details>


<details markdown="1"> <summary>Network Stack in Action</summary>

**TCP/IP Network Stack with RTL8139 Driver**

The OS includes a full networking implementation with Ethernet, ARP, IPv4, and ICMP protocols. Here's a demonstration of the `ifconfig`, `ping`, and `arp` commands working in QEMU with user-mode networking:

<img width="1094" height="506" alt="image" src="https://github.com/user-attachments/assets/5f50768e-1b64-42f0-ad31-2298009f19b9" />


Features shown:
- **Interface Configuration**: `ifconfig` displays MAC address (52:54:00:12:34:56) and IP (10.0.2.15)
- **ICMP Ping**: Successfully pinging QEMU gateway at 10.0.2.2
- **ARP Resolution**: ARP cache showing resolved MAC address for gateway
- **Packet Flow**: Serial debug output showing TX/RX packet processing with RTL8139 driver

For detailed networking documentation, see [docs/net.md](docs/net.md)

</details>


<details markdown="1"> <summary>Running Some Unit Tests</summary>
  
<img width="909" height="144" alt="image" src="https://github.com/user-attachments/assets/ea1a4b66-5351-47c8-9e56-83366e27fb87" />

<img width="906" height="140" alt="image" src="https://github.com/user-attachments/assets/90d5622b-060a-41a2-a25c-d617b8e69903" />

</details>


## Features

### Core Kernel Features
- **x86-64 Bare Metal Architecture**: Boots on real hardware and QEMU using the Bootloader crate
- **Interrupt Handling**: Complete IDT with handlers for breakpoints, double faults, page faults, timer, and keyboard interrupts
- **Memory Management**: 
  - Page table support with virtual-to-physical address translation
  - Heap allocator at `0x_4444_4444_0000` (100 KiB default)
  - Multiple allocator strategies: bump, linked list, and fixed-size block
- **Global Descriptor Table (GDT)**: CPU segmentation with double fault IST support
- **Async/Await Task System**: Custom executor with waker-based scheduling for cooperative multitasking

### Hardware Detection (Native C/Assembly)
- **CPU Information**: Brand string, vendor ID, and feature flags via CPUID instruction
- **Real-Time Clock**: Date/time reading from CMOS RTC chip
- **PCI Device Enumeration**: Detection and listing of PCI devices on the bus
- **FFI Architecture**: Rust bindings to optimized C and x86-64 assembly code
- **Hardware Summary Display**: Integrated into boot menu for system information

### Graphics & UI
- **VGA Text Mode**: Direct memory-mapped buffer access at `0xb8000`
- **Text Graphics Library** (`src/graphics/text_graphics.rs`):
  - Box drawing with single/double lines and shadows
  - Progress bars with animations
  - Message boxes, status bars, and menus
- **Splash Screens**: ASCII art and loading animations
- **Interactive Graphics Demo**: Showcases all visual features (Menu Option 3)
- **VGA Mode 13h** (Experimental): 320x200 pixel graphics framework

### Filesystem
- **RAMfs**: Full-featured in-memory filesystem with file/directory operations
- **VFS Layer**: Virtual filesystem abstraction for future storage backends
- **Script Embedding**: Compile-time inclusion of `.rscript` files into kernel binary
- **Interactive Browser**: Navigate and execute scripts with keyboard controls

### Shell/Command Interpreter
- **Interactive CLI**: Full-featured command-line interface with command parsing
- **File Commands**: `ls`, `cat`, `mkdir`, `touch`, `cd`, `pwd` for filesystem operations
- **Script Execution**: `run` command to execute RustrialScript files
- **Network Commands**: `ifconfig`, `ping`, `arp`, `tcptest` for network diagnostics
- **System Commands**: `rustrialfetch`, `netinfo`, `pciinfo`, `dmastat` for system info
- **Command History**: Navigate previous commands with arrow keys (up to 50 commands)
- **Scrollback Buffer**: Page Up/Down to scroll through command history
- **Customization**: `color` command to change terminal colors, `clear` to reset screen
- **Path Resolution**: Support for absolute paths, relative paths, `.` and `..`
- **Desktop Integration**: Accessible via Shell icon in the desktop environment

### RustrialScript Language
- **Stack-Based VM**: Minimal scripting language with 256-value stack
- **Type System**: Integer, boolean, and nil types
- **Control Flow**: If statements, while loops, and arithmetic operations
- **No-Std Compatible**: Runs in bare-metal environment with `alloc` only
- **VGA Integration**: Direct output to screen during script execution
- **Example Scripts**: Fibonacci, factorial, GCD, prime checker, Collatz conjecture, sum of squares

### Desktop GUI Environment
- **Interactive Desktop**: Graphical environment with mouse-driven interface
- **Icon System**: Launch applications via double-click (Shell, Scripts, Hardware Info, etc.)
- **Smooth Mouse Cursor**: 8x subpixel precision for fluid diagonal movement
- **Visual Feedback**: Icon highlighting and click detection
- **Multi-window Support**: Switch between desktop and applications seamlessly

### Interactive Menu System
The OS boots into a feature-rich menu:
1. **Normal Mode**: System info with hardware detection summary + keyboard echo
2. **RustrialScript**:
   - Run Demo: Fibonacci sequence demonstration
   - Browse Scripts: Interactive file browser with 6 embedded scripts
3. **Graphics Demo**: Comprehensive showcase of text graphics capabilities
4. **Hardware Info**: Detailed CPU, RTC, and PCI device information
5. **Help**: Documentation and usage instructions
6. **Desktop**: Launch graphical desktop environment with mouse support

**Navigation:**
- Number keys: Select menu options
- ↑/↓ or W/S: Navigate script browser
- Enter: Execute selection
- ESC: Return to previous menu
- Mouse: Point and click in desktop mode

### Network Stack 
- **TCP/IP Implementation**: Full protocol stack with Ethernet, ARP, IPv4, ICMP, UDP, TCP
- **RTL8139 Driver**: PCI network card driver with DMA support (256×2KB ring buffers)
- **Async RX/TX Processing**: Waker-based task scheduling for packet handling
- **ARP Protocol**: Address resolution with caching for IPv4→MAC mapping
- **ICMP Echo (Ping)**: Working ping implementation with round-trip time tracking
- **UDP Protocol**: Socket-based API with port registry and ephemeral port allocation
- **TCP Protocol**: Full socket API (connect, listen, accept, send, recv, close)
  - Time-based ISN generation using RTC for secure sequence numbers
  - Sliding window flow control for efficient data transmission
  - Congestion control with AIMD algorithm and fast retransmit
  - 3-way handshake and graceful connection teardown
- **DNS Client**: Full DNS resolver supporting A record queries (RFC 1035 compliant)
- **Domain Name Resolution**: Ping any hostname (e.g., `ping google.com`) with automatic DNS lookup
- **QEMU Networking**: User-mode networking support with hardcoded gateway MAC workaround
- **Shell Commands**: `ifconfig`, `ping <ip|hostname>`, `arp`, `netinfo`, `tcptest` for network management
- **Status**: Fully operational - Successfully resolves DNS and communicates with internet hosts
- **Documentation**: See `docs/net.md` for detailed architecture and setup

### I/O & Serial
- **Serial Port**: UART 16550 (COM1) for debugging and test output
- **Keyboard**: Async PS/2 scancode processing with US layout support
- 
## Architecture

### System Overview

<img width="1065" height="632" alt="image" src="https://github.com/user-attachments/assets/46e40508-430a-435d-8418-5df8b07de51a" />


### Boot Sequence

<img width="1195" height="135" alt="image" src="https://github.com/user-attachments/assets/1b9c7ee2-8225-47c4-9c79-b3a4bfc1ea72" />


### Memory Architecture
#### Virtual memory layout and allocation strategy

<img width="709" height="506" alt="image" src="https://github.com/user-attachments/assets/d5bd1048-9353-4c10-be72-2eabf0d09437" />


### Interrupt System
#### Exception and hardware interrupt handling

<img width="699" height="602" alt="image" src="https://github.com/user-attachments/assets/bb6b3ee3-18b9-4a60-9138-7b6d3121e516" />


### Async Task System
#### Cooperative multitasking with async/await

<img width="772" height="471" alt="image" src="https://github.com/user-attachments/assets/c16ce3b6-059b-4b93-9467-1e34f6f6fda9" />


### Filesystem Architecture
#### Virtual filesystem with in-memory storage

<img width="413" height="652" alt="image" src="https://github.com/user-attachments/assets/b97d6f0d-b9df-4e2a-b094-c2f7af65ead1" />


### Network Stack
#### TCP/IP implementation with RTL8139 driver

<img width="458" height="799" alt="image" src="https://github.com/user-attachments/assets/dbabf908-208d-415c-a644-f8bc30fb99eb" />


### Graphics System
#### VGA text mode and desktop environment

<img width="729" height="535" alt="image" src="https://github.com/user-attachments/assets/9413386b-103e-4a7d-b955-a9595b4fb58a" />


### Hardware Detection
#### Native C/Assembly for performance-critical hardware access

<img width="581" height="521" alt="image" src="https://github.com/user-attachments/assets/8ac4b1b1-04d0-47d1-bb3e-c318f738ed80" />


### RustrialScript Interpreter
#### Stack-based virtual machine for embedded scripting

<img width="953" height="155" alt="image" src="https://github.com/user-attachments/assets/2d7b8f02-aa28-41dd-9718-2e3edc125dca" />

### User Interaction Flow

<img width="1005" height="378" alt="image" src="https://github.com/user-attachments/assets/947ddd33-05bf-4d37-94f2-0905e1818565" />


## Project Structure

```
src/
├── main.rs                  # Kernel entry point and boot sequence
├── lib.rs                   # Core initialization and test infrastructure
├── vga_buffer.rs            # VGA text buffer and print! macros
├── serial.rs                # UART serial I/O for debugging
├── gdt.rs                   # Global Descriptor Table setup
├── interrupts.rs            # IDT and exception handlers
├── memory.rs                # Paging and address translation
├── allocator.rs             # Heap allocator selection
├── rustrial_menu.rs         # Interactive menu with hardware info
├── script_loader.rs         # Compile-time script embedding
├── native_ffi.rs            # FFI bindings to C/Assembly code
├── graphics.rs              # Graphics subsystem interface
├── desktop.rs               # Desktop GUI environment
├── shell.rs                 # Command-line shell interpreter
│
├── allocator/               # Memory allocator implementations
│   ├── bump.rs              # Simple bump allocator
│   ├── linked_list.rs       # Free-list allocator
│   └── fixed_size_block.rs  # Fixed-size block allocator (default)
│
├── fs/                      # Filesystem layer
│   ├── mod.rs               # FS initialization and mounting
│   ├── vfs.rs               # Virtual filesystem abstraction
│   └── ramfs.rs             # In-memory filesystem
│
├── graphics/                # Visual enhancements
│   ├── text_graphics.rs     # Box drawing and UI components
│   ├── vga_graphics.rs      # VGA Mode 13h (experimental)
│   ├── splash.rs            # Boot splash and status bars
│   ├── demo.rs              # Interactive demos
│   └── graphicsdemo.md      # Graphics API reference
│
├── native/                  # Native C and Assembly code
│   ├── cpu_asm.asm          # CPUID and feature detection
│   ├── pci.c                # PCI bus enumeration
│   ├── rtc.c                # Real-time clock access
│   ├── native.md            # Native code documentation
│   └── include/             # C header files
│
├── drivers/                 # Device drivers
│   └── net/                 # Network drivers
│       ├── mod.rs           # Driver abstraction layer
│       └── rtl8139/         # RTL8139 NIC driver
│
├── net/                     # Network stack
│   ├── mod.rs               # Network module exports
│   ├── stack.rs             # Core network coordinator
│   ├── buffer.rs            # Packet buffer management
│   ├── ethernet.rs          # Ethernet frame handling
│   ├── arp.rs               # ARP protocol with caching
│   ├── ipv4.rs              # IPv4 packet processing
│   ├── icmp.rs              # ICMP echo request/reply
│   ├── udp.rs               # UDP socket implementation
│   ├── tcp.rs               # TCP with sliding window & congestion control
│   ├── dns.rs               # DNS client (A records)
│   └── loopback.rs          # Loopback interface (127.0.0.1)
│
├── rustrial_script/         # Scripting language
│   ├── mod.rs               # Interpreter interface
│   ├── lexer.rs             # Tokenizer
│   ├── parser.rs            # Bytecode compiler
│   ├── vm.rs                # Stack-based VM
│   ├── value.rs             # Value types (int/bool/nil)
│   ├── examples/            # Example .rscript files
│   └── docs/                # Language documentation
│
└── task/                    # Async runtime
    ├── mod.rs               # Task structures
    ├── executor.rs          # Waker-based executor
    ├── simple_executor.rs   # Basic executor
    ├── keyboard.rs          # Async keyboard processing
    └── mouse.rs             # Async PS/2 mouse driver

tests/                       # Integration tests
├── basic_boot.rs            # Boot verification
├── heap_allocation.rs       # Allocator tests
├── should_panic.rs          # Panic handling
└── stack_overflow.rs        # Exception tests

boot/                        # Custom bootloader (optional)
├── boot.asm                 # Stage 1 (512 bytes)
├── stage2.asm               # Stage 2 (Long Mode)
├── build.ps1                # Windows build script
├── build.sh                 # Linux/macOS build script
└── custombootloader.md      # Documentation

Cargo.toml                   # Dependencies and configuration
x86_64-rustrial_os.json      # Custom bare-metal target
build.rs                     # Build script (NASM + Clang)
```

### Memory Map
| Address Range | Usage |
|--------------|-------|
| `0xb8000` | VGA text buffer (80x25) |
| `0x3F8` | Serial port (COM1) |
| `0x_4444_4444_0000` - `0x_4444_4445_9000` | Heap (100 KiB) |
| IST Entry 0 | Double fault stack (20 KiB) |

## Prerequisites

### Required Tools

1. **Rustup**
 - On **Linux**:
  ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- On **Windows**: You can download the executable in the rustup.rs website
  
2. **Rust Toolchain** (Nightly)
   ```bash
   rustup install nightly
   rustup override set nightly
   rustup component add rust-src llvm-tools-preview
   ```

3. **Bootimage**
   ```bash
   cargo install bootimage
   ```
   On Linux, if you encounter any errors, you should probably install build-essential first by running the command:
    ```bash
   sudo apt install build-essential
   ```

5. **QEMU x86-64 Emulator**
   - **Linux**: `sudo apt install qemu-system-x86` or `sudo pacman -S qemu-system-x86` (qemu-dekstop if qemu-system-x86 cannot be installed)
   - **macOS**: `brew install qemu`
   - **Windows**: Download from [qemu.org](https://www.qemu.org/download/)

6. **NASM** (for custom bootloader or native code)
   - **Linux**: `sudo apt install nasm` or `sudo pacman -S nasm`
   - **macOS**: `brew install nasm`
   - **Windows**: Download from [nasm.us](https://www.nasm.us/)

7. **Clang** (for native C code compilation)
   - **Linux**: `sudo apt install clang` or `sudo pacman -S clang`
   - **macOS**: Included with Xcode Command Line Tools
   - **Windows**: Install LLVM from [llvm.org](https://llvm.org/)

### System Requirements
- Rust 1.70+ nightly (edition 2024)
- 2 GB RAM minimum
- Internet connection for initial dependency download

## Build & Run

### Quick Start

```bash
# Clone the repository
git clone https://github.com/lazzerex/rustrial-os.git
cd rustrial-os

# Build and run (all-in-one)
cargo run
```

The OS boots into an interactive menu:
1. **Normal Mode**: System info + keyboard echo
2. **RustrialScript**: Run demo or browse embedded scripts
3. **Graphics Demo**: Showcase text-mode graphics
4. **Hardware Info**: CPU, RTC, and PCI device details
5. **Help**: Usage instructions

### Manual Build Commands

```bash
# Build kernel only
cargo build --target x86_64-rustrial_os.json

# Create bootable image
cargo bootimage --target x86_64-rustrial_os.json

# Run with custom QEMU options
qemu-system-x86_64 -drive format=raw,file=target/x86_64-rustrial_os/debug/bootimage-rustrial_os.bin -serial stdio
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test --test heap_allocation

# Verbose output
cargo test -- --nocapture
```

### Custom Bootloader (does not work currently)

A two-stage x86-64 assembly bootloader for educational purposes. Full documentation: `docs/custombootloader.md`

**Quick Build:**

```powershell
# Windows - Use automated script
.\boot\build-custom.ps1
```

```bash
# Linux/macOS - Use automated script  
./boot/build-custom.sh
```

**Manual Steps:**
1. Build bootloader: `cd boot && make` (Linux) or `.\build.ps1` (Windows)
2. Build kernel with `--features custom_bootloader`
3. Convert ELF to binary with `rust-objcopy`
4. Create disk image and write bootloader + kernel at correct sectors

See `docs/custombootloader.md` for detailed instructions and memory layout.

## Technical Details

### Memory Management
- **Allocators**: Choice of bump, linked list, or fixed-size block (default)
- **Paging**: 4-level page tables with virtual-to-physical translation
- **Heap**: 100 KiB at `0x_4444_4444_0000`, expandable via configuration
- **Frame Allocation**: Uses bootloader-provided memory map

### Interrupt System
- **IDT**: Handles exceptions (breakpoint, double fault, page fault) and hardware interrupts (timer, keyboard)
- **PIC 8259**: Programmable Interrupt Controller for IRQ management
- **IST**: Dedicated 20 KiB stack for double fault handler prevents triple faults

### Async Runtime
- **Custom Executor**: Waker-based task polling for cooperative multitasking
- **Keyboard Task**: Async scancode processing with queue-based event handling
- **Future Support**: Full async/await syntax with `no_std` compatibility

### Filesystem
- **RAMfs**: In-memory VFS with file/directory operations
- **Script Loader**: Compile-time embedding via `include_bytes!` macro
- **Mount Point**: Scripts loaded at `/scripts/` during boot
- **Extensible**: VFS abstraction allows future storage backends

### Native Hardware Detection
- **CPU Info**: CPUID instruction via assembly (`src/native/cpu_asm.asm`)
- **PCI Enumeration**: Bus scanning in C (`src/native/pci.c`)
- **RTC Access**: CMOS real-time clock in C (`src/native/rtc.c`)
- **Build Integration**: NASM + Clang via `build.rs` script

## Dependencies

| Crate | Purpose |
|-------|---------|
| `bootloader` | x86-64 boot process and memory mapping |
| `x86_64` | CPU instructions and structures |
| `volatile` | Prevent compiler optimizations on hardware access |
| `uart_16550` | Serial port communication |
| `pc-keyboard` | PS/2 scancode decoding |
| `spin` | Spinlock synchronization primitives |
| `lazy_static` | Static initialization (IDT, PIC) |
| `pic8259` | Programmable Interrupt Controller |
| `linked_list_allocator` | Heap allocator implementation |
| `conquer-once` | One-time initialization |
| `crossbeam-queue` | Lock-free concurrent queue |
| `futures-util` | Async primitives |

## Configuration

**Custom Target** (`x86_64-rustrial_os.json`):
- Bare-metal x86-64 with no OS
- LLD linker, software floating-point
- Red-zone disabled, panic=abort

**Cargo Config** (`.cargo/config.toml`):
- Custom runner via `bootimage`
- Build-std for `core`, `compiler_builtins`, `alloc`

## Development

### Extending the OS

**Add Interrupt Handler:**
```rust
// In interrupts.rs
idt.your_interrupt.set_handler_fn(your_handler);
```

**Add Device Driver:**
Follow patterns in `serial.rs` or `vga_buffer.rs` - use volatile reads/writes for MMIO.

**Add Test:**
Create file in `tests/` with `#![no_std]`, `#![no_main]`, and test functions.

### Debugging

```bash
# Serial output
cargo run -- -serial stdio

# GDB debugging
qemu-system-x86_64 -s -S -drive format=raw,file=disk.img
# In another terminal:
gdb target/x86_64-rustrial_os/debug/rustrial_os
(gdb) target remote :1234
```

### Limitations

- **Single-core**: No SMP support
- **No persistence**: RAMfs only, no disk I/O
- **No user-space**: Kernel-only, no process isolation
- **Cooperative multitasking**: Async/await only, no preemption
- **Limited drivers**: VGA, PS/2 (keyboard + mouse), serial, RTL8139 NIC
- **100 KiB heap**: Configurable but fixed at build time
- **Text mode only**: VGA 80×25 (Mode 13h experimental)

## Roadmap

**Completed:**
- RAMfs in-memory filesystem with VFS abstraction
- RustrialScript interpreter (stack-based VM)
- Interactive menu with hardware detection
- Text-mode graphics library and desktop GUI
- Native C/Assembly hardware detection (CPU, PCI, RTC)
- Network stack with TCP/IP (Ethernet, ARP, IPv4, ICMP, UDP, TCP, DNS)
- PS/2 mouse driver with smooth cursor movement
- Shell with network diagnostics and scrollback

**In Progress / Planned:**
- [ ] HTTP client (TCP ready, needs implementation)
- [ ] DHCP client for dynamic IP configuration
- [ ] Block device drivers (ATA/AHCI)
- [ ] Persistent filesystem (FAT32 or ext2)
- [ ] Pre-emptive multitasking
- [ ] User-space programs with system calls
- [ ] ACPI and power management
- [ ] USB support
- [ ] Higher resolution graphics (VESA/VBE)

## Contributing

Contributions welcome! Open an issue or PR for:
- Bug fixes
- New features (drivers, syscalls, etc.)
- Documentation improvements
- Tests

**Guidelines:**
1. Follow Rust conventions and `cargo fmt`
2. Add tests for new functionality
3. Ensure `cargo test` passes
4. Update relevant documentation
5. Write clear commit messages

## Troubleshooting

**Build Errors:**
- `target not found`: Install rust-src with `rustup component add rust-src`
- `bootimage not found`: Install with `cargo install bootimage`
- `NASM not found`: Install NASM and add to PATH
- `clang not found`: Install Clang/LLVM for native code compilation

**Runtime Issues:**
- **Black screen**: Check QEMU installation, try `cargo run -- -serial stdio`
- **Kernel panic**: Check serial output for error messages
- **Triple fault**: Likely memory or interrupt configuration issue - check GDT/IDT setup

**Custom Bootloader Issues:**
- **Won't boot**: Verify disk image has bootloader at sector 0
- **Hangs after boot**: Ensure `--features custom_bootloader` was used during build
- **Kernel too large**: Increase `KERNEL_SECTORS` in `boot/stage2.asm`
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

---

## Phase 1: Hardware Detection (Multi-Language Implementation)

Rustrial OS now includes **Phase 1** hardware detection using a hybrid approach with **Rust, C, and Assembly**:

### Implementation Approaches

**Dual Implementation Strategy:**
- **Pure Rust** (`src/cpu.rs`, `src/pci.rs`, `src/rtc.rs`) - Safe, idiomatic Rust
- **Native C/ASM** (`src/native/`) - Performance-critical code in Assembly + C

### Phase 1 Modules

#### 1. CPU Feature Detection
- **Rust**: `src/cpu.rs` - CPUID using `core::arch::x86_64`
- **Assembly**: `src/native/cpu_asm.asm` - Direct CPUID instructions (NASM)
- **Features**: SSE, SSE2, SSE3, SSE4, AVX, AVX2, AES-NI, RDRAND, BMI1/2
- **Info**: Vendor ID, brand string, feature flags

```rust
// Rust API
use rustrial_os::cpu;
let info = cpu::CpuInfo::get();
println!("CPU: {}", info.brand_str());
if cpu::has_avx2() { /* use AVX2 */ }
```

#### 2. PCI Device Enumeration
- **Rust**: `src/pci.rs` - Type-safe PCI access
- **C**: `src/native/pci.c` - Direct I/O port operations
- **Features**: Full bus scanning, device info, BAR reading
- **Classes**: Mass storage, network, display, multimedia, etc.

```rust
// Rust API
use rustrial_os::pci;
let devices = pci::enumerate_devices();
for dev in devices {
    println!("{}", dev);
}
```

#### 3. Real-Time Clock (RTC)
- **Rust**: `src/rtc.rs` - Safe CMOS access
- **C**: `src/native/rtc.c` - BCD conversion and time reading
- **Features**: Date/time, BCD/binary modes, 12/24h support

```rust
// Rust API
use rustrial_os::rtc;
let dt = rtc::read_datetime();
println!("Date: {}", dt);
```

#### 4. VESA Framebuffer Framework
- **Rust**: `src/vesa.rs` - Pixel operations and drawing
- **Status**: Framework ready (requires UEFI bootloader)

### Building with Native Code

**Prerequisites:**
- NASM (Netwide Assembler) - https://www.nasm.us/
- Clang or GCC with freestanding support
- ar (archiver tool)

## Graphics API Quick Reference

```rust
use rustrial_os::graphics::text_graphics::*;
use rustrial_os::vga_buffer::Color;

// Box drawing
draw_box(10, 5, 40, 10, Color::Cyan, Color::Black);
draw_shadow_box(15, 8, 50, 8, Color::Yellow, Color::Blue);

// Progress bar
draw_progress_bar(10, 12, 50, 75, 100, Color::Green, Color::Black);

// Status bar
show_status_bar("Rustrial OS v0.1.0");

// See docs/graphicsdemo.md for full API
```

## Hardware Detection

Native hardware detection uses **C + Assembly** for optimal performance:

**Build automatically compiles:**
- `src/native/cpu_asm.asm` - CPUID via NASM
- `src/native/pci.c` - PCI enumeration via Clang
- `src/native/rtc.c` - Real-time clock via Clang

Access via **Menu Option 4** for CPU info, RTC date/time, and PCI device listing.

> **Note**: Native code is experimental. Menu options 1, 2, 3, and 5 are stable.

## Quick Reference

### Menu Navigation
| Key | Action |
|-----|--------|
| **1** | Normal mode (system info + keyboard) |
| **2** | RustrialScript (demo or browser) |
| **3** | Graphics showcase |
| **4** | Hardware info (CPU, RTC, PCI) |
| **5** | Help documentation |
| **↑/↓** or **W/S** | Navigate browser |
| **Enter** | Execute selection |
| **ESC** | Return to previous menu |

### Common Commands
```bash
cargo run                    # Quick start
cargo test                   # Run tests
cargo run -- -serial stdio   # With serial output
./boot/build-custom.ps1      # Build custom bootloader (Windows)
./boot/build-custom.sh       # Build custom bootloader (Linux/macOS)
```

### Key Directories
- `src/` - Kernel source code
- `src/rustrial_script/examples/` - Embedded scripts
- `src/graphics/` - Graphics library
- `src/native/` - C and Assembly code
- `boot/` - Custom bootloader assembly files
- `docs/` - Documentation
- `tests/` - Integration tests

## Resources & Learning

This project is heavily inspired by and follows guidance from:

- **[Writing an OS in Rust](https://os.phil-opp.com/)** by Philipp Oppermann - Comprehensive blog series on OS development
- **[OSDev Wiki](https://wiki.osdev.org/)** - Extensive operating system development resources
- **[The Rust Book](https://doc.rust-lang.org/book/)** - Official Rust programming guide
- **[Rust Embedded Book](https://rust-embedded.github.io/book/)** - Embedded Rust programming

## License

This project is open source and available for educational purposes. See individual file headers for specific licensing information.

---

**Made with Rust** | [Issues](https://github.com/lazzerex/rustrial-os/issues) | [Contributing](https://github.com/lazzerex/rustrial-os/pulls)
