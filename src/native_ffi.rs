/// FFI Bindings for Native C/Assembly Code
/// 
/// This module provides Rust interfaces to the C and Assembly implementations
/// of CPU detection, PCI enumeration, and RTC access.

use core::fmt;
use x86_64::PhysAddr;
extern crate alloc;
use alloc::vec::Vec;

// ============================================================================
// CPU Detection (Assembly + Rust wrapper)
// ============================================================================

unsafe extern "C" {
    fn cpu_get_vendor(buffer: *mut u8);
    fn cpu_get_features() -> u64;
    fn cpu_has_sse2() -> bool;
    fn cpu_has_avx() -> bool;
    fn cpu_get_brand(buffer: *mut u8);
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub vendor: [u8; 12],
    pub brand: [u8; 48],
    pub features: u64,
}

impl CpuInfo {
    pub fn get() -> Self {
        let mut vendor = [0u8; 12];
        let mut brand = [0u8; 48];
        let features = unsafe {
            cpu_get_vendor(vendor.as_mut_ptr());
            cpu_get_brand(brand.as_mut_ptr());
            cpu_get_features()
        };
        
        CpuInfo { vendor, brand, features }
    }
    
    pub fn vendor_str(&self) -> &str {
        let vendor = core::str::from_utf8(&self.vendor).unwrap_or("Unknown");
        if vendor.is_empty() || vendor.chars().all(|c| c == '\0') {
            "Unknown"
        } else {
            vendor.trim_end_matches('\0').trim()
        }
    }
    
    pub fn brand_str(&self) -> &str {
        let brand = core::str::from_utf8(&self.brand).unwrap_or("Unknown CPU");
        let trimmed = brand.trim_end_matches('\0').trim();
        if trimmed.is_empty() {
            "Unknown CPU"
        } else {
            trimmed
        }
    }
    
    pub fn has_sse(&self) -> bool { (self.features & (1 << 25)) != 0 }
    pub fn has_sse2(&self) -> bool { (self.features & (1 << 26)) != 0 }
    pub fn has_sse3(&self) -> bool { (self.features & (1 << 0)) != 0 } // ecx bit 0
    pub fn has_avx(&self) -> bool { (self.features & (1 << 28)) != 0 } // ecx bit 28
    pub fn has_fpu(&self) -> bool { (self.features & (1 << 0)) != 0 }
    pub fn has_mmx(&self) -> bool { (self.features & (1 << 23)) != 0 }
    pub fn has_msr(&self) -> bool { (self.features & (1 << 5)) != 0 }
    
    pub fn features_str(&self) -> alloc::string::String {
        use alloc::string::String;
        use alloc::vec::Vec;
        
        let mut features = Vec::new();
        if self.has_fpu() { features.push("FPU"); }
        if self.has_msr() { features.push("MSR"); }
        if self.has_mmx() { features.push("MMX"); }
        if self.has_sse() { features.push("SSE"); }
        if self.has_sse2() { features.push("SSE2"); }
        if self.has_sse3() { features.push("SSE3"); }
        if self.has_avx() { features.push("AVX"); }
        
        if features.is_empty() {
            String::from("None detected")
        } else {
            features.join(", ")
        }
    }
}

impl fmt::Display for CpuInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "CPU Vendor:   {}", self.vendor_str())?;
        writeln!(f, "CPU Brand:    {}", self.brand_str())?;
        write!(f, "Features:     {}", self.features_str())?;
        Ok(())
    }
}

pub fn print_cpu_info() {
    let info = CpuInfo::get();
    crate::print!("{}", info);
}

pub fn has_sse2_native() -> bool {
    unsafe { cpu_has_sse2() }
}

pub fn has_avx_native() -> bool {
    unsafe { cpu_has_avx() }
}

// ============================================================================
// PCI Device Enumeration (C)
// ============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    pub header_type: u8,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub bar: [u32; 6],
}

