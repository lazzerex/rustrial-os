//! Shutdown button and logic for Rustrial OS desktop

use crate::vga_buffer::Color;
use crate::graphics::text_graphics::{draw_filled_box, draw_box, write_at};

pub struct ShutdownButton {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl ShutdownButton {
    pub fn new(x: usize, y: usize) -> Self {
        ShutdownButton {
            x,
            y,
            width: 14,
            height: 5,
        }
    }

    pub fn contains_point(&self, px: i16, py: i16) -> bool {
        px >= self.x as i16 && px < (self.x + self.width) as i16 &&
        py >= self.y as i16 && py < (self.y + self.height) as i16
    }

    pub fn render(&self, selected: bool) {
        let bg_color = if selected { Color::Red } else { Color::Red };
        let fg_color = Color::White;
        // Draw button background
        draw_filled_box(self.x, self.y, self.width, self.height, fg_color, bg_color);
        // Draw button border
        draw_box(self.x, self.y, self.width, self.height, Color::Yellow, bg_color);
        // Draw shutdown symbol
        write_at(self.x + (self.width / 2) - 2, self.y + 1, "[OFF]", Color::Yellow, bg_color);
        // Draw label
        write_at(self.x + (self.width / 2) - 4, self.y + 3, "Shut Down", Color::White, Color::Red);
    }
}

/// Actually perform shutdown (halt CPU)
pub fn shutdown_system() -> ! {
    use x86_64::instructions::hlt;
    // Optionally print a message
    crate::vga_buffer::clear_screen();
    write_at(20, 12, "System is shutting down...", Color::White, Color::Red);
    // Halt CPU forever
    loop {
        hlt();
    }
}
