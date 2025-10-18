# Rustrial OS

A small, in-development educational operating system written in Rust.

This repository is a work-in-progress learning project experimenting with bare-metal programming on x86_64 using Rust. It follows many of the techniques and guidance from Philipp Oppermann's excellent "Writing an OS in Rust" blog series (see Credit section below).

## Status

This project is currently under active development. Expect incomplete features, breaking changes, and manual build steps. The codebase is not yet suitable for production use.

## Repository layout / files of interest

- `Cargo.toml` - crate configuration and dependencies (uses `bootloader`, `volatile`, `spin`, etc.)
- `x86_64-rustrial_os.json` - custom target specification (x86_64, no OS, rust-lld as linker).
- `src/main.rs` - kernel entry point.
- `src/vga_buffer.rs` - VGA text buffer driver used for printing to the screen during early boot.

## Prerequisites

- Rust toolchain (stable or nightly as needed by some tooling). If you run into build errors about unstable features, try installing a recent nightly toolchain and re-running the commands:

```powershell
rustup install nightly
rustup override set nightly
```
- For Linux, it's preferably to use the `Rustup package`
- `rust-lld` / `lld` (the linker used by the custom target). On Windows this generally comes with the LLVM toolchain or via the `lld` package for your toolchain.
- QEMU for running the generated disk/boot image:

  - On Windows, download QEMU for Windows or use your package manager.

- (Optional) `bootimage` — convenient helper for creating a bootable disk image. Install with:

```powershell
cargo install bootimage
```
- You also need the `llvm-tools-preview` rustup component installed for running `bootimage` and building the bootloader. Run this command to have it installed: `rustup component add llvm-tools-preview`.

## Build

Build the kernel using the provided custom target JSON. From the repository root run:

```powershell
cargo build --target x86_64-rustrial_os.json
```

If you have `bootimage` installed you can build and produce a bootable image in one step:

```powershell
cargo bootimage --target x86_64-rustrial_os.json
```

Note: `bootimage` will place the built image under `target/<target-triple>/<debug|release>/bootimage-<crate>.bin` (or `.../bootimage-<crate>.img` depending on version). Adjust paths below accordingly.

## Run (QEMU)

Once you have a bootable image (either manually created or via `bootimage`), run it in QEMU:

```powershell
qemu-system-x86_64 -drive format=raw,file=target/x86_64-rustrial_os/debug/bootimage-rustrial_os.bin 

# or just simply use this command. As configured in the config.toml file, this will run cargo build command for building the kernel, create a bootable image and automatically start QEMU
cargo run
```

This runs the OS in QEMU and connects the serial port to the terminal (`-serial stdio`) so early printk output appears in your console.

## Notes and troubleshooting

- If you see linker errors, ensure `rust-lld` is available and that your Rust toolchain and linker are compatible. The custom target requests `rust-lld` as the linker.
- If `bootimage` fails, try building manually with `cargo build --target ...` and inspect `target/<target>/debug` for artifacts. The bootloader/bootimage versions matter; if needed, pin versions in `Cargo.toml` or consult upstream docs.

## Contributing

Contributions, improvements, and bug fixes are welcome. This project is educational and intentionally small; feel free to open issues or PRs for suggestions, fixes, or enhancements.

## Credit

Much of the approach and many of the techniques used in this repository are based on Philipp Oppermann's "Writing an OS in Rust" blog series and repository. See his guide for a thorough, step-by-step explanation of building a small OS in Rust:

[https://os.phil-opp.com/](https://os.phil-opp.com/)

---