unsafe extern "C" {
    fn pci_enumerate_devices(devices: *mut PciDevice, max_devices: i32) -> i32;
    fn pci_get_class_name(class_code: u8) -> *const u8;
    fn pci_get_vendor_name(vendor_id: u16) -> *const u8;
    // PCI config access
    fn pci_read_config(bus: u8, slot: u8, func: u8, offset: u8) -> u32;
    fn pci_write_config(bus: u8, slot: u8, func: u8, offset: u8, value: u32);
    fn pci_read_config16(bus: u8, device: u8, function: u8, offset: u8) -> u16;
    fn pci_write_config16(bus: u8, device: u8, function: u8, offset: u8, value: u16);
    // Stage 1.2: Enhanced PCI functions
    fn pci_enable_bus_mastering(bus: u8, device: u8, function: u8);
    fn pci_enable_memory_space(bus: u8, device: u8, function: u8);
    fn pci_enable_io_space(bus: u8, device: u8, function: u8);
    fn pci_get_bar_size(bus: u8, device: u8, function: u8, bar_index: u8) -> u32;
}

impl PciDevice {
    pub fn class_name(&self) -> &'static str {
        unsafe {
            let ptr = pci_get_class_name(self.class_code);
            if ptr.is_null() {
                return "Unknown Class";
            }
            let bytes = core::slice::from_raw_parts(ptr, 64);
            let len = bytes.iter().position(|&b| b == 0).unwrap_or(64);
            let name = core::str::from_utf8(&bytes[..len]).unwrap_or("Unknown Class");
            if name.is_empty() { "Unknown Class" } else { name }
        }
    }
    
    pub fn vendor_name(&self) -> &'static str {
        unsafe {
            let ptr = pci_get_vendor_name(self.vendor_id);
            if ptr.is_null() {
                return "Unknown Vendor";
            }
            let bytes = core::slice::from_raw_parts(ptr, 64);
            let len = bytes.iter().position(|&b| b == 0).unwrap_or(64);
            let name = core::str::from_utf8(&bytes[..len]).unwrap_or("Unknown Vendor");
            if name.is_empty() { "Unknown Vendor" } else { name }
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.vendor_id != 0xFFFF && self.vendor_id != 0x0000
    }
}

impl fmt::Display for PciDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:02X}:{:02X}.{}] {:04X}:{:04X} {:16} - {}",
            self.bus, self.device, self.function,
            self.vendor_id, self.device_id,
            self.vendor_name(),
            self.class_name()
        )
    }
}

pub fn enumerate_pci_devices() -> Vec<PciDevice> {
    const MAX_DEVICES: usize = 256;
    let mut devices = Vec::with_capacity(MAX_DEVICES);
    unsafe {
        devices.set_len(MAX_DEVICES);
        let count = pci_enumerate_devices(devices.as_mut_ptr(), MAX_DEVICES as i32);
        devices.set_len(count as usize);
    }
    // filter out invalid devices
    devices.into_iter().filter(|d| d.is_valid()).collect()
}

/// representation of a pci bar returned to rust
#[derive(Debug, Clone, Copy)]
pub struct PciBar {
    pub base_addr: PhysAddr,
    pub size: usize,
    pub is_mmio: bool,
}

/// read raw pci config dword via ffi wrapper
pub fn pci_read_config_dword(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    unsafe { pci_read_config(bus, slot, func, offset) }
}

/// write raw pci config dword via ffi wrapper
pub fn pci_write_config_dword(bus: u8, slot: u8, func: u8, offset: u8, value: u32) {
    unsafe { pci_write_config(bus, slot, func, offset, value) }
}

/// Stage 1.2: Get BAR information with size detection
/// Maps device MMIO/IO regions to virtual memory
pub fn pci_get_bar(device: &PciDevice, bar_index: u8) -> Option<PciBar> {
    if bar_index as usize >= device.bar.len() { 
        return None; 
    }
    
    let raw = device.bar[bar_index as usize];
    if raw == 0 { 
        return None; 
    }
    
    // Get actual size by probing the BAR
    let size = unsafe {
        pci_get_bar_size(device.bus, device.device, device.function, bar_index)
    } as usize;
    
    // Determine if I/O space or memory space
    if (raw & 0x1) == 1 {
        // I/O space BAR
        let base = (raw & 0xFFFFFFFC) as u64;
        Some(PciBar { 
            base_addr: PhysAddr::new(base), 
            size, 
            is_mmio: false 
        })
    } else {
        // Memory space BAR
        let base = (raw & 0xFFFFFFF0) as u64;
        Some(PciBar { 
            base_addr: PhysAddr::new(base), 
            size, 
            is_mmio: true 
        })
    }
}

