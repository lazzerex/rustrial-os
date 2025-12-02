use crate::{print, println};
use crate::task::keyboard;
use crate::fs::FileSystem; // Import the FileSystem trait
use crate::graphics::text_graphics::{
    draw_filled_box,
    draw_hline,
    draw_shadow_box,
    write_at,
    write_centered,
};
use crate::graphics::splash::show_status_bar;
use crate::vga_buffer::Color;
use core::cmp::min;
use futures_util::stream::StreamExt;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc, string::String};


#[derive(PartialEq)]
enum MenuState {
    MainMenu,
    ScriptChoice,     // New state for choosing demo or browse
    ScriptBrowser,
    NormalMode,
    HardwareMenu,     // New state for hardware information
    HardwareDetail(HardwarePanel),
    HelpMode,
}

#[derive(Clone, Copy, PartialEq)]
enum HardwarePanel {
    Overview,
    Cpu,
    Rtc,
    Pci,
}

/// Interactive menu - returns true if user wants to return to desktop, false to stay in menu
pub async fn interactive_menu() -> bool {
    
    let mut scancodes = keyboard::ScancodeStream::new();
    let mut kb = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    
    let mut menu_state = MenuState::MainMenu;
    let mut selected_script_index: usize = 0;
    let mut script_browser_page: usize = 0;
    let mut return_to_browser = false; // Track if we should return to browser after help mode
    show_menu_screen();
    
    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = kb.add_byte(scancode) {
            if let Some(key) = kb.process_keyevent(key_event) {
                match menu_state {
                    MenuState::MainMenu => {
                        if let DecodedKey::Unicode(ch) = key {
                            match ch {
                                '1' => {
                                    show_system_info();
                                    menu_state = MenuState::NormalMode;
                                }
                                '2' => {
                                    use crate::vga_buffer::clear_screen;
                                    clear_screen();
                                    show_script_choice();
                                    menu_state = MenuState::ScriptChoice;
                                }
                                '3' => {
                                    show_status_bar("Launching graphics showcase… any key returns afterwards");
                                    crate::graphics::demo::run_graphics_demo();
                                    show_status_bar("Press any key to return to the main menu");
                                    menu_state = MenuState::HelpMode;
                                }
                                '4' => {
                                    use crate::vga_buffer::clear_screen;
                                    clear_screen();
                                    show_hardware_submenu();
                                    menu_state = MenuState::HardwareMenu;
                                }
                                '5' => {
                                    show_help();
                                    show_status_bar("Press any key to return to the main menu");
                                    menu_state = MenuState::HelpMode;
                                }
                                '0' => {
                                    // Exit to desktop
                                    return true;
                                }
                                _ => {
                                    // ignore other keys in menu mode
                                }
                            }
                        }
                    }
                    MenuState::ScriptChoice => {
                        if let DecodedKey::Unicode(ch) = key {
                            match ch {
                                '1' => {
                                    println!("\n→ Running RustrialScript Demo...\n");
                                    run_demo();
                                    println!("\n→ Demo complete! Press any key to return to menu...");
                                    return_to_browser = false;
                                    menu_state = MenuState::HelpMode;
                                }
                                '2' => {
                                    use crate::vga_buffer::clear_screen;
                                    clear_screen();
                                    selected_script_index = 0;
                                    script_browser_page = 0;
                                    show_script_browser(selected_script_index, script_browser_page);
                                    menu_state = MenuState::ScriptBrowser;
                                }
                                _ => {
                                    // ESC or other keys return to main menu
                                }
                            }
                        }
                        // Handle ESC key
                        if let DecodedKey::RawKey(KeyCode::Escape) = key {
                            use crate::vga_buffer::clear_screen;
                            clear_screen();
                            show_menu_screen();
                            menu_state = MenuState::MainMenu;
                        }
                    }
                    MenuState::ScriptBrowser => {
                        handle_script_browser_input(
                            key,
                            &mut selected_script_index,
                            &mut script_browser_page,
                            &mut menu_state,
                            &mut return_to_browser,
                        );
                    }
                    MenuState::HardwareMenu => {
                        // Handle hardware menu input
                        if let DecodedKey::Unicode(ch) = key {
                            match ch {
                                '1' => {
                                    use crate::vga_buffer::clear_screen;
                                    clear_screen();
                                    show_all_hardware_info();
                                    menu_state = MenuState::HardwareDetail(HardwarePanel::Overview);
                                }
                                '2' => {
                                    use crate::vga_buffer::clear_screen;
                                    clear_screen();
                                    show_cpu_info();
                                    menu_state = MenuState::HardwareDetail(HardwarePanel::Cpu);
                                }
                                '3' => {
                                    use crate::vga_buffer::clear_screen;
                                    clear_screen();
                                    show_rtc_info();
                                    menu_state = MenuState::HardwareDetail(HardwarePanel::Rtc);
                                }
                                '4' => {
                                    use crate::vga_buffer::clear_screen;
                                    clear_screen();
                                    show_pci_info();
                                    menu_state = MenuState::HardwareDetail(HardwarePanel::Pci);
                                }
                                _ => {}
                            }
                        }
                        // ESC returns to main menu
                        match key {
                            DecodedKey::RawKey(KeyCode::Escape) | DecodedKey::Unicode('\x1b') => {
                                use crate::vga_buffer::clear_screen;
                                clear_screen();
                                show_menu_screen();
                                menu_state = MenuState::MainMenu;
                            }
                            _ => {}
                        }
                        // Hardware submenu remains active until user leaves or selects details
                    }
                    MenuState::HardwareDetail(_current_panel) => {
                        match key {
                            DecodedKey::RawKey(KeyCode::Escape) | DecodedKey::Unicode('\x1b') => {
                                use crate::vga_buffer::clear_screen;
                                clear_screen();
                                show_hardware_submenu();
                                menu_state = MenuState::HardwareMenu;
                            }
                            DecodedKey::Unicode(ch) => {
                                let next_panel = match ch {
                                    '1' => Some(HardwarePanel::Overview),
                                    '2' => Some(HardwarePanel::Cpu),
                                    '3' => Some(HardwarePanel::Rtc),
                                    '4' => Some(HardwarePanel::Pci),
                                    'h' | 'H' => {
                                        use crate::vga_buffer::clear_screen;
                                        clear_screen();
                                        show_hardware_submenu();
                                        menu_state = MenuState::HardwareMenu;
                                        None
                                    }
                                    '0' => {
                                        use crate::vga_buffer::clear_screen;
                                        clear_screen();
                                        show_menu_screen();
                                        menu_state = MenuState::MainMenu;
                                        None
                                    }
                                    _ => None,
                                };

                                if let Some(panel) = next_panel {
                                    match panel {
                                        HardwarePanel::Overview => show_all_hardware_info(),
                                        HardwarePanel::Cpu => show_cpu_info(),
                                        HardwarePanel::Rtc => show_rtc_info(),
                                        HardwarePanel::Pci => show_pci_info(),
                                    }
                                    menu_state = MenuState::HardwareDetail(panel);
                                }
                            }
                            _ => {}
                        }
                    }
                    MenuState::HelpMode => {
                        // ESC or any key returns to menu from help
                        if return_to_browser {
                            use crate::vga_buffer::clear_screen;
                            clear_screen();
                            show_script_browser(selected_script_index, script_browser_page);
                            menu_state = MenuState::ScriptBrowser;
                            return_to_browser = false;
                        } else {
                            use crate::vga_buffer::clear_screen;
                            clear_screen();
                            show_menu_screen();
                            menu_state = MenuState::MainMenu;
                        }
                    }
                    MenuState::NormalMode => {
                        match key {
                            DecodedKey::RawKey(KeyCode::Escape) | DecodedKey::Unicode('\x1b') => {
                                use crate::vga_buffer::clear_screen;
                                clear_screen();
                                show_menu_screen();
                                menu_state = MenuState::MainMenu;
                            }
                            DecodedKey::Unicode('\x08') => {
                                // Backspace
                                use crate::vga_buffer::backspace;
                                backspace();
                            }
                            DecodedKey::Unicode(character) => print!("{}", character),
                            DecodedKey::RawKey(_) => {
                                // Ignore other raw keys
                            }
                        }
                    }

                }
            }
        }
    }
    
    // Should never reach here, but return false by default
    false
}

