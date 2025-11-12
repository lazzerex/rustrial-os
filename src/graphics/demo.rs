/// Graphics demonstration and examples
/// Shows off the visual capabilities of Rustrial OS

use crate::println;
use crate::graphics::text_graphics::*;
use crate::graphics::splash::*;
use crate::task::keyboard;
use crate::vga_buffer::Color;

/// Run a graphics demonstration
pub fn run_graphics_demo() {
    use crate::vga_buffer::clear_screen;

    println!("\n=== Graphics Demonstration ===\n");
    println!("Press ESC at any time to return to the main menu.\n");

    let mut aborted = false;

    demo_boxes();
    if delay_with_escape(12_000_000) {
        aborted = true;
    }

    if !aborted {
        aborted = !demo_progress_bars();
    }

    if !aborted {
        aborted = !demo_ui_elements();
    }

    clear_screen();

    if aborted {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║  Graphics Demo cancelled (ESC pressed) ║");
        println!("╚═══════════════════════════════════════╝");
    } else {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║     Graphics Demo Complete!           ║");
        println!("╚═══════════════════════════════════════╝");
        println!("\nDemo showcased:");
        println!("  • Box drawing (single/double lines)");
        println!("  • Progress bars and animations");
        println!("  • UI elements (message boxes, status bars)");
        println!("  • Terminal windows and spinners");
    }
}

fn demo_boxes() {
    use crate::vga_buffer::clear_screen;
    clear_screen();
    
    write_centered(1, "Box Drawing Demo", Color::Yellow, Color::Black);
    draw_hline(0, 2, 80, Color::Cyan, Color::Black);
    
    // Single-line box
    write_at(2, 4, "Single-line box:", Color::White, Color::Black);
    draw_box(2, 5, 25, 5, Color::Green, Color::Black);
    write_at(4, 7, "Hello from a box!", Color::LightGreen, Color::Black);
    
    // Double-line box
    write_at(30, 4, "Double-line box:", Color::White, Color::Black);
    draw_double_box(30, 5, 25, 5, Color::Cyan, Color::Black);
    write_at(32, 7, "Fancy borders!", Color::LightCyan, Color::Black);
    
    // Filled box
    write_at(58, 4, "Filled box:", Color::White, Color::Black);
    draw_filled_box(58, 5, 20, 5, Color::White, Color::Blue);
    write_at(60, 7, "With color!", Color::Yellow, Color::Blue);
    
    // Shadow box
    write_at(2, 12, "Shadow box:", Color::White, Color::Black);
    draw_shadow_box(2, 13, 30, 6, Color::Magenta, Color::Black);
    write_at(4, 15, "3D effect with shadow", Color::LightRed, Color::Black);

    show_escape_hint();
}

fn demo_progress_bars() -> bool {
    use crate::vga_buffer::clear_screen;
    clear_screen();
    
    write_centered(1, "Progress Bar Demo", Color::Yellow, Color::Black);
    draw_hline(0, 2, 80, Color::Cyan, Color::Black);
    
    // Different progress levels
    write_at(5, 5, "Loading system components:", Color::White, Color::Black);
    
    write_at(7, 7, "Kernel:", Color::LightGray, Color::Black);
    draw_progress_bar(15, 7, 50, 100, 100, Color::Green, Color::Black);
    
    write_at(7, 9, "Drivers:", Color::LightGray, Color::Black);
    draw_progress_bar(15, 9, 50, 75, 100, Color::LightGreen, Color::Black);
    
    write_at(7, 11, "Filesystem:", Color::LightGray, Color::Black);
    draw_progress_bar(15, 11, 50, 50, 100, Color::Yellow, Color::Black);
    
    write_at(7, 13, "Scripting:", Color::LightGray, Color::Black);
    draw_progress_bar(15, 13, 50, 25, 100, Color::LightRed, Color::Black);
    
    write_at(7, 15, "Graphics:", Color::LightGray, Color::Black);
    draw_progress_bar(15, 15, 50, 10, 100, Color::Red, Color::Black);
    
    // Animated progress bar
    write_at(5, 18, "Animated progress (simulated):", Color::White, Color::Black);
    for i in 0..=100 {
        draw_progress_bar(5, 19, 70, i, 100, Color::Cyan, Color::Black);
        
        if delay_with_escape(80_000) {
            return false;
        }
    }

    show_escape_hint();
    true
}

fn demo_ui_elements() -> bool {
    use crate::vga_buffer::clear_screen;
    clear_screen();
    
    write_centered(1, "UI Elements Demo", Color::Yellow, Color::Black);
    draw_hline(0, 2, 80, Color::Cyan, Color::Black);
    
    // Status bar
    show_status_bar("Rustrial OS v0.1.0 | Memory: 128MB | CPU: 100%");
    
    // Message box
    show_message_box(
        "Information",
        "This is a message box!",
        15, 5, 50
    );
    
    // Terminal window
    create_terminal_window(5, 12, 70, 10, "RustrialScript Terminal");
    write_at(7, 14, "rustrial> print(\"Hello, World!\")", Color::LightGreen, Color::Black);
    write_at(7, 15, "Hello, World!", Color::White, Color::Black);
    write_at(7, 16, "rustrial> ", Color::LightGreen, Color::Black);
    
    // Spinners
    for i in 0..16 {
        show_spinner(75, 5, i);
        if delay_with_escape(150_000) {
            return false;
        }
    }

    show_escape_hint();
    true
}

fn delay_with_escape(iterations: usize) -> bool {
    for _ in 0..iterations {
        if escape_pressed() {
            return true;
        }
        unsafe { core::arch::asm!("nop") };
    }
    false
}

fn escape_pressed() -> bool {
    const ESC_MAKE: u8 = 0x01;
    const ESC_BREAK: u8 = 0x81;

    let mut pressed = false;

    while let Some(scancode) = keyboard::try_pop_scancode() {
        if scancode == ESC_MAKE || scancode == ESC_BREAK {
            pressed = true;
        }
    }

    pressed
}

fn show_escape_hint() {
    draw_filled_box(8, 23, 64, 1, Color::Black, Color::Black);
    write_centered(23, "Press ESC to return to the main menu", Color::LightGray, Color::Black);
}

/// Show the full splash screen with loading animation
pub fn show_boot_splash() {
    run_boot_sequence();
}

/// Display enhanced menu using graphics
pub fn show_graphics_menu() {
    use crate::vga_buffer::clear_screen;
    clear_screen();
    
    let items = [
        "Normal Operation",
        "Run Graphics Demo",
        "RustrialScript",
        "System Information",
        "Help",
        "Shutdown",
    ];
    
    show_fancy_menu(&items, 0);
    show_status_bar("Use ↑↓ to navigate, Enter to select, ESC to exit");
}
