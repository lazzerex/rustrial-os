use crate::println;
use crate::graphics::text_graphics::{
    draw_shadow_box,
    draw_hline,
    write_at,
    write_centered,
    draw_filled_box,
};
use crate::graphics::splash::show_status_bar;
use crate::vga_buffer::Color;
use crate::fs::FileSystem;
use pc_keyboard::{DecodedKey, KeyCode};
use alloc::{format, vec::Vec, string::String};
use core::cmp::min;

/// Display the script choice menu (Demo vs Browser)
pub fn show_script_choice() {
    use crate::vga_buffer::clear_screen;
    clear_screen();

    const FRAME_X: usize = 10;
    const FRAME_Y: usize = 3;
    const FRAME_WIDTH: usize = 60;
    const FRAME_HEIGHT: usize = 14;

    draw_shadow_box(FRAME_X, FRAME_Y, FRAME_WIDTH, FRAME_HEIGHT, Color::LightBlue, Color::Black);

    draw_filled_box(FRAME_X + 1, FRAME_Y + 1, FRAME_WIDTH - 2, 3, Color::White, Color::Blue);
    write_centered(FRAME_Y + 2, "RustrialScript Options", Color::Yellow, Color::Blue);
    write_centered(FRAME_Y + 3, "Choose how to explore the scripting engine", Color::LightGray, Color::Blue);

    draw_hline(FRAME_X + 2, FRAME_Y + 4, FRAME_WIDTH - 4, Color::Cyan, Color::Black);

    let options = [
        ("[1] Run Demo", "Guided Fibonacci walk-through", Color::LightGreen),
        ("[2] Browse Scripts", "Interactive filesystem-powered browser", Color::LightCyan),
    ];

    for (index, (title, description, accent)) in options.iter().enumerate() {
        let base_y = FRAME_Y + 6 + index * 4;

        draw_filled_box(FRAME_X + 3, base_y - 1, FRAME_WIDTH - 6, 4, Color::Black, Color::Black);
        draw_filled_box(FRAME_X + 4, base_y - 1, 4, 4, Color::Black, *accent);

        write_at(FRAME_X + 5, base_y, title, Color::Black, *accent);
        write_at(FRAME_X + 11, base_y, description, Color::White, Color::Black);
        write_at(FRAME_X + 11, base_y + 1, "- Press corresponding number to continue", Color::LightGray, Color::Black);
    }

    show_status_bar("Press 1-2 to choose  • ESC returns to the main menu");
}

/// Display the script browser with pagination
pub fn show_script_browser(selected_index: usize, page: usize) {
    use crate::vga_buffer::clear_screen;
    clear_screen();

    const FRAME_X: usize = 6;
    const FRAME_Y: usize = 2;
    const FRAME_WIDTH: usize = 68;
    const FRAME_HEIGHT: usize = 20;
    const MAX_VISIBLE: usize = 10;

    draw_shadow_box(FRAME_X, FRAME_Y, FRAME_WIDTH, FRAME_HEIGHT, Color::LightCyan, Color::Black);
    draw_filled_box(FRAME_X + 1, FRAME_Y + 1, FRAME_WIDTH - 2, 3, Color::White, Color::Blue);
    write_centered(FRAME_Y + 2, "Script Browser", Color::Yellow, Color::Blue);
    write_centered(FRAME_Y + 3, "Browse /scripts and run embedded examples", Color::LightGray, Color::Blue);

    draw_hline(FRAME_X + 2, FRAME_Y + 4, FRAME_WIDTH - 4, Color::Cyan, Color::Black);

    let scripts: Vec<String> = crate::fs::root_fs()
        .and_then(|fs| fs.lock().list_dir("/scripts").ok())
        .unwrap_or_else(Vec::new);

    if scripts.is_empty() {
        write_centered(FRAME_Y + FRAME_HEIGHT / 2, "No scripts found in /scripts", Color::LightRed, Color::Black);
        show_status_bar("ESC returns to main menu");
        return;
    }

    let total = scripts.len();
    let total_pages = (total + MAX_VISIBLE - 1) / MAX_VISIBLE;
    let page = page.min(total_pages.saturating_sub(1));
    let window_start = page * MAX_VISIBLE;
    let window_end = min(window_start + MAX_VISIBLE, total);

    // Always clear all possible rows in the visible area
    for visible_row in 0..MAX_VISIBLE {
        let base_y = FRAME_Y + 6 + visible_row * 2;
        draw_filled_box(FRAME_X + 3, base_y - 1, FRAME_WIDTH - 6, 2, Color::Black, Color::Black);
    }

    for (visible_row, script_idx) in (window_start..window_end).enumerate() {
        let filename = scripts[script_idx].trim_start_matches("/scripts/");
        let truncated = if filename.len() > 44 {
            let mut name = String::from(&filename[..44]);
            name.push_str("...");
            name
        } else {
            String::from(filename)
        };

        let base_y = FRAME_Y + 6 + visible_row * 2;
        let highlight = script_idx == selected_index;

        if highlight {
            draw_filled_box(FRAME_X + 3, base_y - 1, FRAME_WIDTH - 6, 2, Color::Black, Color::LightBlue);
            write_at(FRAME_X + 5, base_y, &format!("→ [{:02}] {}", script_idx + 1, truncated), Color::Black, Color::LightBlue);
        } else {
            // Already cleared above
            write_at(FRAME_X + 5, base_y, &format!("  [{:02}] {}", script_idx + 1, truncated), Color::White, Color::Black);
        }
    }

    let footer_y = FRAME_Y + FRAME_HEIGHT - 4;
    write_at(FRAME_X + 4, footer_y, &format!("Page {}/{} | Showing {}-{} of {} scripts", page + 1, total_pages, window_start + 1, window_end, total), Color::LightGray, Color::Black);
    write_at(FRAME_X + 4, footer_y + 1, "↑/↓: Move  ←/→: Page  Enter: Run", Color::LightGray, Color::Black);
    show_status_bar("ESC returns • Enter runs selection");
}

