#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
extern crate alloc;

pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod allocator;
pub mod serial;
pub mod vga_buffer;
pub mod task;

pub mod rustrial_menu;
pub mod fs;
pub mod script_loader;
pub mod graphics;
pub mod desktop;
pub mod shell;

// hardware detection with native implementation (C + Assembly)
pub mod native_ffi; // FFI bindings to C/Assembly code

pub mod rustrial_script;

//Networking infrastructure
pub mod net;

//Network drivers
pub mod drivers;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    
    // Unmask IRQ 12 (mouse) on the secondary PIC
    // The secondary PIC's IMR is at port 0xA1
    // IRQ 12 is bit 4 on the secondary PIC (IRQ 12 = 8 + 4)
    // We also need to unmask IRQ 2 on primary PIC (cascade to secondary)
    unsafe {
        use x86_64::instructions::port::Port;
        
        // Read current masks
        let mut pic1_data: Port<u8> = Port::new(0x21); // Primary PIC data/IMR
        let mut pic2_data: Port<u8> = Port::new(0xA1); // Secondary PIC data/IMR
        
        // Unmask IRQ 2 on primary (cascade) - bit 2
        let mask1: u8 = pic1_data.read();
        pic1_data.write(mask1 & !0x04);
        
        // Unmask IRQ 12 on secondary (mouse) - bit 4
        let mask2: u8 = pic2_data.read();
        pic2_data.write(mask2 & !0x10);
        
        serial_println!("[PIC] Unmasked IRQ2 (cascade) and IRQ12 (mouse)");
    }
    
    x86_64::instructions::interrupts::enable();
}
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);


#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}