// Hardware menu helper functions (native C/Assembly implementation)
fn show_all_hardware_info() {
    use crate::graphics::text_graphics::{draw_shadow_box, write_centered, draw_hline, write_at};
    use crate::vga_buffer::Color;
    
    crate::vga_buffer::clear_screen();
    
    const BOX_X: usize = 2;
    const BOX_Y: usize = 1;
    const BOX_WIDTH: usize = 76;
    const BOX_HEIGHT: usize = 23;
    
    draw_shadow_box(BOX_X, BOX_Y, BOX_WIDTH, BOX_HEIGHT, Color::LightCyan, Color::Black);
    write_centered(BOX_Y + 1, "HARDWARE INFORMATION - Native C/Assembly", Color::Yellow, Color::Black);
    draw_hline(BOX_X + 2, BOX_Y + 2, BOX_WIDTH - 4, Color::Cyan, Color::Black);
    
    // CPU Section
    write_at(BOX_X + 2, BOX_Y + 4, "[CPU - Assembly CPUID]", Color::LightBlue, Color::Black);
    let cpu_info = crate::native_ffi::CpuInfo::get();
    write_at(BOX_X + 3, BOX_Y + 5, &alloc::format!("Vendor: {}", cpu_info.vendor_str()), Color::White, Color::Black);
    
    let brand = cpu_info.brand_str();
    let brand_display = if brand.len() > 60 { &brand[..60] } else { brand };
    write_at(BOX_X + 3, BOX_Y + 6, &alloc::format!("Brand:  {}", brand_display), Color::White, Color::Black);
    
    // RTC Section
    write_at(BOX_X + 2, BOX_Y + 8, "[Real-Time Clock - C RTC Driver]", Color::Magenta, Color::Black);
    let datetime = crate::native_ffi::DateTime::read();
    write_at(BOX_X + 3, BOX_Y + 9, &alloc::format!("{}", datetime), Color::LightCyan, Color::Black);
    
    // PCI Section
    write_at(BOX_X + 2, BOX_Y + 11, "[PCI Devices - C PCI Enumeration]", Color::LightGreen, Color::Black);
    let devices = crate::native_ffi::enumerate_pci_devices();
    
    if devices.is_empty() {
        write_at(BOX_X + 3, BOX_Y + 12, "No PCI devices found.", Color::LightRed, Color::Black);
    } else {
        const MAX_VISIBLE: usize = 8;
        for (i, device) in devices.iter().take(MAX_VISIBLE).enumerate() {
            let device_str = alloc::format!("{}", device);
            write_at(BOX_X + 3, BOX_Y + 12 + i, &device_str, Color::White, Color::Black);
        }
        
        if devices.len() > MAX_VISIBLE {
            write_at(BOX_X + 3, BOX_Y + 20, &alloc::format!("...and {} more devices", devices.len() - MAX_VISIBLE), Color::Yellow, Color::Black);
        }
    }
    
    write_centered(BOX_Y + BOX_HEIGHT - 1, "Press ESC to return to Hardware Menu", Color::LightGray, Color::Black);
}

