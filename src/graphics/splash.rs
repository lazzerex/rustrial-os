/// Splash screen and boot animations for Rustrial OS

use alloc::format;
use crate::graphics::text_graphics::*;
use crate::vga_buffer::Color;

/// Display the Rustrial OS boot splash screen
pub fn show_splash_screen() {
    use crate::vga_buffer::clear_screen;
    clear_screen();

    // Draw a nice border around the screen
    draw_double_box(0, 0, 80, 25, Color::Cyan, Color::Black);
    
    // Title box
    draw_filled_box(10, 3, 60, 3, Color::White, Color::Blue);
    write_centered(4, "RUSTRIAL OS", Color::Yellow, Color::Blue);
    
    // ASCII Art Logo
    let logo = [
        " ██████╗ ██╗   ██╗███████╗████████╗██████╗ ██╗ █████╗ ██╗     ",
        " ██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔══██╗██║██╔══██╗██║     ",
        " ██████╔╝██║   ██║███████╗   ██║   ██████╔╝██║███████║██║     ",
        " ██╔══██╗██║   ██║╚════██║   ██║   ██╔══██╗██║██╔══██║██║     ",
        " ██║  ██║╚██████╔╝███████║   ██║   ██║  ██║██║██║  ██║███████╗",
        " ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝  ╚═╝╚══════╝",
    ];
    
    let mut y = 7;
    for line in &logo {
        write_centered(y, line, Color::Cyan, Color::Black);
        y += 1;
    }

    // Version and info
    write_centered(14, "Version 0.1.0", Color::White, Color::Black);
    write_centered(15, "A Custom Operating System with Scripting Support", Color::LightGray, Color::Black);
    
    // Loading bar
    write_centered(18, "Initializing...", Color::Yellow, Color::Black);
    draw_box(15, 19, 50, 3, Color::Green, Color::Black);
    
    // Feature highlights
    let features = [
        "Features:",
        "  ✓ Custom Kernel",
        "  ✓ RustrialScript Interpreter", 
        "  ✓ Memory Management",
        "  ✓ Interrupt Handling",
    ];
    
    let mut y = 8;
    for feature in &features {
        write_at(3, y, feature, Color::LightGreen, Color::Black);
        y += 1;
    }
}

/// Run the full boot splash sequence with staged loading messages.
pub fn run_boot_sequence() {
    show_splash_screen();

    let stages: [(&str, usize); 10] = [
        ("Bootloader hand-off", 5),
        ("Detecting CPU features", 10),
        ("Setting up memory map", 15),
        ("Initializing heap allocator", 10),
        ("Mounting /scripts filesystem", 10),
        ("Loading RustrialScript runtime", 10),
        ("Configuring interrupt controllers", 10),
        ("Activating graphics subsystem", 10),
        ("Spinning up task executor", 10),
        ("Finalizing boot", 10),
    ];

    let mut progress = 0;
    let mut spinner_frame = 0;

    for (label, amount) in stages.iter() {
        draw_filled_box(8, 21, 64, 1, Color::Black, Color::Black);
        let status = format!("{}...", label);
        write_centered(21, &status, Color::LightGray, Color::Black);

        for _ in 0..*amount {
            progress += 1;
            draw_progress_bar(16, 20, 48, progress, 100, Color::Green, Color::Black);
            show_spinner(62, 20, spinner_frame);
            spinner_frame = spinner_frame.wrapping_add(1);
            short_delay(120_000);
        }
    }

    draw_progress_bar(16, 20, 48, 100, 100, Color::Green, Color::Black);
    draw_filled_box(8, 21, 64, 1, Color::Black, Color::Black);
    write_centered(21, "Boot sequence complete!", Color::LightGreen, Color::Black);
    show_spinner(62, 20, spinner_frame);
    short_delay(4_000_000);
}

/// Animated loading progress bar
pub fn show_loading_progress(steps: usize) {
    for i in 0..=steps {
        draw_progress_bar(16, 20, 48, i, steps, Color::Green, Color::Black);
        // Delay (would need a proper timer in real implementation)
        short_delay(1_000_000);
    }
}

/// Simple boot logo using ASCII art
pub fn show_simple_logo() {
    use crate::vga_buffer::clear_screen;
    clear_screen();
    
    let logo = [
        "",
        "    ▄████████ ███    █▄     ▄████████     ███        ▄████████  ▄█     ▄████████  ▄█       ",
        "   ███    ███ ███    ███   ███    ███ ▀█████████▄   ███    ███ ███    ███    ███ ███       ",
        "   ███    ███ ███    ███   ███    █▀     ▀███▀▀██   ███    ███ ███▌   ███    ███ ███       ",
        "  ▄███▄▄▄▄██▀ ███    ███   ███            ███   ▀  ▄███▄▄▄▄██▀ ███▌   ███    ███ ███       ",
        " ▀▀███▀▀▀▀▀   ███    ███ ▀███████████     ███     ▀▀███▀▀▀▀▀   ███▌ ▀███████████ ███       ",
        " ▀███████████ ███    ███          ███     ███     ▀███████████ ███    ███    ███ ███       ",
        "   ███    ███ ███    ███    ▄█    ███     ███       ███    ███ ███    ███    ███ ███▌    ▄ ",
        "   ███    ███ ████████▀   ▄████████▀     ▄████▀     ███    ███ █▀     ███    █▀  █████▄▄██ ",
        "   ███    ███                                        ███    ███                   ▀         ",
        "",
    ];
    
    let mut y = 2;
    for line in &logo {
        write_centered(y, line, Color::Cyan, Color::Black);
        y += 1;
    }
    
    write_centered(15, "Kernel v0.1.0 - Initializing...", Color::Yellow, Color::Black);
}

