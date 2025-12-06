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
    use x86_64::instructions::port::Port;
    
    crate::vga_buffer::clear_screen();
    write_at(15, 10, "Power Off", Color::Yellow, Color::Black);
    write_at(10, 12, "It is now safe to close this window", Color::White, Color::Black);
    
    unsafe {
        // Method 1: QEMU old-style (ISA debug exit device)
        let mut port = Port::new(0xf4);
        port.write(0x10u32);
        
        // Method 2: ACPI shutdown for PIIX4 chipset
        let mut port = Port::new(0x604);
        port.write(0x2000u16);
        
        // Method 3: ACPI shutdown for ICH9/Q35 chipset
        let mut port = Port::new(0x600);
        port.write(0x2000u16);
        
        // Method 4: APM (Advanced Power Management) shutdown
        let mut port = Port::new(0xB004);
        port.write(0x2000u16);
        
        // Method 5: Try writing to ACPI PM1a control block (common location)
        let mut port = Port::new(0x1C00);
        port.write(0x2000u16);
    }
    
    // halt CPU 
    loop {
        hlt();
    }
}
