# Graphics Module for Rustrial OS

## 🎨 Overview

This module adds visual enhancements to Rustrial OS with text-mode graphics and VGA pixel graphics support.

## Features

### Text-Mode Graphics (80x25 VGA)
- Box drawing (single/double lines, shadows)
- Progress bars with percentages  
- 16-color support
- UI components (message boxes, status bars, menus)
- Splash screens with ASCII art

### VGA Graphics Mode (Experimental)
- 320x200 pixel mode with 256 colors
- Basic shapes (lines, rectangles, circles)
- Pixel-level framebuffer access

## Files

- `mod.rs` - Main interface
- `text_graphics.rs` - Box drawing & UI
- `vga_graphics.rs` - Pixel graphics (experimental)
- `splash.rs` - Splash screens
- `demo.rs` - Interactive demos

## Usage Examples

### 1. Display Boot Splash Screen

```rust
use rustrial_os::graphics::demo::show_boot_splash;

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Initialize OS...
    
    // Show splash screen with loading animation
    show_boot_splash();
    
    // Continue with normal operation...
}
```

### 2. Draw Boxes and UI Elements

```rust
use rustrial_os::graphics::text_graphics::*;
use rustrial_os::vga_buffer::Color;

// Draw a simple box
draw_box(10, 5, 40, 10, Color::Cyan, Color::Black);

// Draw a fancy double-line box
draw_double_box(15, 8, 50, 8, Color::Yellow, Color::Blue);

// Draw a shadow box for 3D effect
draw_shadow_box(20, 10, 40, 8, Color::Green, Color::Black);

// Write text at specific position
write_at(22, 12, "Hello, Rustrial OS!", Color::White, Color::Black);

// Center text on a line
write_centered(15, "Centered Text", Color::LightCyan, Color::Black);
```

### 3. Progress Bars

```rust
use rustrial_os::graphics::text_graphics::draw_progress_bar;
use rustrial_os::vga_buffer::Color;

// Draw a progress bar showing 75% completion
draw_progress_bar(10, 12, 50, 75, 100, Color::Green, Color::Black);

// Animated progress bar
for i in 0..=100 {
    draw_progress_bar(10, 12, 50, i, 100, Color::Cyan, Color::Black);
    // Add delay here
}
```

### 4. Message Boxes

```rust
use rustrial_os::graphics::splash::show_message_box;

show_message_box(
    "Success",
    "Operation completed successfully!",
    20, 10, 40
);
```

### 5. Status Bars

```rust
use rustrial_os::graphics::splash::show_status_bar;

show_status_bar("Rustrial OS v0.1.0 | Memory: 128MB | F1=Help");
```

### 6. Fancy Menus

```rust
use rustrial_os::graphics::splash::show_fancy_menu;

let menu_items = [
    "Start System",
    "Run Script",
    "Settings",
    "Help",
];

show_fancy_menu(&menu_items, 0); // 0 = selected index
```

### 7. Run Full Graphics Demo

```rust
use rustrial_os::graphics::demo::run_graphics_demo;

// Show off all graphics capabilities
run_graphics_demo();
```

### 8. VGA Graphics Mode (Advanced)

```rust
use rustrial_os::graphics::vga_graphics::{VgaGraphics, VgaColor};

let mut vga = VgaGraphics::new();

// Enter 320x200x256 graphics mode
unsafe {
    vga.enter_mode_13h();
}

// Clear screen to blue
vga.clear(VgaColor::Blue as u8);

// Draw a red rectangle
vga.fill_rect(50, 50, 100, 80, VgaColor::Red as u8);

// Draw a white circle
vga.draw_circle(160, 100, 40, VgaColor::White as u8);

// Draw a line
vga.draw_line(0, 0, 319, 199, VgaColor::Yellow as u8);

// Set individual pixels
for x in 0..320 {
    vga.set_pixel(x, 100, VgaColor::Green as u8);
}

// Return to text mode when done
unsafe {
    vga.return_to_text_mode();
}
```

## Box Drawing Characters

The module provides constants for IBM PC box-drawing characters:

