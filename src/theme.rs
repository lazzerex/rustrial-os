use core::sync::atomic::{AtomicU8, Ordering};
use crate::vga_buffer::Color;

pub static THEME_ID: AtomicU8 = AtomicU8::new(0);

pub fn get() -> u8 {
    THEME_ID.load(Ordering::Relaxed)
}

pub fn set(t: u8) {
    THEME_ID.store(t.min(2), Ordering::Relaxed);
}

pub fn desktop_bg() -> Color {
    match get() {
        1 => Color::Black,
        2 => Color::Blue,
        _ => Color::Cyan,
    }
}
