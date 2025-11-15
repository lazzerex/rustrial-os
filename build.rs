use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let src_dir = PathBuf::from("src/native");

    println!("cargo:rerun-if-changed=src/native");

    // ----------------------------
    // 1. Compile ASM using NASM
    // ----------------------------
    let asm_file = src_dir.join("cpu_asm.asm");
    let asm_obj = out_dir.join("cpu_asm.o");

    let nasm = Command::new("nasm")
        .args([
            "-f", "elf64",            // FIXED (was win64)
            "-o", asm_obj.to_str().unwrap(),
            asm_file.to_str().unwrap(),
        ])
        .status();

    if !nasm.as_ref().map(|s| s.success()).unwrap_or(false) {
        panic!("NASM failed to compile cpu_asm.asm");
    }

    // ----------------------------
    // 2. Compile C files with clang
    // ----------------------------
    let compiler = "clang";

    let c_files = ["pci.c", "rtc.c"];

    for file in c_files {
        let src = src_dir.join(file);
        let obj = out_dir.join(file.replace(".c", ".o"));

        let status = Command::new(compiler)
            .args([
                "-c",
                "-ffreestanding",
                "-fno-stack-protector",
                "-fno-pie",
                "-mno-red-zone",
                "-target", "x86_64-unknown-none",
                "-I", src_dir.join("include").to_str().unwrap(),
                "-o", obj.to_str().unwrap(),
                src.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to run clang");

        if !status.success() {
            panic!("Clang failed to compile {}", file);
        }
    }

    // ----------------------------
    // 3. Archive into libnative.a
    // ----------------------------
    let mut ar = Command::new("ar");
    ar.args(["rcs", out_dir.join("libnative.a").to_str().unwrap()]);

    ar.arg(asm_obj.to_str().unwrap());
    for file in c_files {
        ar.arg(out_dir.join(file.replace(".c", ".o")).to_str().unwrap());
    }

    let ar_status = ar.status().expect("Failed to run ar");
    if !ar_status.success() {
        panic!("Failed to create libnative.a");
    }

    // ----------------------------
    // 4. Link library
    // ----------------------------
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=native");
}