fn short_delay(iterations: usize) {
    for _ in 0..iterations {
        unsafe { core::arch::asm!("nop") };
    }
}

/// Display system information box
pub fn show_system_info_box() {
    draw_shadow_box(50, 2, 28, 8, Color::Yellow, Color::Blue);
    write_at(52, 3, "System Information", Color::White, Color::Blue);
    draw_hline(52, 4, 24, Color::Yellow, Color::Blue);
    write_at(52, 5, "Memory: OK", Color::LightGreen, Color::Blue);
    write_at(52, 6, "GDT: Loaded", Color::LightGreen, Color::Blue);
    write_at(52, 7, "IDT: Configured", Color::LightGreen, Color::Blue);
    write_at(52, 8, "Heap: Active", Color::LightGreen, Color::Blue);
}

/// Display a message box
pub fn show_message_box(title: &str, message: &str, x: usize, y: usize, width: usize) {
    let height = 6;
    
    // Background
    draw_filled_box(x, y, width, height, Color::White, Color::Blue);
    
    // Border
    draw_double_box(x, y, width, height, Color::Yellow, Color::Blue);
    
    // Title
    let title_x = x + (width - title.len()) / 2;
    write_at(title_x, y + 1, title, Color::White, Color::Blue);
    
    // Separator
    draw_hline(x + 1, y + 2, width - 2, Color::Yellow, Color::Blue);
    
    // Message
    let msg_x = x + 2;
    write_at(msg_x, y + 3, message, Color::LightGray, Color::Blue);
}

/// Display a fancy menu with shadow
pub fn show_fancy_menu(items: &[&str], selected: usize) {
    let width = 40;
    let height = items.len() + 4;
    let x = (80 - width) / 2;
    let y = (25 - height) / 2;
    
    // Draw menu box with shadow
    draw_shadow_box(x, y, width, height, Color::Cyan, Color::Blue);
    
    // Title
    write_centered(y + 1, "MAIN MENU", Color::Yellow, Color::Blue);
    draw_hline(x + 2, y + 2, width - 4, Color::Cyan, Color::Blue);
    
    // Menu items
    for (i, item) in items.iter().enumerate() {
        let item_y = y + 3 + i;
        let item_text = alloc::format!("  {}. {}", i + 1, item);
        
        if i == selected {
            // Highlight selected item
            draw_filled_box(x + 2, item_y, width - 4, 1, Color::Black, Color::LightGray);
            write_at(x + 3, item_y, &item_text, Color::Black, Color::LightGray);
            write_at(x + 2, item_y, "►", Color::Yellow, Color::LightGray);
        } else {
            write_at(x + 3, item_y, &item_text, Color::White, Color::Blue);
        }
    }
}

/// Display a status bar at the bottom of the screen
pub fn show_status_bar(status: &str) {
    draw_filled_box(0, 24, 80, 1, Color::Black, Color::LightGray);
    write_at(2, 24, status, Color::Black, Color::LightGray);
    
    // Show time/info on the right (placeholder)
    write_at(65, 24, "F1=Help", Color::Black, Color::LightGray);
}

/// Animated spinning loader
pub fn show_spinner(x: usize, y: usize, frame: usize) {
    let spinners = ['|', '/', '-', '\\'];
    let spinner_char = spinners[frame % 4];
    
    write_at(x, y, &alloc::format!("{}", spinner_char), Color::Yellow, Color::Black);
}

/// Draw a banner with gradient effect (using different characters)
pub fn show_banner(text: &str, y: usize) {
    let chars = ['█', '▓', '▒', '░'];
    
    // Draw gradient background
    for x in 0..80 {
        let char_idx = (x / 20) % 4;
        write_at(x, y, &alloc::format!("{}", chars[char_idx]), Color::Blue, Color::Black);
    }
    
    // Draw text on top
    write_centered(y, text, Color::White, Color::Black);
}

/// Create a terminal-style window
pub fn create_terminal_window(x: usize, y: usize, width: usize, height: usize, title: &str) {
    // Window background
    draw_filled_box(x, y, width, height, Color::White, Color::Black);
    
    // Title bar
    draw_filled_box(x, y, width, 1, Color::Black, Color::LightGray);
    write_at(x + 2, y, title, Color::Black, Color::LightGray);
    
    // Close button
    write_at(x + width - 3, y, "[X]", Color::Red, Color::LightGray);
    
    // Border
    draw_box(x, y, width, height, Color::LightGray, Color::Black);
}