fn show_cpu_info() {
    use crate::graphics::text_graphics::{draw_shadow_box, write_centered, draw_hline, write_at};
    use crate::vga_buffer::Color;
    
    crate::vga_buffer::clear_screen();
    
    const BOX_X: usize = 5;
    const BOX_Y: usize = 2;
    const BOX_WIDTH: usize = 70;
    const BOX_HEIGHT: usize = 10;
    
    draw_shadow_box(BOX_X, BOX_Y, BOX_WIDTH, BOX_HEIGHT, Color::LightCyan, Color::Black);
    write_centered(BOX_Y + 1, "CPU INFORMATION - Assembly CPUID", Color::Yellow, Color::Black);
    draw_hline(BOX_X + 2, BOX_Y + 2, BOX_WIDTH - 4, Color::Cyan, Color::Black);
    
    let cpu_info = crate::native_ffi::CpuInfo::get();
    write_at(BOX_X + 3, BOX_Y + 4, &alloc::format!("CPU Vendor: {}", cpu_info.vendor_str()), Color::White, Color::Black);
    write_at(BOX_X + 3, BOX_Y + 5, &alloc::format!("CPU Brand:  {}", cpu_info.brand_str()), Color::White, Color::Black);
    
    let mut features = alloc::vec::Vec::new();
    if cpu_info.has_sse() { features.push("SSE"); }
    if cpu_info.has_sse2() { features.push("SSE2"); }
    if cpu_info.has_sse3() { features.push("SSE3"); }
    if cpu_info.has_avx() { features.push("AVX"); }
    
    write_at(BOX_X + 3, BOX_Y + 6, &alloc::format!("Features:   {}", features.join(", ")), Color::LightGreen, Color::Black);
    
    write_centered(BOX_Y + BOX_HEIGHT - 1, "Press ESC to return to Hardware Menu", Color::LightGray, Color::Black);
}

