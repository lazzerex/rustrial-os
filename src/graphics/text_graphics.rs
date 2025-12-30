/// Text-mode graphics enhancements using ASCII art and box-drawing characters

use crate::vga_buffer::{Color, ColorCode};
use x86_64::instructions::interrupts;
use volatile::Volatile;

// Pointer to the active VGA text buffer. For Limine builds this gets
// re-pointed to the HHDM-mapped VGA memory at runtime.
#[cfg(feature = "limine")]
static mut TEXT_BUFFER_PTR: *mut [[Volatile<ScreenChar>; 80]; 25] =
    0xFFFF_FFFF_FFFF_F000 as *mut _;

#[cfg(not(feature = "limine"))]
static mut TEXT_BUFFER_PTR: *mut [[Volatile<ScreenChar>; 80]; 25] =
    0xb8000 as *mut _;

#[inline]
fn buffer() -> &'static mut [[Volatile<ScreenChar>; 80]; 25] {
    unsafe { &mut *TEXT_BUFFER_PTR }
}

/// Box drawing characters
pub const BOX_HORIZONTAL: u8 = 0xC4; // ─
pub const BOX_VERTICAL: u8 = 0xB3;   // │
pub const BOX_TOP_LEFT: u8 = 0xDA;   // ┌
pub const BOX_TOP_RIGHT: u8 = 0xBF;  // ┐
pub const BOX_BOTTOM_LEFT: u8 = 0xC0; // └
pub const BOX_BOTTOM_RIGHT: u8 = 0xD9; // ┘
pub const BOX_T_RIGHT: u8 = 0xC3;    // ├
pub const BOX_T_LEFT: u8 = 0xB4;     // ┤
pub const BOX_T_DOWN: u8 = 0xC2;     // ┬
pub const BOX_T_UP: u8 = 0xC1;       // ┴
pub const BOX_CROSS: u8 = 0xC5;      // ┼

/// Double-line box drawing characters
pub const BOX_DBL_HORIZONTAL: u8 = 0xCD; // ═
pub const BOX_DBL_VERTICAL: u8 = 0xBA;   // ║
pub const BOX_DBL_TOP_LEFT: u8 = 0xC9;   // ╔
pub const BOX_DBL_TOP_RIGHT: u8 = 0xBB;  // ╗
pub const BOX_DBL_BOTTOM_LEFT: u8 = 0xC8; // ╚
pub const BOX_DBL_BOTTOM_RIGHT: u8 = 0xBC; // ╝

/// Block characters for progress bars and graphics
pub const BLOCK_FULL: u8 = 0xDB;  // █
pub const BLOCK_DARK: u8 = 0xB2;  // ▒
pub const BLOCK_MEDIUM: u8 = 0xB1; // ░
pub const BLOCK_LIGHT: u8 = 0xB0; // ░

/// Represents a positioned character with color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// Draw a box with single lines
pub fn draw_box(x: usize, y: usize, width: usize, height: usize, fg: Color, bg: Color) {
    if width < 2 || height < 2 || x + width > 80 || y + height > 25 {
        return; // Invalid dimensions
    }

    let buffer = buffer();
    let color = ColorCode::new(fg, bg);

    interrupts::without_interrupts(|| {
        // Top border
        buffer[y][x].write(ScreenChar {
            ascii_character: BOX_TOP_LEFT,
            color_code: color,
        });
        for i in 1..width - 1 {
            buffer[y][x + i].write(ScreenChar {
                ascii_character: BOX_HORIZONTAL,
                color_code: color,
            });
        }
        buffer[y][x + width - 1].write(ScreenChar {
            ascii_character: BOX_TOP_RIGHT,
            color_code: color,
        });

        // Sides
        for row in 1..height - 1 {
            buffer[y + row][x].write(ScreenChar {
                ascii_character: BOX_VERTICAL,
                color_code: color,
            });
            buffer[y + row][x + width - 1].write(ScreenChar {
                ascii_character: BOX_VERTICAL,
                color_code: color,
            });
        }

        // Bottom border
        buffer[y + height - 1][x].write(ScreenChar {
            ascii_character: BOX_BOTTOM_LEFT,
            color_code: color,
        });
        for i in 1..width - 1 {
            buffer[y + height - 1][x + i].write(ScreenChar {
                ascii_character: BOX_HORIZONTAL,
                color_code: color,
            });
        }
        buffer[y + height - 1][x + width - 1].write(ScreenChar {
            ascii_character: BOX_BOTTOM_RIGHT,
            color_code: color,
        });
    });
}

/// Draw a filled box
pub fn draw_filled_box(x: usize, y: usize, width: usize, height: usize, fg: Color, bg: Color) {
    if x + width > 80 || y + height > 25 {
        return;
    }

    let buffer = buffer();
    let color = ColorCode::new(fg, bg);

    interrupts::without_interrupts(|| {
        for row in 0..height {
            for col in 0..width {
                buffer[y + row][x + col].write(ScreenChar {
                    ascii_character: b' ',
                    color_code: color,
                });
            }
        }
    });
}