/// Stage 1.2: Enable PCI bus mastering for DMA operations
pub fn pci_enable_dma(device: &PciDevice) {
    unsafe {
        pci_enable_bus_mastering(device.bus, device.device, device.function);
        pci_enable_memory_space(device.bus, device.device, device.function);
    }
}

/// Stage 1.2: Enable memory-mapped I/O for device
pub fn pci_enable_mmio(device: &PciDevice) {
    unsafe {
        pci_enable_memory_space(device.bus, device.device, device.function);
    }
}

/// Stage 1.2: Enable port I/O for device
pub fn pci_enable_io(device: &PciDevice) {
    unsafe {
        pci_enable_io_space(device.bus, device.device, device.function);
    }
}

/// Stage 1.2: Read interrupt line configuration
pub fn pci_get_interrupt_line(device: &PciDevice) -> u8 {
    device.interrupt_line
}

/// Stage 1.2: Read interrupt pin (which pin device uses: INTA-INTD)
pub fn pci_get_interrupt_pin(device: &PciDevice) -> u8 {
    device.interrupt_pin
}

pub fn print_pci_devices() {
    let devices = enumerate_pci_devices();
    if devices.is_empty() {
        crate::println!("  │ No PCI devices found.");
        return;
    }
    
    for device in &devices {
        crate::println!("  │ {}", device);
    }
    
    crate::println!("  │");
    crate::println!("  │ Total devices found: {}", devices.len());
}

// ============================================================================
// RTC (C)
// ============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub weekday: u8,
}

unsafe extern "C" {
    fn rtc_read_datetime(dt: *mut DateTime);
    fn rtc_weekday_str(weekday: u8) -> *const u8;
    fn rtc_month_str(month: u8) -> *const u8;
}

impl DateTime {
    pub fn read() -> Self {
        let mut dt = DateTime {
            year: 0, month: 0, day: 0,
            hour: 0, minute: 0, second: 0, weekday: 0,
        };
        unsafe {
            rtc_read_datetime(&mut dt as *mut DateTime);
        }
        dt.validate()
    }
    
    fn validate(mut self) -> Self {
        // validate and clamp values to reasonable ranges
        if self.year < 1970 || self.year > 2100 {
            self.year = 2026; // default to current year
        }
        if self.month < 1 || self.month > 12 {
            self.month = 1;
        }
        if self.day < 1 || self.day > 31 {
            self.day = 1;
        }
        if self.hour > 23 {
            self.hour = 0;
        }
        if self.minute > 59 {
            self.minute = 0;
        }
        if self.second > 59 {
            self.second = 0;
        }
        if self.weekday > 7 {
            self.weekday = 0;
        }
        self
    }
    
    pub fn weekday_str(&self) -> &'static str {
        unsafe {
            let ptr = rtc_weekday_str(self.weekday);
            let bytes = core::slice::from_raw_parts(ptr, 32);
            let len = bytes.iter().position(|&b| b == 0).unwrap_or(32);
            core::str::from_utf8(&bytes[..len]).unwrap_or("Unknown")
        }
    }
    
    pub fn month_str(&self) -> &'static str {
        unsafe {
            let ptr = rtc_month_str(self.month);
            let bytes = core::slice::from_raw_parts(ptr, 32);
            let len = bytes.iter().position(|&b| b == 0).unwrap_or(32);
            core::str::from_utf8(&bytes[..len]).unwrap_or("Unknown")
        }
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {} {:02}, {:04} - {:02}:{:02}:{:02}",
            self.weekday_str(),
            self.month_str(),
            self.day,
            self.year,
            self.hour,
            self.minute,
            self.second
        )
    }
}

pub fn print_datetime() {
    let dt = DateTime::read();
    crate::print!("{}", dt);
}