| Constant | Character | Description |
|----------|-----------|-------------|
| `BOX_HORIZONTAL` | ─ | Horizontal line |
| `BOX_VERTICAL` | │ | Vertical line |
| `BOX_TOP_LEFT` | ┌ | Top-left corner |
| `BOX_TOP_RIGHT` | ┐ | Top-right corner |
| `BOX_BOTTOM_LEFT` | └ | Bottom-left corner |
| `BOX_BOTTOM_RIGHT` | ┘ | Bottom-right corner |
| `BOX_DBL_HORIZONTAL` | ═ | Double horizontal |
| `BOX_DBL_VERTICAL` | ║ | Double vertical |
| `BLOCK_FULL` | █ | Solid block |
| `BLOCK_DARK` | ▒ | Dark shade |
| `BLOCK_MEDIUM` | ░ | Medium shade |

## VGA Color Palette

### Text Mode (16 colors)
- Black, Blue, Green, Cyan, Red, Magenta, Brown, LightGray
- DarkGray, LightBlue, LightGreen, LightCyan, LightRed, Pink, Yellow, White

### Graphics Mode (256 colors)
- 256-color palette with custom RGB values
- Use `set_palette_entry()` to customize colors

## Integration with Existing Features

### 1. Enhanced Menu System

Integrate graphics into your interactive menu:

```rust
use rustrial_os::graphics::splash::show_fancy_menu;
use rustrial_os::graphics::text_graphics::*;
use rustrial_os::vga_buffer::{clear_screen, Color};

pub async fn interactive_menu() {
    clear_screen();
    
    // Draw a nice border
    draw_double_box(0, 0, 80, 25, Color::Cyan, Color::Black);
    
    // Show title
    write_centered(2, "RUSTRIAL OS", Color::Yellow, Color::Black);
    
    // Show menu
    let items = ["Normal Mode", "Scripts", "Help"];
    show_fancy_menu(&items, selected_index);
    
    // Show status bar
    show_status_bar("↑↓: Navigate | Enter: Select | ESC: Exit");
}
```

### 2. Script Execution Visualization

Add progress bars during script execution:

```rust
use rustrial_os::graphics::text_graphics::draw_progress_bar;

fn execute_script(script: &str) {
    draw_progress_bar(10, 20, 60, 0, 100, Color::Green, Color::Black);
    
    // Execute script...
    
    draw_progress_bar(10, 20, 60, 100, 100, Color::Green, Color::Black);
}
```

### 3. System Information Display

```rust
use rustrial_os::graphics::splash::show_system_info_box;

show_system_info_box();
```

## Performance Considerations

### Text Mode
- **Very Fast**: Direct VGA memory writes at 0xB8000
- **No Overhead**: Hardware text mode is extremely efficient
- **Recommended**: Use for interactive UIs and terminals

### Graphics Mode (320x200)
- **Slower**: Must set each pixel individually
- **Memory**: 64,000 bytes of video memory (320 × 200)
- **Use Cases**: Logos, diagrams, pixel art, simple games

## Future Enhancements

### VESA/VBE Support (Coming Soon)
```rust
// Future API example
let mut vbe = VbeGraphics::new();
vbe.set_mode(1024, 768, 32); // 1024x768 with 32-bit color
```

Features planned:
- Higher resolutions (1024x768, 1280x1024, etc.)
- True color (24/32-bit)
- Hardware acceleration
- Framebuffer console

## Troubleshooting

### Text Mode Issues
- **Characters not appearing**: Check VGA buffer address (0xB8000)
- **Wrong colors**: Verify Color enum values
- **Overwriting content**: Use `clear_screen()` before drawing

### Graphics Mode Issues
- **Black screen after mode switch**: May need to run in QEMU with `-vga std`
- **Can't return to text mode**: Ensure proper register restoration
- **Flickering**: Implement double-buffering

## Demo Commands

To see the graphics in action:

```rust
// In kernel_main or menu:
use rustrial_os::graphics::demo::{run_graphics_demo, show_boot_splash};

// Option 1: Full demo
run_graphics_demo();

// Option 2: Just splash screen
show_boot_splash();

// Option 3: Graphics menu
show_graphics_menu();
```

## Best Practices

1. **Always clear before redraw**: Use `clear_screen()` or `clear_region()`
2. **Use interrupts::without_interrupts()**: Prevent flickering during drawing
3. **Test in QEMU**: Graphics features work best in emulation
4. **Color contrast**: Ensure text is readable against backgrounds
5. **Status feedback**: Use progress bars for long operations

## Contributing

Want to add more graphics features? Consider:
- Font rendering system
- Bitmap image support
- Window management
- Mouse cursor
- Higher resolution modes
- Advanced shapes (polygons, bezier curves)
- Sprite system for animations

---

**Rustrial OS Graphics Module v0.1.0**  
Making terminal computing beautiful! 🎨
