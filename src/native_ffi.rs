/// FFI Bindings for Native C/Assembly Code
/// 
/// This module provides Rust interfaces to the C and Assembly implementations
/// of CPU detection, PCI enumeration, and RTC access.

use core::fmt;
use x86_64::PhysAddr;
use x86_64::{
    VirtAddr,
    structures::paging::{
        Page, PhysFrame, PageTableFlags, Size4KiB, 
        Mapper, FrameAllocator, mapper::MapToError
    },
};

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
    // pci config access
    fn pci_read_config(bus: u8, slot: u8, func: u8, offset: u8) -> u32;
    fn pci_write_config(bus: u8, slot: u8, func: u8, offset: u8, value: u32);
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

/// get bar information for device with a proper size detection hopefully 
pub fn pci_get_bar(device: &PciDevice, bar_index: u8) -> Option<PciBar> {
    if bar_index as usize >= device.bar.len() { 
        return None; 
    }
    
    let raw = device.bar[bar_index as usize];
    if raw == 0 { 
        return None; 
    }
    
    // probe the actual size
    let size = pci_probe_bar_size(device, bar_index);
    
    // determine type and extract base address
    if (raw & 0x1) == 1 {
        // I/O Space BAR
        let base = (raw & 0xFFFFFFFC) as u64;
        Some(PciBar { 
            base_addr: PhysAddr::new(base), 
            size, 
            is_mmio: false 
        })
    } else {
        // Memory Space BAR
        let base = (raw & 0xFFFFFFF0) as u64;
        
        // do chheck for 64-bit BAR (bits 2:1 == 0b10)
        let bar_type = (raw >> 1) & 0x3;
        if bar_type == 2 && bar_index < 5 {
            // 64-bit BAR, read upper 32 bits from next BAR
            let upper = device.bar[bar_index as usize + 1] as u64;
            let base_64 = base | (upper << 32);
            Some(PciBar { 
                base_addr: PhysAddr::new(base_64), 
                size, 
                is_mmio: true 
            })
        } else {
            Some(PciBar { 
                base_addr: PhysAddr::new(base), 
                size, 
                is_mmio: true 
            })
        }
    }
}

/// enable pci device memory space and bus mastering by setting command register
pub fn pci_enable_bus_mastering(device: &PciDevice) {
    // command register at offset 0x04: bit 1 = memory space, bit 2 = bus master
    let mut cmd = pci_read_config_dword(device.bus, device.device, device.function, 0x04);
    cmd |= 0x6; // set bits 1 (mem space) and 2 (bus master)
    pci_write_config_dword(device.bus, device.device, device.function, 0x04, cmd);
}

/// probe bar size by writing 0xFFFFFFFF and reading back mask. returns size in bytes (0 if unknown)
pub fn pci_probe_bar_size(device: &PciDevice, bar_index: u8) -> usize {
    if bar_index as usize >= device.bar.len() { return 0; }
    let offset = 0x10 + (bar_index as u8 * 4);
    let orig = pci_read_config_dword(device.bus, device.device, device.function, offset as u8);
    // write all ones
    pci_write_config_dword(device.bus, device.device, device.function, offset as u8, 0xFFFF_FFFF);
    let mask = pci_read_config_dword(device.bus, device.device, device.function, offset as u8);
    // restore original
    pci_write_config_dword(device.bus, device.device, device.function, offset as u8, orig);

    if mask == 0 || mask == 0xFFFF_FFFF { return 0; }

    // if io space (bit 0 == 1)
    if (orig & 0x1) == 1 {
        let masked = mask & 0xFFFFFFFC;
        let size = (!(masked) as u64).wrapping_add(1);
        return size as usize;
    } else {
        // memory space, mask lower 4 bits
        let masked = mask & 0xFFFF_FFF0;
        let size = (!(masked) as u64).wrapping_add(1);
        return size as usize;
    }
}

/// convert a physical pci bar address to kernel virtual using given physical_memory_offset
/// does not create table entries
pub fn pci_bar_phys_to_virt(phys: PhysAddr, physical_memory_offset: u64) -> x86_64::VirtAddr {
    x86_64::VirtAddr::new(physical_memory_offset + phys.as_u64())
}

//this will create page table entries
pub fn map_mmio_range<A>(
    phys_start: PhysAddr,
    size: usize,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut A,
) -> Result<VirtAddr, MapToError<Size4KiB>>
where
    A: FrameAllocator<Size4KiB>,
{
    // MMIO regions need specific flags
    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::NO_CACHE          // disable caching for MMIO
        | PageTableFlags::WRITE_THROUGH;    // write-through for MMIO
    
    // calculate how many pages we would need
    let page_count = (size + 0xFFF) / 0x1000;
    
    
    // need to track this in a global allocator
    let virt_start = VirtAddr::new(0xFFFF_FF00_0000_0000 + phys_start.as_u64());
    
    for i in 0..page_count {
        let page: Page = Page::containing_address(virt_start + i as u64 * 4096);
        let frame = PhysFrame::containing_address(phys_start + i as u64 * 4096);
        
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)?
                .flush();
        }
    }
    
    Ok(virt_start)
}

/// convenience function: Get BAR and map it in one call
pub fn get_and_map_bar<A>(
    device: &PciDevice,
    bar_index: u8,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut A,
) -> Result<(PciBar, VirtAddr), &'static str>
where
    A: FrameAllocator<Size4KiB>,
{
    let bar = pci_get_bar(device, bar_index)
        .ok_or("BAR not found or invalid")?;
    
    if !bar.is_mmio {
        return Err("Cannot map I/O space BAR (use port I/O instead)");
    }
    
    if bar.size == 0 {
        return Err("BAR has zero size");
    }
    
    let virt_addr = map_mmio_range(bar.base_addr, bar.size, mapper, frame_allocator)
        .map_err(|_| "Failed to map BAR to virtual memory")?;
    
    Ok((bar, virt_addr))
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

impl PciDevice {
    /// het the IRQ line this device uses
    pub fn get_irq(&self) -> Option<u8> {
        if self.interrupt_line == 0xFF || self.interrupt_line == 0 {
            None // no IRQ assigned
        } else {
            Some(self.interrupt_line)
        }
    }
    
    /// register an IRQ handler for this device
    pub fn register_irq_handler(&self, handler: fn()) -> Result<(), &'static str> {
        let irq = self.get_irq().ok_or("No IRQ assigned to device")?;
        if irq >= 16 {
            return Err("IRQ number out of range for PIC (need APIC/MSI)");
        }
        crate::interrupts::register_irq_handler(irq, handler);
        Ok(())
    }
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
