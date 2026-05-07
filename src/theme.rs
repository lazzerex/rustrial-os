use core::sync::atomic::{AtomicU8, Ordering};
use crate::vga_buffer::Color;

pub static THEME_ID: AtomicU8 = AtomicU8::new(0);

pub fn get() -> u8 {
    THEME_ID.load(Ordering::Relaxed)
}

pub fn set(t: u8) {
    THEME_ID.store(t.min(4), Ordering::Relaxed);
}

pub fn desktop_bg() -> Color {
    match get() {
        1 => Color::Black,
        2 => Color::Blue,
        3 => Color::Green,
        4 => Color::Magenta,
        _ => Color::Cyan,
    }
}

pub fn title_bar_bg() -> Color {
    match get() {
        1 => Color::DarkGray,
        2 => Color::Blue,
        3 => Color::DarkGray,
        4 => Color::Magenta,
        _ => Color::Blue,
    }
}

pub fn taskbar_bg() -> Color {
    match get() {
        1 => Color::Black,
        2 => Color::Black,
        3 => Color::DarkGray,
        4 => Color::DarkGray,
        _ => Color::DarkGray,
    }
}

pub fn taskbar_fg() -> Color {
    match get() {
        3 => Color::LightGreen,
        4 => Color::Pink,
        _ => Color::LightGray,
    }
}
