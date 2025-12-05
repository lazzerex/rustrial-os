pub mod hardware_menu;
pub mod script_menu;

// Re-export hardware menu functions for convenience
pub use hardware_menu::{
    show_hardware_submenu,
    show_all_hardware_info,
    show_cpu_info,
    show_rtc_info,
    show_pci_info,
};

pub use script_menu::{
    show_script_choice,
    show_script_browser,
    handle_script_browser_input,
    run_selected_script,
    run_demo,
};
