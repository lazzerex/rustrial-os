// ============================================================================
// FILE: src/menu/hardware_menu.rs
// ============================================================================

use crate::graphics::text_graphics::{
    draw_shadow_box, 
    write_centered, 
    draw_hline, 
    write_at,
    draw_filled_box,
};
use crate::graphics::splash::show_status_bar;
use crate::vga_buffer::Color;
use alloc::format;
use alloc::vec::Vec;

/// Display the hardware submenu with options
pub fn show_hardware_submenu() {
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
        ("[0]", "Return to Desktop", "Exit hardware info menu"),
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

    show_status_bar("Press 1-4 to select  • 0 returns to desktop");
}

/// Display all hardware information in one comprehensive view
pub fn show_all_hardware_info() {
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
    write_at(BOX_X + 3, BOX_Y + 5, &format!("Vendor: {}", cpu_info.vendor_str()), Color::White, Color::Black);
    
    let brand = cpu_info.brand_str();
    let brand_display = if brand.len() > 60 { &brand[..60] } else { brand };
    write_at(BOX_X + 3, BOX_Y + 6, &format!("Brand:  {}", brand_display), Color::White, Color::Black);
    
    // RTC Section
    write_at(BOX_X + 2, BOX_Y + 8, "[Real-Time Clock - C RTC Driver]", Color::Magenta, Color::Black);
    let datetime = crate::native_ffi::DateTime::read();
    write_at(BOX_X + 3, BOX_Y + 9, &format!("{}", datetime), Color::LightCyan, Color::Black);
    
    // PCI Section
    write_at(BOX_X + 2, BOX_Y + 11, "[PCI Devices - C PCI Enumeration]", Color::LightGreen, Color::Black);
    let devices = crate::native_ffi::enumerate_pci_devices();
    
    if devices.is_empty() {
        write_at(BOX_X + 3, BOX_Y + 12, "No PCI devices found.", Color::LightRed, Color::Black);
    } else {
        const MAX_VISIBLE: usize = 8;
        for (i, device) in devices.iter().take(MAX_VISIBLE).enumerate() {
            let device_str = format!("{}", device);
            write_at(BOX_X + 3, BOX_Y + 12 + i, &device_str, Color::White, Color::Black);
        }
        
        if devices.len() > MAX_VISIBLE {
            write_at(BOX_X + 3, BOX_Y + 20, &format!("...and {} more devices", devices.len() - MAX_VISIBLE), Color::Yellow, Color::Black);
        }
    }
    
    write_centered(BOX_Y + BOX_HEIGHT - 1, "Press ESC to return to Hardware Menu", Color::LightGray, Color::Black);
}

/// Display detailed CPU information
pub fn show_cpu_info() {
    crate::vga_buffer::clear_screen();
    
    const BOX_X: usize = 5;
    const BOX_Y: usize = 2;
    const BOX_WIDTH: usize = 70;
    const BOX_HEIGHT: usize = 10;
    
    draw_shadow_box(BOX_X, BOX_Y, BOX_WIDTH, BOX_HEIGHT, Color::LightCyan, Color::Black);
    write_centered(BOX_Y + 1, "CPU INFORMATION - Assembly CPUID", Color::Yellow, Color::Black);
    draw_hline(BOX_X + 2, BOX_Y + 2, BOX_WIDTH - 4, Color::Cyan, Color::Black);
    
    let cpu_info = crate::native_ffi::CpuInfo::get();
    write_at(BOX_X + 3, BOX_Y + 4, &format!("CPU Vendor: {}", cpu_info.vendor_str()), Color::White, Color::Black);
    write_at(BOX_X + 3, BOX_Y + 5, &format!("CPU Brand:  {}", cpu_info.brand_str()), Color::White, Color::Black);
    
    let mut features = Vec::new();
    if cpu_info.has_sse() { features.push("SSE"); }
    if cpu_info.has_sse2() { features.push("SSE2"); }
    if cpu_info.has_sse3() { features.push("SSE3"); }
    if cpu_info.has_avx() { features.push("AVX"); }
    
    write_at(BOX_X + 3, BOX_Y + 6, &format!("Features:   {}", features.join(", ")), Color::LightGreen, Color::Black);
    
    write_centered(BOX_Y + BOX_HEIGHT - 1, "Press ESC to return to Hardware Menu", Color::LightGray, Color::Black);
}

/// Display Real-Time Clock information with UTC+7 adjustment
pub fn show_rtc_info() {
    crate::vga_buffer::clear_screen();
    
    const BOX_X: usize = 5;
    const BOX_Y: usize = 2;
    const BOX_WIDTH: usize = 70;
    const BOX_HEIGHT: usize = 8;
    
    draw_shadow_box(BOX_X, BOX_Y, BOX_WIDTH, BOX_HEIGHT, Color::LightCyan, Color::Black);
    write_centered(BOX_Y + 1, "REAL-TIME CLOCK - C RTC Driver", Color::Yellow, Color::Black);
    draw_hline(BOX_X + 2, BOX_Y + 2, BOX_WIDTH - 4, Color::Cyan, Color::Black);
    
    let mut datetime = crate::native_ffi::DateTime::read();
    
    // Full UTC+7 adjustment
    let mut hour = datetime.hour as i16 + 7;
    let mut day = datetime.day as i16;
    let mut month = datetime.month as i16;
    let mut year = datetime.year as i16;
    let mut weekday = datetime.weekday as i16;
    
    let days_in_month = |month: i16, year: i16| -> i16 {
        match month {
            1 => 31,
            2 => if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) { 29 } else { 28 },
            3 => 31,
            4 => 30,
            5 => 31,
            6 => 30,
            7 => 31,
            8 => 31,
            9 => 30,
            10 => 31,
            11 => 30,
            12 => 31,
            _ => 31,
        }
    };
    
    if hour >= 24 {
        hour -= 24;
        day += 1;
        weekday = (weekday + 1) % 7;
        
        if day > days_in_month(month, year) {
            day = 1;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }
    
    datetime.hour = hour as u8;
    datetime.day = day as u8;
    datetime.month = month as u8;
    datetime.year = year as u16;
    datetime.weekday = weekday as u8;
    
    let dt_str = format!("{}", datetime);
    write_at(BOX_X + 3, BOX_Y + 4, &dt_str, Color::LightCyan, Color::Black);
    
    write_centered(BOX_Y + BOX_HEIGHT - 1, "Press 0 to return to Desktop", Color::LightGray, Color::Black);
}

/// Display PCI devices information
pub fn show_pci_info() {
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
        let device_str = format!("{}", device);
        let y_pos = BOX_Y + 4 + i;
        write_at(BOX_X + 3, y_pos, &device_str, Color::White, Color::Black);
    }
    
    let footer_y = BOX_Y + BOX_HEIGHT - 2;
    if total > MAX_VISIBLE {
        write_at(BOX_X + 3, footer_y, &format!("Showing {} of {} devices", display_count, total), Color::Yellow, Color::Black);
    } else {
        write_at(BOX_X + 3, footer_y, &format!("Total: {} devices", total), Color::LightGreen, Color::Black);
    }
    
    write_centered(BOX_Y + BOX_HEIGHT - 1, "Press ESC to return to Hardware Menu", Color::LightGray, Color::Black);
}