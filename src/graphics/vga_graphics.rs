/// VGA 320x200x256 Graphics Mode Support
/// Provides pixel-level graphics capabilities

use core::ptr;
use x86_64::instructions::port::Port;

const VGA_WIDTH: usize = 320;
const VGA_HEIGHT: usize = 200;
const VGA_MEMORY: usize = 0xA0000;

/// VGA Graphics Controller
pub struct VgaGraphics {
    framebuffer: *mut u8,
    width: usize,
    height: usize,
}

impl VgaGraphics {
    /// Create a new VGA graphics interface (doesn't switch modes yet)
    pub const fn new() -> Self {
        Self {
            framebuffer: VGA_MEMORY as *mut u8,
            width: VGA_WIDTH,
            height: VGA_HEIGHT,
        }
    }

    /// Switch to VGA mode 13h (320x200x256)
    pub unsafe fn enter_mode_13h(&mut self) {
        // Set VGA mode 13h using BIOS interrupt (requires real mode or v86 mode)
        // This is a simplified version - full implementation would need more setup
        
        // VGA register setup for mode 13h
        unsafe {
            self.write_registers();
        }
    }

    /// Write VGA registers for mode 13h
    unsafe fn write_registers(&self) {
        unsafe {
            // Miscellaneous Output Register
            let mut misc_port = Port::<u8>::new(0x3C2);
            misc_port.write(0x63);

            // Sequencer Registers
            self.write_sequencer();

            // CRT Controller Registers
            self.write_crtc();

            // Graphics Controller Registers
            self.write_graphics_controller();

            // Attribute Controller Registers
            self.write_attribute_controller();

            // Color Palette (256 colors)
            self.set_default_palette();
        }
    }

    unsafe fn write_sequencer(&self) {
        unsafe {
            let mut addr_port = Port::<u8>::new(0x3C4);
            let mut data_port = Port::<u8>::new(0x3C5);

            // Sequencer registers for mode 13h
            let sequencer_regs = [
                0x03, // Reset
                0x01, // Clocking Mode
                0x0F, // Map Mask
                0x00, // Character Map Select
                0x0E, // Memory Mode
            ];

            for (i, &reg) in sequencer_regs.iter().enumerate() {
                addr_port.write(i as u8);
                data_port.write(reg);
            }
        }
    }

    unsafe fn write_crtc(&self) {
        unsafe {
            let mut addr_port = Port::<u8>::new(0x3D4);
            let mut data_port = Port::<u8>::new(0x3D5);

            // CRTC registers for mode 13h
            let crtc_regs = [
                0x5F, 0x4F, 0x50, 0x82, 0x54, 0x80, 0xBF, 0x1F,
                0x00, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x9C, 0x0E, 0x8F, 0x28, 0x40, 0x96, 0xB9, 0xA3,
                0xFF,
            ];

            for (i, &reg) in crtc_regs.iter().enumerate() {
                addr_port.write(i as u8);
                data_port.write(reg);
            }
        }
    }

