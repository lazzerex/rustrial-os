#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustrial_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use rustrial_os::println;
use rustrial_os::task::Task;
use rustrial_os::task::executor::Executor;
//use x86_64::structures::paging::PageTable;

entry_point!(kernel_main);

//no more mangle and extern C since we are using the bootloader crate's entry_point macro (which is good)
// #[unsafe(no_mangle)]
// pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    //use rustrial_os::memory::active_level_4_table;
    //use x86_64::VirtAddr;
    //use rustrial_os::memory::translate_addr;
    // use x86_64::{structures::paging::Translate, VirtAddr};
    // use x86_64::{structures::paging::Page, VirtAddr};
    use rustrial_os::allocator;
    use x86_64::VirtAddr;
    use rustrial_os::memory::{self, BootInfoFrameAllocator}; 


    //println!("Hello From the Rustrial Kernel{}", "!");
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
    // let ptr = 0x205014 as *mut u8;
    // unsafe { *ptr = 42; }

    // unsafe { let x = *ptr; }
    // println!("read worked");

    // unsafe { *ptr = 42; }
    // println!("write worked");

    // use x86_64::registers::control::Cr3;

    // let (level_4_page_table, _) = Cr3::read();
    // println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    // for (i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 Entry {}: {:?}", i, entry);

    //         let phys = entry.frame().unwrap().start_address();
    //         let virt = phys.as_u64() + boot_info.physical_memory_offset;
    //         let ptr = VirtAddr::new(virt).as_mut_ptr();
    //         let l3_table: &PageTable = unsafe { &*ptr };

    //         for (i, entry) in l3_table.iter().enumerate() {
    //             if !entry.is_unused() {
    //                 println!("  L3 Entry {}: {:?}", i, entry);
    //             }
    //         }
    //     }
    // }

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // Initialize filesystem and load scripts
    rustrial_os::fs::init();
    rustrial_os::script_loader::load_scripts()
        .expect("failed to load scripts");

    // let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};


    // let addresses = [
    //     0xb8000,
    //     0x201008,
    //     0x0100_0020_1a10,
    //     boot_info.physical_memory_offset,
    // ];

    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);
    //     let phys = mapper.translate_addr(virt);
    //     println!("{:?} -> {:?}", virt, phys);
    // }


    #[cfg(test)]
    test_main();

    // Display boot splash with staged progress before entering the main menu
    rustrial_os::graphics::splash::run_boot_sequence();

    // clear screen and show interactive menu
    use rustrial_os::vga_buffer::clear_screen;
    clear_screen();
    
    println!("Hello From the Rustrial Kernel!");
    
    let mut executor = Executor::new();
    executor.spawn(Task::new(rustrial_os::rustrial_menu::interactive_menu()));
    executor.run();

    println!("It did not crash!");
    //rustrial_os::hlt_loop();
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
