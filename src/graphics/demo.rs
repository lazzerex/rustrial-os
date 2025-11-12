/// Graphics demonstration and examples
/// Shows off the visual capabilities of Rustrial OS

use crate::println;
use crate::graphics::text_graphics::*;
use crate::graphics::splash::*;
use crate::vga_buffer::Color;

/// Run a graphics demonstration
pub fn run_graphics_demo() {
    use crate::vga_buffer::clear_screen;
    
    println!("\n=== Graphics Demonstration ===\n");
    println!("This demo will show various graphics features...\n");
    
    // Demo 1: Box drawing
    wait_for_keypress("");
    demo_boxes();
    wait_for_keypress("");
    
    // Demo 2: Progress bars
    demo_progress_bars();
    wait_for_keypress("");
    
    // Demo 3: Fancy UI elements
    demo_ui_elements();
    wait_for_keypress("");
    
    clear_screen();
    println!("\n╔═══════════════════════════════════════╗");
    println!("║     Graphics Demo Complete!           ║");
    println!("╚═══════════════════════════════════════╝");
    println!("\nDemo showcased:");
    println!("  • Box drawing (single/double lines)");
    println!("  • Progress bars and animations");
    println!("  • UI elements (message boxes, status bars)");
    println!("  • Terminal windows and spinners");
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
}

fn demo_progress_bars() {
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
        
        // Simple delay
        for _ in 0..500_000 {
            unsafe { core::arch::asm!("nop") };
        }
    }
}

fn demo_ui_elements() {
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
        for _ in 0..2_000_000 {
            unsafe { core::arch::asm!("nop") };
        }
    }
}

fn wait_for_keypress(_message: &str) {
    // Simplified: just add a delay for demo purposes
    // The menu system will handle the actual keypress to return
    for _ in 0..30_000_000 {
        unsafe { core::arch::asm!("nop") };
    }
}

/// Show the full splash screen with loading animation
pub fn show_boot_splash() {
    show_splash_screen();
    show_loading_progress(50);
    
    // Wait a moment
    for _ in 0..50_000_000 {
        unsafe { core::arch::asm!("nop") };
    }
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