fn show_rtc_info() {
    use crate::graphics::text_graphics::{draw_shadow_box, write_centered, draw_hline, write_at};
    use crate::vga_buffer::Color;
    
    crate::vga_buffer::clear_screen();
    
    const BOX_X: usize = 5;
    const BOX_Y: usize = 2;
    const BOX_WIDTH: usize = 70;
    const BOX_HEIGHT: usize = 8;
    
    draw_shadow_box(BOX_X, BOX_Y, BOX_WIDTH, BOX_HEIGHT, Color::LightCyan, Color::Black);
    write_centered(BOX_Y + 1, "REAL-TIME CLOCK - C RTC Driver", Color::Yellow, Color::Black);
    draw_hline(BOX_X + 2, BOX_Y + 2, BOX_WIDTH - 4, Color::Cyan, Color::Black);
    
    let datetime = crate::native_ffi::DateTime::read();
    let dt_str = alloc::format!("{}", datetime);
    write_at(BOX_X + 3, BOX_Y + 4, &dt_str, Color::LightCyan, Color::Black);
    
    write_centered(BOX_Y + BOX_HEIGHT - 1, "Press ESC to return to Hardware Menu", Color::LightGray, Color::Black);
}

fn show_pci_info() {
    use crate::graphics::text_graphics::{draw_shadow_box, write_centered, draw_hline, write_at};
    use crate::vga_buffer::Color;
    
    crate::vga_buffer::clear_screen();
    
    const BOX_X: usize = 3;
    const BOX_Y: usize = 1;
    const BOX_WIDTH: usize = 74;
    const BOX_HEIGHT: usize = 22;
    
    draw_shadow_box(BOX_X, BOX_Y, BOX_WIDTH, BOX_HEIGHT, Color::LightCyan, Color::Black);
    write_centered(BOX_Y + 1, "PCI DEVICES - C PCI Enumeration", Color::Yellow, Color::Black);
    draw_hline(BOX_X + 2, BOX_Y + 2, BOX_WIDTH - 4, Color::Cyan, Color::Black);
    
    let devices = crate::native_ffi::enumerate_pci_devices();
    
    if devices.is_empty() {
        write_at(BOX_X + 3, BOX_Y + 4, "No PCI devices found.", Color::LightRed, Color::Black);
        write_centered(BOX_Y + BOX_HEIGHT - 1, "Press ESC to return to Hardware Menu", Color::LightGray, Color::Black);
        return;
    }
    
    const MAX_VISIBLE: usize = 15;
    let total = devices.len();
    let display_count = core::cmp::min(total, MAX_VISIBLE);
    
    for (i, device) in devices.iter().take(display_count).enumerate() {
        let device_str = alloc::format!("{}", device);
        let y_pos = BOX_Y + 4 + i;
        write_at(BOX_X + 3, y_pos, &device_str, Color::White, Color::Black);
    }
    
    let footer_y = BOX_Y + BOX_HEIGHT - 2;
    if total > MAX_VISIBLE {
        write_at(BOX_X + 3, footer_y, &alloc::format!("Showing {} of {} devices", display_count, total), Color::Yellow, Color::Black);
    } else {
        write_at(BOX_X + 3, footer_y, &alloc::format!("Total: {} devices", total), Color::LightGreen, Color::Black);
    }
    
    write_centered(BOX_Y + BOX_HEIGHT - 1, "Press ESC to return to Hardware Menu", Color::LightGray, Color::Black);
}


