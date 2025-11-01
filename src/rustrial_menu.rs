use crate::{print, println};
use crate::task::keyboard;
use futures_util::stream::StreamExt;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};


pub async fn interactive_menu() {
    
    show_menu_screen();
    let mut scancodes = keyboard::ScancodeStream::new();
    let mut kb = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    
    let mut menu_active = true;
    
    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = kb.add_byte(scancode) {
            if let Some(key) = kb.process_keyevent(key_event) {
                if menu_active {
                    if let DecodedKey::Unicode(ch) = key {
                        match ch {
                            '1' => {
                                println!("\n→ Continuing with normal operation...\n");
                                show_system_info();
                                println!("\nNow in normal operation mode. Type characters to see them echoed:");
                                menu_active = false;
                            }
                            '2' => {
                                println!("\n→ Running RustrialScript Demo...\n");
                                run_demo();
                                println!("\n→ Demo complete! Now in normal operation mode.");
                                println!("Type characters to see them echoed:");
                                menu_active = false;
                            }
                            _ => {
                                // ignore other keys in menu mode
                            }
                        }
                    }
                } else {
                    match key {
                        DecodedKey::Unicode(character) => print!("{}", character),
                        DecodedKey::RawKey(k) => print!("{:?}", k),
                    }
                }
            }
        }
    }
}


fn show_menu_screen() {
    println!("\n╔════════════════════════════════════════════════╗");
    println!("║         RustrialOS - Main Menu                ║");
    println!("╚════════════════════════════════════════════════╝");
    println!();
    println!("  [1] Continue with normal operation");
    println!("  [2] Run RustrialScript Demo");
    println!();
    println!("Press a number key (1 or 2) to select...");
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
