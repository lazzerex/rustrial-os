#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustrial_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use rustrial_os::println;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello From the Rustrial Kernel{}", "!");

    rustrial_os::init();

    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // }

    // fn stack_overflow() {
    //     stack_overflow();
    // }

    // stack_overflow();
    
    //breakpoint exception for debugging
    //x86_64::instructions::interrupts::int3();

    //the instruction pointer might be different on other systems so adjust accordingly when debugging
    let ptr = 0x205014 as *mut u8;
    unsafe { *ptr = 42; }

    unsafe { let x = *ptr; }
    println!("read worked");

    unsafe { *ptr = 42; }
    println!("write worked");

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    rustrial_os::hlt_loop();
}


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rustrial_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustrial_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}