fn show_menu_screen() {
    crate::vga_buffer::clear_screen();

    const FRAME_X: usize = 5;
    const FRAME_Y: usize = 2;
    const FRAME_WIDTH: usize = 70;
    const FRAME_HEIGHT: usize = 22;

    draw_shadow_box(FRAME_X, FRAME_Y, FRAME_WIDTH, FRAME_HEIGHT, Color::LightCyan, Color::Black);

    // Header band
    draw_filled_box(FRAME_X + 1, FRAME_Y + 1, FRAME_WIDTH - 2, 3, Color::White, Color::Blue);
    write_centered(FRAME_Y + 2, "RustrialOS // Main Menu", Color::Yellow, Color::Blue);
    write_centered(FRAME_Y + 3, "Select a mode to explore the system", Color::LightGray, Color::Blue);

    draw_hline(FRAME_X + 2, FRAME_Y + 4, FRAME_WIDTH - 4, Color::Cyan, Color::Black);

    let menu_items = [
        ("[1]", "Continue with normal operation", "System information and keyboard echo mode"),
        ("[2]", "RustrialScript", "Run the demo or browse the embedded script library"),
        ("[3]", "Graphics Demo", "Showcase the text-mode UI and visual effects"),
        ("[4]", "Hardware Info", "Native C/Assembly implementation (Phase 1)"),
        ("[5]", "Show Help", "Keyboard shortcuts and feature overview"),
        ("[0]", "Exit to Desktop", "Return to the desktop environment"),
    ];

    for (index, (label, title, description)) in menu_items.iter().enumerate() {
        let base_y = FRAME_Y + 6 + index * 3;
        draw_filled_box(FRAME_X + 2, base_y - 1, FRAME_WIDTH - 4, 3, Color::Black, Color::Black);

        let accent_color = match index {
            0 => Color::LightGreen,
            1 => Color::LightBlue,
            2 => Color::LightRed,
            _ => Color::Magenta,
        };

        draw_filled_box(FRAME_X + 3, base_y - 1, 4, 3, Color::Black, accent_color);

        write_at(FRAME_X + 4, base_y, label, Color::Black, accent_color);
        write_at(FRAME_X + 10, base_y, title, Color::White, Color::Black);
        write_at(FRAME_X + 10, base_y + 1, description, Color::LightGray, Color::Black);

        if index < menu_items.len() - 1 {
            draw_hline(FRAME_X + 3, base_y + 2, FRAME_WIDTH - 6, Color::DarkGray, Color::Black);
        }
    }

    write_centered(FRAME_Y + FRAME_HEIGHT + 1, "Welcome to the Rustrial OS playground", Color::LightCyan, Color::Black);
    show_status_bar("Press 1-5 to select, 0 to exit  •  ESC returns from other views");
}