/// Handle keyboard input for the script browser
pub fn handle_script_browser_input(
    key: DecodedKey,
    selected_index: &mut usize,
    page: &mut usize,
    menu_state: &mut super::super::MenuState,
    return_to_browser: &mut bool,
)
{
    match key {
        DecodedKey::RawKey(KeyCode::Escape) | DecodedKey::Unicode('\x1b') => {
            use crate::vga_buffer::clear_screen;
            clear_screen();
            show_script_choice();
            *menu_state = super::super::MenuState::ScriptChoice;
        }
        DecodedKey::RawKey(KeyCode::ArrowUp) | DecodedKey::Unicode('w') | DecodedKey::Unicode('W') => {
            if *selected_index > 0 {
                *selected_index -= 1;
                let max_visible = 10;
                if *selected_index < *page * max_visible {
                    if *page > 0 {
                        *page -= 1;
                    }
                }
                use crate::vga_buffer::clear_screen;
                clear_screen();
                show_script_browser(*selected_index, *page);
            }
        }
        DecodedKey::RawKey(KeyCode::ArrowDown) | DecodedKey::Unicode('s') | DecodedKey::Unicode('S') => {
            let max_index = crate::fs::root_fs()
                .and_then(|fs| {
                    let fs_guard = fs.lock();
                    fs_guard.list_dir("/scripts").ok()
                })
                .map(|scripts| scripts.len())
                .unwrap_or(0);

            if max_index > 0 && *selected_index < max_index.saturating_sub(1) {
                *selected_index += 1;
                let max_visible = 10;
                if *selected_index >= (*page + 1) * max_visible {
                    *page += 1;
                }
                use crate::vga_buffer::clear_screen;
                clear_screen();
                show_script_browser(*selected_index, *page);
            }
        }
        DecodedKey::RawKey(KeyCode::ArrowLeft) => {
            if *page > 0 {
                *page -= 1;
                *selected_index = *page * 10;
                show_script_browser(*selected_index, *page);
            }
        }
        DecodedKey::RawKey(KeyCode::ArrowRight) => {
            let max_index = crate::fs::root_fs()
                .and_then(|fs| {
                    let fs_guard = fs.lock();
                    fs_guard.list_dir("/scripts").ok()
                })
                .map(|scripts| scripts.len())
                .unwrap_or(0);
            let max_page = if max_index == 0 { 0 } else { (max_index - 1) / 10 };
            if *page < max_page {
                *page += 1;
                *selected_index = *page * 10;
                show_script_browser(*selected_index, *page);
            }
        }
        DecodedKey::Unicode('\n') | DecodedKey::RawKey(KeyCode::Return) => {
            // Run the selected script
            run_selected_script(*selected_index);
            show_status_bar("Press any key to return to the script browser");
            *return_to_browser = true;
            *menu_state = super::super::MenuState::HelpMode; // Reuse help mode for "press any key to continue"
        }
        _ => {
            // Ignore other keys
        }
    }
}

/// Run a script at the given index from /scripts directory
pub fn run_selected_script(index: usize) {
    println!("\n╔════════════════════════════════════════╗");
    println!("           Running Script");
    println!("╚════════════════════════════════════════╝\n");
    
    if let Some(fs) = crate::fs::root_fs() {
        let fs = fs.lock();
        match fs.list_dir("/scripts") {
            Ok(scripts) => {
                if index < scripts.len() {
                    let script_path = &scripts[index];
                    let filename = script_path.trim_start_matches("/scripts/");
                    println!("Running: {}\n", filename);
                    
                    match fs.read_file_to_string(script_path) {
                        Ok(content) => {
                            match crate::rustrial_script::run(&content) {
                                Ok(_) => println!("\n✓ Script completed successfully!"),
                                Err(e) => println!("\n✗ Script error: {}", e),
                            }
                        }
                        Err(e) => {
                            println!("Error reading script: {}", e);
                        }
                    }
                } else {
                    println!("Invalid script index");
                }
            }
            Err(e) => {
                println!("Error listing scripts: {}", e);
            }
        }
    } else {
        println!("Error: Filesystem not initialized");
    }
}

/// Run the built-in Fibonacci demo script
pub fn run_demo() {
    println!("=== RustrialScript Demo: Fibonacci Sequence ===\n");
    
    let script = r#"
        // Fibonacci sequence
        let a = 0;
        let b = 1;
        let n = 10;
        let i = 0;
        
        print(a);
        print(b);
        
        while (i < n) {
            let temp = a + b;
            print(temp);
            a = b;
            b = temp;
            i = i + 1;
        }
    "#;
    
    match crate::rustrial_script::run(script) {
        Ok(_) => println!("\n=== Demo completed successfully! ==="),
        Err(e) => println!("\n=== Error: {} ===", e),
    }
}
