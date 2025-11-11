/// Graphics and visual enhancements for Rustrial OS
/// Provides both text-mode enhancements and VGA graphics mode support

use crate::vga_buffer::{Color, WRITER};
use core::fmt::Write;
use x86_64::instructions::interrupts;

pub mod text_graphics;
pub mod vga_graphics;
pub mod splash;
pub mod demo;

/// Initialize graphics subsystem
pub fn init() {
    // Currently in text mode - can be extended later
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
        let buffer = unsafe { &mut *(0xb8000 as *mut [[u16; 80]; 25]) };
        
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