fn show_hardware_submenu() {
    use crate::vga_buffer::clear_screen;
    clear_screen();

    const FRAME_X: usize = 8;
    const FRAME_Y: usize = 3;
    const FRAME_WIDTH: usize = 64;
    const FRAME_HEIGHT: usize = 16;

    draw_shadow_box(FRAME_X, FRAME_Y, FRAME_WIDTH, FRAME_HEIGHT, Color::LightCyan, Color::Black);

    // Header band
    draw_filled_box(FRAME_X + 1, FRAME_Y + 1, FRAME_WIDTH - 2, 3, Color::White, Color::Blue);
    write_centered(FRAME_Y + 2, "Hardware Information Menu", Color::Yellow, Color::Blue);
    write_centered(FRAME_Y + 3, "Native C/Assembly Phase 1 Implementation", Color::LightGray, Color::Blue);

    draw_hline(FRAME_X + 2, FRAME_Y + 4, FRAME_WIDTH - 4, Color::Cyan, Color::Black);

    let menu_items = [
        ("[1]", "Show All Hardware Info", "CPU, RTC, and PCI in one view"),
        ("[2]", "CPU Information", "Assembly CPUID detection"),
        ("[3]", "Real-Time Clock", "C RTC driver (date & time)"),
        ("[4]", "PCI Devices", "C PCI enumeration"),
    ];

    for (index, (label, title, description)) in menu_items.iter().enumerate() {
        let base_y = FRAME_Y + 6 + index * 2;
        
        let accent_color = match index {
            0 => Color::LightGreen,
            1 => Color::LightBlue,
            2 => Color::Magenta,
            3 => Color::LightRed,
            _ => Color::Cyan,
        };

        draw_filled_box(FRAME_X + 3, base_y - 1, 4, 2, Color::Black, accent_color);
        write_at(FRAME_X + 4, base_y, label, Color::Black, accent_color);
        write_at(FRAME_X + 10, base_y, title, Color::White, Color::Black);
        write_at(FRAME_X + 10, base_y + 1, description, Color::LightGray, Color::Black);
    }

    write_centered(FRAME_Y + FRAME_HEIGHT - 2, "Press ESC to return to Main Menu", Color::LightCyan, Color::Black);
    show_status_bar("Press 1-4 to select  •  ESC returns to main menu");
}

fn show_script_choice() {
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

    show_status_bar("Press 1-2 to choose  •  ESC returns to the main menu");
}

fn show_script_browser(selected_index: usize, page: usize) {
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
            write_at(FRAME_X + 5, base_y, &alloc::format!("→ [{:02}] {}", script_idx + 1, truncated), Color::Black, Color::LightBlue);
        } else {
            // Already cleared above
            write_at(FRAME_X + 5, base_y, &alloc::format!("  [{:02}] {}", script_idx + 1, truncated), Color::White, Color::Black);
        }
    }

    let footer_y = FRAME_Y + FRAME_HEIGHT - 4;
    write_at(FRAME_X + 4, footer_y, &alloc::format!("Page {}/{} | Showing {}-{} of {} scripts", page + 1, total_pages, window_start + 1, window_end, total), Color::LightGray, Color::Black);
    write_at(FRAME_X + 4, footer_y + 1, "↑/↓: Move  ←/→: Page  Enter: Run", Color::LightGray, Color::Black);
    show_status_bar("ESC returns • Enter runs selection");
}

