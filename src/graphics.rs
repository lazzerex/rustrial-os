/// Graphics and visual enhancements for Rustrial OS
/// Provides both text-mode enhancements and VGA graphics mode support

use crate::vga_buffer::{Color, WRITER};
use core::fmt::Write;
use x86_64::instructions::interrupts;

// Raw VGA buffer pointer for helper routines that bypass WRITER.
// For Limine builds this is retargeted to the HHDM-mapped address at runtime.
#[cfg(feature = "limine")]
static mut TEXT_BUFFER_PTR: *mut [[u16; 80]; 25] = 0xFFFF_FFFF_FFFF_F000 as *mut _;

#[cfg(not(feature = "limine"))]
static mut TEXT_BUFFER_PTR: *mut [[u16; 80]; 25] = 0xb8000 as *mut _;

#[inline]
fn raw_buffer() -> &'static mut [[u16; 80]; 25] {
    unsafe { &mut *TEXT_BUFFER_PTR }
}

pub mod text_graphics;
pub mod vga_graphics;
pub mod splash;
pub mod demo;

/// Initialize graphics subsystem
pub fn init() {
    // Currently in text mode - can be extended later
}

/// Point raw VGA helpers at the HHDM-mapped buffer for Limine boots.
#[cfg(feature = "limine")]
pub fn init_with_hhdm(hhdm_offset: u64) {
    interrupts::without_interrupts(|| unsafe {
        TEXT_BUFFER_PTR = (hhdm_offset + 0xb8000) as *mut _;
    });
}

/// Draw a simple progress bar in text mode
pub fn draw_progress_bar(progress: usize, total: usize, width: usize) {
    let filled = (progress * width) / total;
    let empty = width - filled;
    
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        write!(writer, "[").unwrap();
        for _ in 0..filled {
            write!(writer, "█").unwrap();
        }
        for _ in 0..empty {
            write!(writer, "░").unwrap();
        }
        write!(writer, "] {}/{}", progress, total).unwrap();
    });
}

/// Clear a specific region of the screen
pub fn clear_region(x: usize, y: usize, width: usize, height: usize) {
    interrupts::without_interrupts(|| {
        let _writer = WRITER.lock();
        let buffer = raw_buffer();
        
        for row in y..(y + height).min(25) {
            for col in x..(x + width).min(80) {
                buffer[row][col] = 0x0720; // Black background, white space
            }
        }
    });
}

/// Set colors for subsequent text output
pub fn set_colors(fg: Color, bg: Color) {
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writer.set_color(fg, bg);
    });
}
