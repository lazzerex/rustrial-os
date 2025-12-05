pub mod hardware_menu;

// Re-export hardware menu functions for convenience
pub use hardware_menu::{
    show_hardware_submenu,
    show_all_hardware_info,
    show_cpu_info,
    show_rtc_info,
    show_pci_info,
};