/// FFI Bindings for Native C/Assembly Code
/// 
/// This module provides Rust interfaces to the C and Assembly implementations
/// of CPU detection, PCI enumeration, and RTC access.

use core::fmt;
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
        core::str::from_utf8(&self.vendor).unwrap_or("Unknown")
    }
    
    pub fn brand_str(&self) -> &str {
        let s = core::str::from_utf8(&self.brand).unwrap_or("Unknown CPU");
        s.trim_end_matches('\0').trim()
    }
    
    pub fn has_sse(&self) -> bool { (self.features & (1 << 25)) != 0 }
    pub fn has_sse2(&self) -> bool { (self.features & (1 << 26)) != 0 }
    pub fn has_sse3(&self) -> bool { (self.features & (1 << 32)) != 0 }
    pub fn has_avx(&self) -> bool { (self.features & (1 << 60)) != 0 }
}

impl fmt::Display for CpuInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "CPU Vendor: {}", self.vendor_str())?;
        writeln!(f, "CPU Brand:  {}", self.brand_str())?;
        write!(f, "Features:   ")?;
        
        let mut first = true;
        for (feature, name) in [
            (self.has_sse(), "SSE"),
            (self.has_sse2(), "SSE2"),
            (self.has_sse3(), "SSE3"),
            (self.has_avx(), "AVX"),
        ] {
            if feature {
                if !first { write!(f, ", ")?; }
                write!(f, "{}", name)?;
                first = false;
            }
        }
        
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
}

impl PciDevice {
    pub fn class_name(&self) -> &'static str {
        unsafe {
            let ptr = pci_get_class_name(self.class_code);
            let bytes = core::slice::from_raw_parts(ptr, 64);
            let len = bytes.iter().position(|&b| b == 0).unwrap_or(64);
            core::str::from_utf8(&bytes[..len]).unwrap_or("Unknown")
        }
    }
    
    pub fn vendor_name(&self) -> &'static str {
        unsafe {
            let ptr = pci_get_vendor_name(self.vendor_id);
            let bytes = core::slice::from_raw_parts(ptr, 64);
            let len = bytes.iter().position(|&b| b == 0).unwrap_or(64);
            core::str::from_utf8(&bytes[..len]).unwrap_or("Unknown")
        }
    }
}

impl fmt::Display for PciDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:02X}:{:02X}.{}] {:04X}:{:04X} {} - {}",
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
    devices
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
        dt
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
