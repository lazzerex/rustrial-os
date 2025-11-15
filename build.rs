// Build script for compiling C and Assembly code
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let src_dir = PathBuf::from("src/native");
    
    // Tell cargo to rerun if C/ASM files change
    println!("cargo:rerun-if-changed=src/native/cpu_asm.asm");
    println!("cargo:rerun-if-changed=src/native/pci.c");
    println!("cargo:rerun-if-changed=src/native/rtc.c");
    
    // Compile Assembly file with NASM
    let asm_file = src_dir.join("cpu_asm.asm");
    let asm_obj = out_dir.join("cpu_asm.o");
    
    let nasm_status = Command::new("nasm")
        .args(&[
            "-f", "win64",  // For Windows, use "elf64" for Linux/Mac
            "-o", asm_obj.to_str().unwrap(),
            asm_file.to_str().unwrap(),
        ])
        .status();
    
    match nasm_status {
        Ok(status) if status.success() => {
            println!("cargo:warning=Successfully compiled assembly");
        }
        _ => {
            println!("cargo:warning=NASM not found or compilation failed - skipping native assembly");
            println!("cargo:warning=Install NASM from https://www.nasm.us/ for native code support");
            return;
        }
    }
    
    // Compile C files with appropriate compiler
    let compiler = if cfg!(windows) { "clang" } else { "gcc" };
    
    for c_file in &["pci.c", "rtc.c"] {
        let src = src_dir.join(c_file);
        let obj = out_dir.join(c_file.replace(".c", ".o"));
        
        let c_status = Command::new(compiler)
            .args(&[
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
            .status();
        
        match c_status {
            Ok(status) if status.success() => {
                println!("cargo:warning=Successfully compiled {}", c_file);
            }
            _ => {
                println!("cargo:warning=C compiler not found or compilation failed for {}", c_file);
                println!("cargo:warning=Install clang or gcc with freestanding support");
                return;
            }
        }
    }
    
    // Create static library
    let lib_path = out_dir.join("libnative.a");
    
    let ar_status = Command::new("ar")
        .args(&[
            "rcs",
            lib_path.to_str().unwrap(),
            out_dir.join("cpu_asm.o").to_str().unwrap(),
            out_dir.join("pci.o").to_str().unwrap(),
            out_dir.join("rtc.o").to_str().unwrap(),
        ])
        .status();
    
    match ar_status {
        Ok(status) if status.success() => {
            println!("cargo:warning=Successfully created native library");
            println!("cargo:rustc-link-search=native={}", out_dir.display());
            println!("cargo:rustc-link-lib=static=native");
        }
        _ => {
            println!("cargo:warning=ar (archiver) not found - cannot create library");
            println!("cargo:warning=Native code will not be available");
        }
    }
}
