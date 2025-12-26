#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustrial_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[cfg(feature = "custom_bootloader")]
use core::panic::PanicInfo;

#[cfg(not(feature = "custom_bootloader"))]
use bootloader::{BootInfo, entry_point};

#[cfg(not(feature = "custom_bootloader"))]
use core::panic::PanicInfo;

use rustrial_os::println;
use rustrial_os::task::Task;
use rustrial_os::task::executor::Executor;
use rustrial_os::native_ffi;
use rustrial_os::desktop::{run_desktop_environment, IconAction};
//use x86_64::structures::paging::PageTable;

fn print_hardware_summary() {
    let cpu_info = native_ffi::CpuInfo::get();
    let dt = native_ffi::DateTime::read();
    let devices = native_ffi::enumerate_pci_devices();
    
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║              HARDWARE DETECTION SUMMARY (Native C/ASM)            ║");
    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║ CPU:  {:<60} ║", truncate_str(cpu_info.brand_str(), 60));
    println!("║ Date: {:<60} ║", truncate_str(&alloc::format!("{}", dt), 60));
    println!("║ PCI:  {} devices detected{:<46} ║", devices.len(), "");
    println!("╚════════════════════════════════════════════════════════════════════╝");
}

fn truncate_str(s: &str, max_len: usize) -> alloc::string::String {
    if s.len() <= max_len {
        alloc::format!("{:width$}", s, width = max_len)
    } else {
        alloc::format!("{}...", &s[..max_len-3])
    }
}

async fn desktop_loop() {
    loop {
        // Show desktop and wait for user action
        let action = run_desktop_environment().await;
        
        // Handle the selected action
        match action {
            IconAction::OpenMenu => {
                use rustrial_os::vga_buffer::clear_screen;
                clear_screen();
                let return_to_desktop = rustrial_os::rustrial_menu::interactive_menu().await;
                if !return_to_desktop {
                    continue;
                }
            }
            IconAction::SystemInfo => {
                use rustrial_os::vga_buffer::clear_screen;
                clear_screen();
                rustrial_os::rustrial_menu::show_system_info_from_desktop().await;
            }
            IconAction::Scripts => {
                use rustrial_os::vga_buffer::clear_screen;
                clear_screen();
                rustrial_os::rustrial_menu::show_scripts_from_desktop().await;
            }
            IconAction::Hardware => {
                use rustrial_os::vga_buffer::clear_screen;
                clear_screen();
                rustrial_os::rustrial_menu::show_hardware_from_desktop().await;
            }
            IconAction::Shell => {
                use rustrial_os::vga_buffer::clear_screen;
                clear_screen();
                rustrial_os::rustrial_menu::menu_system::launch_shell().await;
            }
            IconAction::Shutdown => {
                rustrial_os::rustrial_menu::menu_system::shutdown::shutdown_system();
            }
        }
    }
}
#[cfg(not(feature = "custom_bootloader"))]
entry_point!(kernel_main);

// Custom bootloader entry point, no BootInfo available
#[cfg(feature = "custom_bootloader")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    kernel_main_custom()
}

#[cfg(not(feature = "custom_bootloader"))]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    //i should really clean these
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

   
    //also clean these
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

    //initialize DMA memory for networking
    rustrial_os::memory::dma::init_dma(&mut mapper, &mut frame_allocator, phys_mem_offset)
        .expect("DMA initialization failed");

    //initialize network driver
    println!("[Network] Initializing network driver...");
    match rustrial_os::drivers::net::rtl8139::init_network() {
        Ok(_) => println!("[Network] Network driver initialized successfully"),
        Err(e) => println!("[Network] Failed to initialize network driver: {}", e),
    }

    // initialize filesystem and load scripts
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

    // Display boot splash with staged progress before entering the desktop
    rustrial_os::graphics::splash::run_boot_sequence();

    // Initialize mouse support
    rustrial_os::task::mouse::init();
    
    // Launch desktop environment with menu integration
    let mut executor = Executor::new();
    
    // Initialize network stack (spawn RX/TX processing tasks)
    println!("[Network] Initializing network stack...");
    rustrial_os::net::stack::init(&mut executor);
    println!("[Network] Network stack initialized");
    
    executor.spawn(Task::new(desktop_loop()));
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

// Custom bootloader kernel entry - no BootInfo, simplified initialization
#[cfg(feature = "custom_bootloader")]
fn kernel_main_custom() -> ! {
    //println!("Hello From the Rustrial Kernel{}", "!");
    rustrial_os::init();

    // Initialize filesystem and load scripts (no heap required for basic operation)
    rustrial_os::fs::init();
    rustrial_os::script_loader::load_scripts()
        .expect("failed to load scripts");

    #[cfg(test)]
    test_main();

    // Display boot splash with staged progress before entering the desktop
    rustrial_os::graphics::splash::run_boot_sequence();

    // Initialize mouse support
    rustrial_os::task::mouse::init();
    
    // Launch desktop environment with menu integration
    let mut executor = Executor::new();
    
    // Initialize network stack (spawn RX/TX processing tasks)
    // Note: Custom bootloader path has no DMA/heap, network may not work
    rustrial_os::net::stack::init(&mut executor);
    
    executor.spawn(Task::new(desktop_loop()));
    executor.run();
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
