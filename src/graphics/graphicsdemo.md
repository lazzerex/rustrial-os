# Graphics Module for Rustrial OS

Visual enhancements for Rustrial OS including text-mode graphics and VGA pixel graphics.

## Features

**Text-Mode Graphics** (80x25 VGA)
- Box drawing (single/double lines, shadows) using IBM PC box-drawing characters
- Progress bars with percentage display and animations
- UI components (message boxes, status bars, terminal windows, menus)
- Splash screens with ASCII art
- 16-color VGA palette support

**VGA Graphics Mode** (Experimental)
- Mode 13h: 320x200 pixels, 256 colors
- Pixel drawing, lines, rectangles, circles
- Custom palette support (6-bit RGB)
- Direct framebuffer access at 0xA0000

## Files

- `mod.rs` - Module interface
- `text_graphics.rs` - Box drawing, progress bars, UI components
- `vga_graphics.rs` - VGA Mode 13h pixel graphics (experimental)
- `splash.rs` - Splash screens and fancy UI elements
- `demo.rs` - Interactive graphics demonstrations

## Quick Start

```rust
use rustrial_os::graphics::{text_graphics::*, demo::*, splash::*};
use rustrial_os::vga_buffer::Color;

// Box drawing
draw_box(10, 5, 40, 10, Color::Cyan, Color::Black);
draw_double_box(15, 8, 50, 8, Color::Yellow, Color::Blue);
draw_shadow_box(20, 10, 40, 8, Color::Green, Color::Black);

// Text positioning
write_at(22, 12, "Hello, Rustrial OS!", Color::White, Color::Black);
write_centered(15, "Centered Text", Color::LightCyan, Color::Black);

// Progress bars
draw_progress_bar(10, 12, 50, 75, 100, Color::Green, Color::Black);

// UI elements
show_message_box("Success", "Operation complete!", 20, 10, 40);
show_status_bar("Rustrial OS v0.1.0 | F1=Help");
show_fancy_menu(&["Option 1", "Option 2", "Option 3"], 0);

// Splash and demos
show_boot_splash();           // Boot splash with loading animation
run_graphics_demo();          // Full interactive demo
```

## VGA Graphics Mode (Experimental)

```rust
use rustrial_os::graphics::vga_graphics::{VgaGraphics, VgaColor};

let mut vga = VgaGraphics::new();

// Enter 320x200x256 graphics mode
unsafe { vga.enter_mode_13h(); }

// Drawing operations
vga.clear(VgaColor::Blue as u8);
vga.fill_rect(50, 50, 100, 80, VgaColor::Red as u8);
vga.draw_circle(160, 100, 40, VgaColor::White as u8);
vga.draw_line(0, 0, 319, 199, VgaColor::Yellow as u8);
vga.set_pixel(160, 100, VgaColor::Green as u8);

// Return to text mode
unsafe { vga.return_to_text_mode(); }
```

## API Reference

### Box Drawing Characters
```
─ │ ┌ ┐ └ ┘  (single line)
═ ║ ╔ ╗ ╚ ╝  (double line)
█ ▒ ░          (block shading)
```

### Color Palette
**Text Mode (16 colors)**: Black, Blue, Green, Cyan, Red, Magenta, Brown, LightGray, DarkGray, LightBlue, LightGreen, LightCyan, LightRed, Pink, Yellow, White

**Graphics Mode (256 colors)**: Customizable via `set_palette_entry(index, r, g, b)` (6-bit RGB: 0-63 per channel)

## Performance Notes

- **Text Mode**: Very fast (direct VGA memory at 0xB8000), recommended for UIs
- **Graphics Mode**: Slower (pixel-by-pixel), use for logos and diagrams
- **Best Practice**: Use `interrupts::without_interrupts()` to prevent flickering

## Troubleshooting

**Text Mode**
- Characters not appearing → Check VGA buffer address (0xB8000)
- Wrong colors → Verify Color enum values

**Graphics Mode**
- Black screen → Run QEMU with `-vga std`
- Can't return to text mode → Ensure register restoration
- Flickering → Implement double-buffering

## Future Enhancements

- VESA/VBE support for higher resolutions (1024x768+, true color)
- Font rendering and bitmap images
- Window management and mouse cursor
- Advanced shapes (polygons, bezier curves)
- Sprite system for animations

---

**Graphics Module v0.1.0** - See main README.md for integration examples.
