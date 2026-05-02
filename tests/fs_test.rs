#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustrial_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rustrial_os::{allocator, memory, serial_print, serial_println};
use rustrial_os::fs::{RamFs, FileSystem, VfsError};
use x86_64::VirtAddr;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    rustrial_os::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");
    test_main();
    rustrial_os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustrial_os::test_panic_handler(info)
}

#[test_case]
fn test_create_and_read_file() {
    serial_print!("fs::create_and_read_file... ");
    let mut fs = RamFs::new();
    fs.create_file("/hello.txt", b"hello world").unwrap();
    let data = fs.read_file("/hello.txt").unwrap();
    assert_eq!(data, b"hello world");
    serial_println!("[ok]");
}

#[test_case]
fn test_file_exists() {
    serial_print!("fs::file_exists... ");
    let mut fs = RamFs::new();
    assert!(!fs.exists("/foo.txt"));
    fs.create_file("/foo.txt", b"").unwrap();
    assert!(fs.exists("/foo.txt"));
    assert!(fs.is_file("/foo.txt"));
    assert!(!fs.is_dir("/foo.txt"));
    serial_println!("[ok]");
}

#[test_case]
fn test_create_dir() {
    serial_print!("fs::create_dir... ");
    let mut fs = RamFs::new();
    fs.create_dir("/mydir").unwrap();
    assert!(fs.exists("/mydir"));
    assert!(fs.is_dir("/mydir"));
    assert!(!fs.is_file("/mydir"));
    serial_println!("[ok]");
}

#[test_case]
fn test_file_not_found() {
    serial_print!("fs::file_not_found... ");
    let fs = RamFs::new();
    let result = fs.read_file("/nonexistent.txt");
    assert!(matches!(result, Err(VfsError::NotFound)));
    serial_println!("[ok]");
}

#[test_case]
fn test_write_overwrites_content() {
    serial_print!("fs::write_overwrites_content... ");
    let mut fs = RamFs::new();
    fs.create_file("/data.txt", b"original").unwrap();
    fs.write_file("/data.txt", b"updated").unwrap();
    let data = fs.read_file("/data.txt").unwrap();
    assert_eq!(data, b"updated");
    serial_println!("[ok]");
}

#[test_case]
fn test_delete_file() {
    serial_print!("fs::delete_file... ");
    let mut fs = RamFs::new();
    fs.create_file("/temp.txt", b"data").unwrap();
    assert!(fs.exists("/temp.txt"));
    fs.delete("/temp.txt").unwrap();
    assert!(!fs.exists("/temp.txt"));
    serial_println!("[ok]");
}

#[test_case]
fn test_duplicate_file_error() {
    serial_print!("fs::duplicate_file_error... ");
    let mut fs = RamFs::new();
    fs.create_file("/dup.txt", b"first").unwrap();
    let result = fs.create_file("/dup.txt", b"second");
    assert!(matches!(result, Err(VfsError::AlreadyExists)));
    serial_println!("[ok]");
}

#[test_case]
fn test_read_dir_on_file_fails() {
    serial_print!("fs::read_dir_on_file_fails... ");
    let mut fs = RamFs::new();
    fs.create_file("/notadir.txt", b"data").unwrap();
    let result = fs.read_file("/notadir.txt");
    assert!(result.is_ok());
    let dir_result = fs.list_dir("/notadir.txt");
    assert!(dir_result.is_err());
    serial_println!("[ok]");
}