/// Draw a double-line box
pub fn draw_double_box(x: usize, y: usize, width: usize, height: usize, fg: Color, bg: Color) {
    if width < 2 || height < 2 || x + width > 80 || y + height > 25 {
        return;
    }

    let buffer = buffer();
    let color = ColorCode::new(fg, bg);

    interrupts::without_interrupts(|| {
        // Top border
        buffer[y][x].write(ScreenChar {
            ascii_character: BOX_DBL_TOP_LEFT,
            color_code: color,
        });
        for i in 1..width - 1 {
            buffer[y][x + i].write(ScreenChar {
                ascii_character: BOX_DBL_HORIZONTAL,
                color_code: color,
            });
        }
        buffer[y][x + width - 1].write(ScreenChar {
            ascii_character: BOX_DBL_TOP_RIGHT,
            color_code: color,
        });

        // Sides
        for row in 1..height - 1 {
            buffer[y + row][x].write(ScreenChar {
                ascii_character: BOX_DBL_VERTICAL,
                color_code: color,
            });
            buffer[y + row][x + width - 1].write(ScreenChar {
                ascii_character: BOX_DBL_VERTICAL,
                color_code: color,
            });
        }

        // Bottom border
        buffer[y + height - 1][x].write(ScreenChar {
            ascii_character: BOX_DBL_BOTTOM_LEFT,
            color_code: color,
        });
        for i in 1..width - 1 {
            buffer[y + height - 1][x + i].write(ScreenChar {
                ascii_character: BOX_DBL_HORIZONTAL,
                color_code: color,
            });
        }
        buffer[y + height - 1][x + width - 1].write(ScreenChar {
            ascii_character: BOX_DBL_BOTTOM_RIGHT,
            color_code: color,
        });
    });
}

/// Write text at a specific position
pub fn write_at(x: usize, y: usize, text: &str, fg: Color, bg: Color) {
    if y >= 25 {
        return;
    }

    let buffer = buffer();
    let color = ColorCode::new(fg, bg);

    interrupts::without_interrupts(|| {
        let mut col = x;
        for byte in text.bytes() {
            if col >= 80 {
                break;
            }
            if byte >= 0x20 && byte <= 0x7e {
                buffer[y][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color,
                });
                col += 1;
            }
        }
    });
}

/// For Limine builds, point text graphics at the HHDM-mapped VGA buffer.
#[cfg(feature = "limine")]
pub fn init_with_hhdm(hhdm_offset: u64) {
    interrupts::without_interrupts(|| unsafe {
        TEXT_BUFFER_PTR = (hhdm_offset + 0xb8000) as *mut _;
    });
}

/// Draw a horizontal line
pub fn draw_hline(x: usize, y: usize, length: usize, fg: Color, bg: Color) {
    if y >= 25 || x + length > 80 {
        return;
    }

    let buffer = buffer();
    let color = ColorCode::new(fg, bg);

    interrupts::without_interrupts(|| {
        for i in 0..length {
            buffer[y][x + i].write(ScreenChar {
                ascii_character: BOX_HORIZONTAL,
                color_code: color,
            });
        }
    });
}

/// Draw a progress bar with percentage
pub fn draw_progress_bar(x: usize, y: usize, width: usize, progress: usize, total: usize, fg: Color, bg: Color) {
    if y >= 25 || x + width > 80 || total == 0 {
        return;
    }

    let buffer = buffer();
    let color_filled = ColorCode::new(fg, bg);
    let color_empty = ColorCode::new(Color::DarkGray, bg);

    let filled_width = (progress * width) / total;

    interrupts::without_interrupts(|| {
        for i in 0..width {
            let char_to_write = if i < filled_width {
                BLOCK_FULL
            } else {
                BLOCK_LIGHT
            };
            let color = if i < filled_width { color_filled } else { color_empty };
            
            buffer[y][x + i].write(ScreenChar {
                ascii_character: char_to_write,
                color_code: color,
            });
        }
    });

    // Draw percentage
    let percentage = (progress * 100) / total;
    use alloc::format;
    let text = format!(" {}%", percentage);
    write_at(x + width + 1, y, &text, fg, Color::Black);
}
/// Center text on a line
pub fn write_centered(y: usize, text: &str, fg: Color, bg: Color) {
    if y >= 25 {
        return;
    }
    
    let text_len = text.len();
    if text_len >= 80 {
        write_at(0, y, text, fg, bg);
    } else {
        let x = (80 - text_len) / 2;
        write_at(x, y, text, fg, bg);
    }
}

/// Draw a shadow effect box
pub fn draw_shadow_box(x: usize, y: usize, width: usize, height: usize, fg: Color, bg: Color) {
    // Draw the main box
    draw_double_box(x, y, width, height, fg, bg);
    
    // Draw shadow (if space available)
    if x + width + 1 < 80 && y + height < 25 {
        let shadow_color = ColorCode::new(Color::DarkGray, Color::Black);
        let buffer = buffer();
        
        interrupts::without_interrupts(|| {
            // Right shadow
            for row in 1..=height {
                if y + row < 25 {
                    buffer[y + row][x + width].write(ScreenChar {
                        ascii_character: BLOCK_DARK,
                        color_code: shadow_color,
                    });
                }
            }
            
            // Bottom shadow
            for col in 1..=width {
                if x + col < 80 {
                    buffer[y + height][x + col].write(ScreenChar {
                        ascii_character: BLOCK_DARK,
                        color_code: shadow_color,
                    });
                }
            }
        });
    }
}
