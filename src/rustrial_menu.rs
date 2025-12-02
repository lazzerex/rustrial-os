use crate::{print, println};
use crate::task::keyboard;
use crate::fs::FileSystem;
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
    let mut kb = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    let mut in_submenu = true;
    loop {
        if let Some(scancode) = keyboard::try_pop_scancode() {
            if let Ok(Some(key_event)) = kb.add_byte(scancode) {
                if let Some(key) = kb.process_keyevent(key_event) {
                    if in_submenu {
                        match key {
                            DecodedKey::Unicode('1') => {
                                use crate::vga_buffer::clear_screen;
                                clear_screen();
                                show_all_hardware_info();
                                in_submenu = false;
                            }
                            DecodedKey::Unicode('2') => {
                                use crate::vga_buffer::clear_screen;
                                clear_screen();
                                show_cpu_info();
                                in_submenu = false;
                            }
                            DecodedKey::Unicode('3') => {
                                use crate::vga_buffer::clear_screen;
                                clear_screen();
                                show_rtc_info();
                                in_submenu = false;
                            }
                            DecodedKey::Unicode('4') => {
                                use crate::vga_buffer::clear_screen;
                                clear_screen();
                                show_pci_info();
                                in_submenu = false;
                            }
                            DecodedKey::Unicode('0') => {
                                // Return to desktop
                                return;
                            }
                            DecodedKey::RawKey(KeyCode::Escape) => {
                                return;
                            }
                            _ => {}
                        }
                    } else {
                        match key {
                            DecodedKey::Unicode('0') => {
                                // Return to desktop from detail panel
                                return;
                            }
                            DecodedKey::RawKey(KeyCode::Escape) => {
                                // Exit to desktop from any detail panel
                                return;
                            }
                            _ => {
                                // Any key returns to submenu
                                show_hardware_submenu();
                                in_submenu = true;
                            }
                        }
                    }
                }
            }
        }
        for _ in 0..10000 { core::hint::spin_loop(); }
    }
}
