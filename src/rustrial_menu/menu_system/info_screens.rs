use crate::vga_buffer::clear_screen;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use crate::rustrial_menu::Color;
use crate::graphics::text_graphics::{draw_shadow_box, draw_filled_box, write_centered, draw_hline, write_at};
use crate::graphics::splash::show_status_bar;

pub fn show_system_info() {
    clear_screen();

    const FRAME_X: usize = 6;
    const FRAME_Y: usize = 2;
    const FRAME_WIDTH: usize = 68;
    const FRAME_HEIGHT: usize = 20;

    draw_shadow_box(FRAME_X, FRAME_Y, FRAME_WIDTH, FRAME_HEIGHT, Color::LightGreen, Color::Black);
    draw_filled_box(FRAME_X + 1, FRAME_Y + 1, FRAME_WIDTH - 2, 3, Color::White, Color::Green);
    write_centered(FRAME_Y + 2, "System Diagnostics", Color::Yellow, Color::Green);
    write_centered(FRAME_Y + 3, "Live snapshot of allocator, collections, and async tasks", Color::LightGray, Color::Green);

    draw_hline(FRAME_X + 2, FRAME_Y + 4, FRAME_WIDTH - 4, Color::LightGreen, Color::Black);

    let heap_value = Box::new(41);
    let heap_line = alloc::format!("Heap sample (Box<i32>) @ {:p}", heap_value);

    let mut vec_buffer = Vec::new();
    for i in 0..500 {
        vec_buffer.push(i);
    }
    let vec_line = alloc::format!("Vec capacity {} elements • buffer @ {:p}", vec_buffer.capacity(), vec_buffer.as_ptr());

    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    let rc_line_before = alloc::format!("Rc strong count before drop: {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    let rc_line_after = alloc::format!("Rc strong count after drop: {}", Rc::strong_count(&cloned_reference));

    let info_lines = [
        heap_line,
        vec_line,
        rc_line_before,
        rc_line_after,
        alloc::format!("Async demo value: {}", 42),
    ];

    for (index, line) in info_lines.iter().enumerate() {
        write_at(FRAME_X + 4, FRAME_Y + 6 + index, line, Color::White, Color::Black);
    }

    write_at(FRAME_X + 4, FRAME_Y + 12, "Keyboard echo mode active:", Color::LightCyan, Color::Black);
    write_at(FRAME_X + 4, FRAME_Y + 13, "Type to see characters appear; ESC returns to the main menu", Color::LightGray, Color::Black);

    show_status_bar("Normal mode • Type freely • ESC returns to main menu");
}




pub fn show_help() {
    clear_screen();

    const FRAME_X: usize = 4;
    const FRAME_Y: usize = 1;
    const FRAME_WIDTH: usize = 72;
    const FRAME_HEIGHT: usize = 22;

    draw_shadow_box(FRAME_X, FRAME_Y, FRAME_WIDTH, FRAME_HEIGHT, Color::Pink, Color::Black);
    draw_filled_box(FRAME_X + 1, FRAME_Y + 1, FRAME_WIDTH - 2, 3, Color::White, Color::Magenta);
    write_centered(FRAME_Y + 2, "RustrialOS Help Center", Color::Yellow, Color::Magenta);
    write_centered(FRAME_Y + 3, "Quick reference for navigation and system features", Color::LightGray, Color::Magenta);

    draw_hline(FRAME_X + 2, FRAME_Y + 4, FRAME_WIDTH - 4, Color::Pink, Color::Black);

    let mut section_y = FRAME_Y + 6;
    write_at(FRAME_X + 4, section_y, "Core Capabilities", Color::White, Color::Black);
    section_y += 1;
    let capabilities = [
        "• Custom memory management and heap allocation",
        "• Interrupt handling with async task executor",
        "• In-memory filesystem (RAMfs) with VFS layer",
        "• RustrialScript interpreter and demos",
        "• Styled text-mode UI with graphics showcase",
    ];
    for line in &capabilities {
        write_at(FRAME_X + 6, section_y, line, Color::LightGray, Color::Black);
        section_y += 1;
    }

    section_y += 1;
    write_at(FRAME_X + 4, section_y, "Menu Overview", Color::White, Color::Black);
    section_y += 1;
    let menu_lines = [
        "[1] Normal Operation  - System diagnostics + keyboard echo",
        "[2] RustrialScript    - Demo runner and script browser",
        "[3] Graphics Demo    - Animated showcase of UI elements",
        "[4] Help             - You're here right now!",
    ];
    for line in &menu_lines {
        write_at(FRAME_X + 6, section_y, line, Color::LightGray, Color::Black);
        section_y += 1;
    }

    section_y += 1;
    write_at(FRAME_X + 4, section_y, "Keyboard Shortcuts", Color::White, Color::Black);
    section_y += 1;
    let shortcut_lines = [
        "ESC  - Return to previous screen",
        "Enter - Activate highlighted option",
        "↑/↓ or W/S - Navigate lists",
        "Backspace - Delete last character in echo mode",
    ];
    for line in &shortcut_lines {
        write_at(FRAME_X + 6, section_y, line, Color::LightGray, Color::Black);
        section_y += 1;
    }

    section_y += 1;
    write_at(FRAME_X + 4, section_y, "Docs & Source", Color::White, Color::Black);
    write_at(FRAME_X + 6, section_y + 1, "github.com/lazzerex/rustrial-os", Color::LightCyan, Color::Black);

    show_status_bar("Press any key to return to the main menu");
}