fn handle_script_browser_input(
    key: DecodedKey,
    selected_index: &mut usize,
    page: &mut usize,
    menu_state: &mut MenuState,
    return_to_browser: &mut bool,
)
{
    match key {
        DecodedKey::RawKey(KeyCode::Escape) | DecodedKey::Unicode('\x1b') => {
            use crate::vga_buffer::clear_screen;
            clear_screen();
            show_script_choice();
            *menu_state = MenuState::ScriptChoice;
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
            *menu_state = MenuState::HelpMode; // Reuse help mode for "press any key to continue"
        }
        _ => {
            // Ignore other keys
        }
    }
}

fn run_selected_script(index: usize) {
    println!("\n═══════════════════════════════════════════");
    println!("           Running Script");
    println!("═══════════════════════════════════════════\n");
    
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


fn show_system_info() {
    use crate::vga_buffer::clear_screen;
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


fn run_demo() {
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


fn show_help() {
    use crate::vga_buffer::clear_screen;
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

// Public wrapper functions for desktop icons
pub async fn show_system_info_from_desktop() {
    show_system_info();
    
    // Wait for any key - use try_pop_scancode to avoid creating new stream
    let mut kb = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    
    loop {
        if let Some(scancode) = keyboard::try_pop_scancode() {
            if let Ok(Some(key_event)) = kb.add_byte(scancode) {
                if let Some(_key) = kb.process_keyevent(key_event) {
                    // Any key returns to desktop
                    return;
                }
            }
        }
        // Small delay to prevent busy-waiting
        for _ in 0..10000 {
            core::hint::spin_loop();
        }
    }
}

pub async fn show_scripts_from_desktop() {
    use crate::vga_buffer::clear_screen;
    clear_screen();
    show_script_choice();
    
    // Handle script choice navigation - use try_pop_scancode
    let mut kb = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    
    loop {
        if let Some(scancode) = keyboard::try_pop_scancode() {
            if let Ok(Some(key_event)) = kb.add_byte(scancode) {
                if let Some(key) = kb.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode('1') => {
                            println!("\n→ Running RustrialScript Demo...\n");
                            run_demo();
                            println!("\n→ Demo complete! Press any key to return to desktop...");
                            // Wait for key
                            loop {
                                if let Some(sc) = keyboard::try_pop_scancode() {
                                    if let Ok(Some(ke)) = kb.add_byte(sc) {
                                        if kb.process_keyevent(ke).is_some() {
                                            return;
                                        }
                                    }
                                }
                                for _ in 0..10000 { core::hint::spin_loop(); }
                            }
                        }
                        DecodedKey::Unicode('2') => {
                            clear_screen();
                            let mut selected_index = 0;
                            let mut page = 0;
                            show_script_browser(selected_index, page);
                            
                            // Script browser loop
                            loop {
                                if let Some(sc) = keyboard::try_pop_scancode() {
                                    if let Ok(Some(ke)) = kb.add_byte(sc) {
                                        if let Some(browser_key) = kb.process_keyevent(ke) {
                                            match browser_key {
                                                DecodedKey::RawKey(KeyCode::Escape) | DecodedKey::Unicode('\x1b') => {
                                                    clear_screen();
                                                    show_script_choice();
                                                    break;
                                                }
                                                DecodedKey::RawKey(KeyCode::ArrowUp) | DecodedKey::Unicode('w') | DecodedKey::Unicode('W') => {
                                                    if selected_index > 0 {
                                                        selected_index -= 1;
                                                        if selected_index < page * 10 && page > 0 {
                                                            page -= 1;
                                                        }
                                                        clear_screen();
                                                        show_script_browser(selected_index, page);
                                                    }
                                                }
                                                DecodedKey::RawKey(KeyCode::ArrowDown) | DecodedKey::Unicode('s') | DecodedKey::Unicode('S') => {
                                                    let max_index = crate::fs::root_fs()
                                                        .and_then(|fs| fs.lock().list_dir("/scripts").ok())
                                                        .map(|scripts| scripts.len())
                                                        .unwrap_or(0);
                                                    if max_index > 0 && selected_index < max_index.saturating_sub(1) {
                                                        selected_index += 1;
                                                        if selected_index >= (page + 1) * 10 {
                                                            page += 1;
                                                        }
                                                        clear_screen();
                                                        show_script_browser(selected_index, page);
                                                    }
                                                }
                                                DecodedKey::Unicode('\n') | DecodedKey::RawKey(KeyCode::Return) => {
                                                    run_selected_script(selected_index);
                                                    show_status_bar("Press any key to return to script browser");
                                                    // Wait for key
                                                    loop {
                                                        if let Some(ret_sc) = keyboard::try_pop_scancode() {
                                                            if let Ok(Some(ret_ke)) = kb.add_byte(ret_sc) {
                                                                if kb.process_keyevent(ret_ke).is_some() {
                                                                    clear_screen();
                                                                    show_script_browser(selected_index, page);
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        for _ in 0..10000 { core::hint::spin_loop(); }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                for _ in 0..10000 { core::hint::spin_loop(); }
                            }
                        }
                        DecodedKey::RawKey(KeyCode::Escape) => {
                            return;
                        }
                        _ => {}
                    }
                }
            }
        }
        for _ in 0..10000 { core::hint::spin_loop(); }
    }
}

pub async fn show_hardware_from_desktop() {
    show_hardware_submenu();
    
    // Handle hardware menu navigation - use try_pop_scancode
    let mut kb = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    
    loop {
        if let Some(scancode) = keyboard::try_pop_scancode() {
            if let Ok(Some(key_event)) = kb.add_byte(scancode) {
                if let Some(key) = kb.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode('1') => {
                            use crate::vga_buffer::clear_screen;
                            clear_screen();
                            show_all_hardware_info();
                            // Wait for key to return
                            loop {
                                if let Some(sc) = keyboard::try_pop_scancode() {
                                    if let Ok(Some(ke)) = kb.add_byte(sc) {
                                        if kb.process_keyevent(ke).is_some() {
                                            show_hardware_submenu();
                                            break;
                                        }
                                    }
                                }
                                for _ in 0..10000 { core::hint::spin_loop(); }
                            }
                        }
                        DecodedKey::Unicode('2') => {
                            use crate::vga_buffer::clear_screen;
                            clear_screen();
                            show_cpu_info();
                            loop {
                                if let Some(sc) = keyboard::try_pop_scancode() {
                                    if let Ok(Some(ke)) = kb.add_byte(sc) {
                                        if kb.process_keyevent(ke).is_some() {
                                            show_hardware_submenu();
                                            break;
                                        }
                                    }
                                }
                                for _ in 0..10000 { core::hint::spin_loop(); }
                            }
                        }
                        DecodedKey::Unicode('3') => {
                            use crate::vga_buffer::clear_screen;
                            clear_screen();
                            show_rtc_info();
                            loop {
                                if let Some(sc) = keyboard::try_pop_scancode() {
                                    if let Ok(Some(ke)) = kb.add_byte(sc) {
                                        if kb.process_keyevent(ke).is_some() {
                                            show_hardware_submenu();
                                            break;
                                        }
                                    }
                                }
                                for _ in 0..10000 { core::hint::spin_loop(); }
                            }
                        }
                        DecodedKey::Unicode('4') => {
                            use crate::vga_buffer::clear_screen;
                            clear_screen();
                            show_pci_info();
                            loop {
                                if let Some(sc) = keyboard::try_pop_scancode() {
                                    if let Ok(Some(ke)) = kb.add_byte(sc) {
                                        if kb.process_keyevent(ke).is_some() {
                                            show_hardware_submenu();
                                            break;
                                        }
                                    }
                                }
                                for _ in 0..10000 { core::hint::spin_loop(); }
                            }
                        }
                        DecodedKey::RawKey(KeyCode::Escape) => {
                            return;
                        }
                        _ => {}
                    }
                }
            }
        }
        for _ in 0..10000 { core::hint::spin_loop(); }
    }
}
