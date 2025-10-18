#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

//static HELLO: &[u8] = b"Hello World";

// #[unsafe(no_mangle)]
// pub extern "C" fn _start() -> ! {
//     let vga_buffer = 0xb8000 as *mut u8;

//     for (i, &byte) in HELLO.iter().enumerate() {
//         unsafe {
//             let line_offset: isize = 160 * 2;
//             let char_offset_within_line: isize = i as isize * 2;
//             let color_offset_within_line: isize = i as isize * 2 + 1;
//             let char_offset = char_offset_within_line + line_offset;
//             let color_offset = color_offset_within_line + line_offset;


//             *vga_buffer.offset(char_offset) = byte;
//             *vga_buffer.offset(color_offset) = 0xb;
//         }
//     }

//     loop {}
// }

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello from Rustrial Kernel!").unwrap();
    // write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}", 42, 1.337).unwrap();

    println!("Hello from Rustrial Kernel{}", "!");
    panic!("Crash and burn");

    loop {}
}