    unsafe fn write_graphics_controller(&self) {
        unsafe {
            let mut addr_port = Port::<u8>::new(0x3CE);
            let mut data_port = Port::<u8>::new(0x3CF);

            let graphics_regs = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x05, 0x0F, 0xFF,
            ];

            for (i, &reg) in graphics_regs.iter().enumerate() {
                addr_port.write(i as u8);
                data_port.write(reg);
            }
        }
    }

    unsafe fn write_attribute_controller(&self) {
        unsafe {
            let mut addr_port = Port::<u8>::new(0x3C0);
            let mut read_port = Port::<u8>::new(0x3DA);

            // Reset attribute controller
            read_port.read();

            let attribute_regs = [
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
                0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
                0x41, 0x00, 0x0F, 0x00, 0x00,
            ];

            for (i, &reg) in attribute_regs.iter().enumerate() {
                read_port.read();
                addr_port.write(i as u8);
                addr_port.write(reg);
            }

            // Enable display
            read_port.read();
            addr_port.write(0x20);
        }
    }

    /// Set default 256-color palette
    unsafe fn set_default_palette(&self) {
        unsafe {
            let mut mask_port = Port::<u8>::new(0x3C6);
            let mut addr_port = Port::<u8>::new(0x3C8);
            let mut data_port = Port::<u8>::new(0x3C9);

            mask_port.write(0xFF);
            addr_port.write(0);

            // Write 256 colors (6-bit RGB values)
            for i in 0..=255u16 {
                // Simple gradient palette
                let r = ((i >> 5) & 0x07) << 3;
                let g = ((i >> 2) & 0x07) << 3;
                let b = (i & 0x03) << 4;

                data_port.write(r as u8);
                data_port.write(g as u8);
                data_port.write(b as u8);
            }
        }
    }

    /// Set a specific palette entry
    pub unsafe fn set_palette_entry(&self, index: u8, r: u8, g: u8, b: u8) {
        unsafe {
            let mut addr_port = Port::<u8>::new(0x3C8);
            let mut data_port = Port::<u8>::new(0x3C9);

            addr_port.write(index);
            data_port.write(r >> 2); // 6-bit color
            data_port.write(g >> 2);
            data_port.write(b >> 2);
        }
    }

    /// Set a pixel at (x, y) with color
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        if x < self.width && y < self.height {
            let offset = y * self.width + x;
            unsafe {
                ptr::write_volatile(self.framebuffer.add(offset), color);
            }
        }
    }

    /// Get pixel color at (x, y)
    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        if x < self.width && y < self.height {
            let offset = y * self.width + x;
            unsafe { ptr::read_volatile(self.framebuffer.add(offset)) }
        } else {
            0
        }
    }

    /// Clear screen with a color
    pub fn clear(&mut self, color: u8) {
        for i in 0..(self.width * self.height) {
            unsafe {
                ptr::write_volatile(self.framebuffer.add(i), color);
            }
        }
    }

    /// Draw a horizontal line
    pub fn draw_hline(&mut self, x: usize, y: usize, length: usize, color: u8) {
        for i in 0..length {
            self.set_pixel(x + i, y, color);
        }
    }

    /// Draw a vertical line
    pub fn draw_vline(&mut self, x: usize, y: usize, length: usize, color: u8) {
        for i in 0..length {
            self.set_pixel(x, y + i, color);
        }
    }

    /// Draw a rectangle outline
    pub fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: u8) {
        self.draw_hline(x, y, width, color);
        self.draw_hline(x, y + height - 1, width, color);
        self.draw_vline(x, y, height, color);
        self.draw_vline(x + width - 1, y, height, color);
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: u8) {
        for dy in 0..height {
            self.draw_hline(x, y + dy, width, color);
        }
    }

    /// Draw a circle (Bresenham's algorithm)
    pub fn draw_circle(&mut self, cx: usize, cy: usize, radius: usize, color: u8) {
        let mut x = 0;
        let mut y = radius as isize;
        let mut d = 3 - 2 * radius as isize;

        while x <= y as usize {
            self.plot_circle_points(cx, cy, x, y as usize, color);
            if d < 0 {
                d = d + 4 * x as isize + 6;
            } else {
                d = d + 4 * (x as isize - y) + 10;
                y -= 1;
            }
            x += 1;
        }
    }

    fn plot_circle_points(&mut self, cx: usize, cy: usize, x: usize, y: usize, color: u8) {
        if cx >= x {
            self.set_pixel(cx + x, cy + y, color);
            self.set_pixel(cx - x, cy + y, color);
            self.set_pixel(cx + x, cy.wrapping_sub(y), color);
            self.set_pixel(cx - x, cy.wrapping_sub(y), color);
        }
        if cx >= y {
            self.set_pixel(cx + y, cy + x, color);
            self.set_pixel(cx - y, cy + x, color);
            self.set_pixel(cx + y, cy.wrapping_sub(x), color);
            self.set_pixel(cx - y, cy.wrapping_sub(x), color);
        }
    }

    /// Draw a line (Bresenham's algorithm)
    pub fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u8) {
        let dx = if x1 > x0 { x1 - x0 } else { x0 - x1 };
        let dy = if y1 > y0 { y1 - y0 } else { y0 - y1 };
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = (if dx > dy { dx as isize } else { -(dy as isize) }) / 2;
        let mut x = x0 as isize;
        let mut y = y0 as isize;

        loop {
            self.set_pixel(x as usize, y as usize, color);
            if x == x1 as isize && y == y1 as isize {
                break;
            }
            let e2 = err;
            if e2 > -(dx as isize) {
                err -= dy as isize;
                x += sx;
            }
            if e2 < dy as isize {
                err += dx as isize;
                y += sy;
            }
        }
    }

    /// Return to text mode
    pub unsafe fn return_to_text_mode(&mut self) {
        unsafe {
            // Set VGA mode 3 (text mode)
            let mut misc_port = Port::<u8>::new(0x3C2);
            misc_port.write(0x67);
            
            // Additional register setup would go here
            // This is a simplified version
        }
    }
}

/// Default VGA color palette indices
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum VgaColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}
