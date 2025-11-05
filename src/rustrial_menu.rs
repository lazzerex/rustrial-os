use crate::{print, println};
use crate::task::keyboard;
use crate::fs::FileSystem; // Import the FileSystem trait
use futures_util::stream::StreamExt;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};


#[derive(PartialEq)]
enum MenuState {
    MainMenu,
    ScriptChoice,     // New state for choosing demo or browse
    ScriptBrowser,
    NormalMode,
    HelpMode,
}

pub async fn interactive_menu() {
    
    let mut scancodes = keyboard::ScancodeStream::new();
    let mut kb = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    
    let mut menu_state = MenuState::MainMenu;
    let mut selected_script_index: usize = 0;
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
                                    println!("\n→ Continuing with normal operation...\n");
                                    show_system_info();
                                    println!("\nNow in normal operation mode.");
                                    println!("Type characters to see them echoed, or press ESC to return to menu:");
                                    menu_state = MenuState::NormalMode;
                                }
                                '2' => {
                                    println!("\n→ RustrialScript Options...\n");
                                    use crate::vga_buffer::clear_screen;
                                    clear_screen();
                                    show_script_choice();
                                    menu_state = MenuState::ScriptChoice;
                                }
                                '3' => {
                                    println!("\n→ Showing Help...\n");
                                    show_help();
                                    println!("\nPress any key to return to menu...");
                                    menu_state = MenuState::HelpMode;
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
                                    show_script_browser(selected_script_index);
                                    menu_state = MenuState::ScriptBrowser;
                                }
                                _ => {
                                    // ESC or other keys return to main menu
                                }
                            }
                        }
                        // Handle ESC key
                        if let DecodedKey::RawKey(KeyCode::Escape) = key {
                            println!("\n→ Returning to main menu...\n");
                            use crate::vga_buffer::clear_screen;
                            clear_screen();
                            show_menu_screen();
                            menu_state = MenuState::MainMenu;
                        }
                    }
                    MenuState::ScriptBrowser => {
                        handle_script_browser_input(key, &mut selected_script_index, &mut menu_state, &mut return_to_browser);
                    }
                    MenuState::HelpMode => {
                        // Any key returns to menu from help
                        if return_to_browser {
                            use crate::vga_buffer::clear_screen;
                            clear_screen();
                            show_script_browser(selected_script_index);
                            menu_state = MenuState::ScriptBrowser;
                            return_to_browser = false;
                        } else {
                            println!("\n\n→ Returning to main menu...\n");
                            use crate::vga_buffer::clear_screen;
                            clear_screen();
                            show_menu_screen();
                            menu_state = MenuState::MainMenu;
                        }
                    }
                    MenuState::NormalMode => {
                        match key {
                            DecodedKey::RawKey(KeyCode::Escape) | DecodedKey::Unicode('\x1b') => {
                                println!("\n\n→ Returning to main menu...\n");
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
}


fn show_menu_screen() {
    println!("\n╔═════════════════════════════════════════╗");
    println!("║         RustrialOS - Main Menu            ║");
    println!("╚═══════════════════════════════════════════╝");
    println!();
    println!("  [1] Continue with normal operation");
    println!("  [2] RustrialScript");
    println!("  [3] Show Help");
    println!();
    println!("Press a number key (1-3) to select...");
    println!("(Press ESC anytime in normal mode to return to this menu)");
}

fn show_script_choice() {
    println!("\n╔═════════════════════════════════════════╗");
    println!("║         RustrialScript Options            ║");
    println!("╚═══════════════════════════════════════════╝");
    println!();
    println!("  [1] Run Demo (Fibonacci)");
    println!("  [2] Browse & Run Scripts from Filesystem");
    println!();
    println!("Press 1 or 2 to select, or ESC to return...");
}

fn show_script_browser(selected_index: usize) {
    println!("╔══════════════════════════════════════════════╗");
    println!("║          Script Browser - /scripts/         ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();
    
    // Get list of scripts from filesystem
    if let Some(fs) = crate::fs::root_fs() {
        let fs = fs.lock();
        match fs.list_dir("/scripts") {
            Ok(scripts) => {
                if scripts.is_empty() {
                    println!("  No scripts found in /scripts directory");
                } else {
                    println!("  Available scripts:");
                    println!();
                    for (i, script_path) in scripts.iter().enumerate() {
                        // Extract just the filename from the full path
                        let filename = script_path.trim_start_matches("/scripts/");
                        if i == selected_index {
                            println!("  → [{}] {}", i + 1, filename);
                        } else {
                            println!("    [{}] {}", i + 1, filename);
                        }
                    }
                }
            }
            Err(e) => {
                println!("  Error reading scripts directory: {}", e);
            }
        }
    } else {
        println!("  Error: Filesystem not initialized");
    }
    
    println!();
    println!("═══ Controls ═══");
    println!("  ↑/↓ or W/S - Navigate");
    println!("  Enter      - Run selected script");
    println!("  ESC        - Return to main menu");
}

fn handle_script_browser_input(
    key: DecodedKey,
    selected_index: &mut usize,
    menu_state: &mut MenuState,
    return_to_browser: &mut bool,
) {
    match key {
        DecodedKey::RawKey(KeyCode::Escape) | DecodedKey::Unicode('\x1b') => {
            println!("\n→ Returning to script options...\n");
            use crate::vga_buffer::clear_screen;
            clear_screen();
            show_script_choice();
            *menu_state = MenuState::ScriptChoice;
        }
        DecodedKey::RawKey(KeyCode::ArrowUp) | DecodedKey::Unicode('w') | DecodedKey::Unicode('W') => {
            if *selected_index == 0 {
                return;
            }

            let can_navigate = crate::fs::root_fs()
                .and_then(|fs| {
                    let fs_guard = fs.lock();
                    fs_guard.list_dir("/scripts").ok()
                })
                .map(|scripts| !scripts.is_empty())
                .unwrap_or(false);

            if can_navigate {
                *selected_index -= 1;
                use crate::vga_buffer::clear_screen;
                clear_screen();
                show_script_browser(*selected_index);
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
                use crate::vga_buffer::clear_screen;
                clear_screen();
                show_script_browser(*selected_index);
            }
        }
        DecodedKey::Unicode('\n') | DecodedKey::RawKey(KeyCode::Return) => {
            // Run the selected script
            run_selected_script(*selected_index);
            println!("\nPress any key to return to script browser...");
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
    println!("=== System Information ===\n");
    
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));
    
    println!("\nasync number: 42");
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
    println!("╔════════════════════════════════════════════════╗");
    println!("║              RustrialOS - Help                ║");
    println!("╚════════════════════════════════════════════════╝");
    println!();
    println!("RustrialOS is a hobby x86-64 bare-metal operating system");
    println!("written in Rust, featuring:");
    println!();
    println!("• Custom memory management and heap allocation");
    println!("• Interrupt handling and async task execution");
    println!("• In-memory filesystem (RAMfs) support");
    println!("• RustrialScript: A minimal stack-based interpreter");
    println!("• Interactive menu and script browser system");
    println!();
    println!("═══ Menu Options ═══");
    println!();
    println!("[1] Normal Operation");
    println!("    - Shows system information (heap, memory)");
    println!("    - Enters keyboard echo mode");
    println!();
    println!("[2] RustrialScript");
    println!("    - Run a Fibonacci demo");
    println!("    - Browse and run scripts from the filesystem");
    println!("    - 6 example scripts available");
    println!();
    println!("[3] Help (this screen)");
    println!();
    println!("═══ Keyboard Commands ═══");
    println!();
    println!("  ESC       - Return to previous menu");
    println!("  Backspace - Delete last character");
    println!("  ↑/↓ or W/S - Navigate in script browser");
    println!("  Enter     - Run selected script");
    println!();
    println!("For more information, see the documentation at:");
    println!("github.com/lazzerex/rustrial-os");
}
