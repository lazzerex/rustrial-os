/// Limine bootloader integration for Rustrial OS
/// This module provides the Limine boot protocol support for creating bootable ISOs

// Limine protocol magic numbers and structures
const LIMINE_COMMON_MAGIC: [u64; 2] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b];

#[repr(C)]
struct LimineBaseRevision {
    id: [u64; 2],
    revision: u64,
}

unsafe impl Sync for LimineBaseRevision {}

// Base revision - must be present for Limine to recognize the kernel
#[used]
#[unsafe(link_section = ".limine_reqs")]
static BASE_REVISION: LimineBaseRevision = LimineBaseRevision {
    id: [LIMINE_COMMON_MAGIC[0], LIMINE_COMMON_MAGIC[1]],
    revision: 0,
};

// HHDM (Higher Half Direct Map) request
#[repr(C)]
struct LimineHhdmRequest {
    id: [u64; 4],
    revision: u64,
    response: *mut LimineHhdmResponse,
}

#[repr(C)]
struct LimineHhdmResponse {
    revision: u64,
    offset: u64,
}

unsafe impl Sync for LimineHhdmRequest {}

#[used]
#[unsafe(link_section = ".limine_reqs")]
static mut HHDM_REQUEST: LimineHhdmRequest = LimineHhdmRequest {
    id: [
        LIMINE_COMMON_MAGIC[0],
        LIMINE_COMMON_MAGIC[1],
        0x48dcf1cb8ad2b852,
        0x63984e959a98244b,
    ],
    revision: 0,
    response: core::ptr::null_mut(),
};

// Memory map request
#[repr(C)]
struct LimineMemoryMapRequest {
    id: [u64; 4],
    revision: u64,
    response: *mut LimineMemoryMapResponse,
}

#[repr(C)]
struct LimineMemoryMapResponse {
    revision: u64,
    entry_count: u64,
    entries: *const *const LimineMemoryMapEntry,
}

#[repr(C)]
pub struct LimineMemoryMapEntry {
    pub base: u64,
    pub length: u64,
    pub entry_type: u64,
}

unsafe impl Sync for LimineMemoryMapRequest {}

#[used]
#[unsafe(link_section = ".limine_reqs")]
static mut MEMORY_MAP_REQUEST: LimineMemoryMapRequest = LimineMemoryMapRequest {
    id: [
        LIMINE_COMMON_MAGIC[0],
        LIMINE_COMMON_MAGIC[1],
        0x67cf3d9d378a806f,
        0xe304acdfc50c3c62,
    ],
    revision: 0,
    response: core::ptr::null_mut(),
};

// Terminal request - for text output via Limine's terminal
#[repr(C)]
struct LimineTerminalRequest {
    id: [u64; 4],
    revision: u64,
    response: *mut LimineTerminalResponse,
}

#[repr(C)]
struct LimineTerminalResponse {
    revision: u64,
    terminal_count: u64,
    terminals: *const *const LimineTerminal,
}

#[repr(C)]
pub struct LimineTerminal {
    columns: u64,
    rows: u64,
    framebuffer: *mut u8,
}

unsafe impl Sync for LimineTerminalRequest {}

#[used]
#[unsafe(link_section = ".limine_reqs")]
static mut TERMINAL_REQUEST: LimineTerminalRequest = LimineTerminalRequest {
    id: [
        LIMINE_COMMON_MAGIC[0],
        LIMINE_COMMON_MAGIC[1],
        0xc8ac59310c2b0844,
        0xa68d0c7265d38878,
    ],
    revision: 0,
    response: core::ptr::null_mut(),
};

// Terminal write callback type
type TerminalWriteFn = unsafe extern "C" fn(*const LimineTerminal, *const u8, u64);

/// Boot info structure for Limine
pub struct LimineBootInfo {
    pub hhdm_offset: Option<u64>,
    pub memory_entries: Option<&'static [&'static LimineMemoryMapEntry]>,
}

impl LimineBootInfo {
    /// Get the boot information from Limine
    pub fn get() -> Self {
        let hhdm_offset = unsafe {
            let response = HHDM_REQUEST.response;
            if !response.is_null() {
                Some((*response).offset)
            } else {
                None
            }
        };

        let memory_entries = unsafe {
            let response = MEMORY_MAP_REQUEST.response;
            if !response.is_null() {
                let count = (*response).entry_count as usize;
                let entries_ptr = (*response).entries;
                if !entries_ptr.is_null() && count > 0 {
                    Some(core::slice::from_raw_parts(entries_ptr as *const &'static LimineMemoryMapEntry, count))
                } else {
                    None
                }
            } else {
                None
            }
        };

        Self {
            hhdm_offset,
            memory_entries,
        }
    }

    /// Get the physical memory offset for converting physical to virtual addresses
    pub fn physical_memory_offset(&self) -> Option<u64> {
        self.hhdm_offset
    }
    
    /// Write a string to the Limine terminal (if available)
    pub fn terminal_write(s: &str) {
        unsafe {
            let response = TERMINAL_REQUEST.response;
            if response.is_null() {
                return;
            }
            
            let term_response = &*response;
            if term_response.terminal_count == 0 || term_response.terminals.is_null() {
                return;
            }
            
            // Get the first terminal
            let terminals = core::slice::from_raw_parts(
                term_response.terminals,
                term_response.terminal_count as usize
            );
            
            if terminals.is_empty() || terminals[0].is_null() {
                return;
            }
            
            // TODO: Properly get the write callback from Limine terminal
            // For now, just skip terminal output
        }
    }

    /// Get memory regions as an iterator
    pub fn memory_regions(&self) -> Option<impl Iterator<Item = MemoryEntry> + '_> {
        self.memory_entries.map(|entries| {
            entries.iter().map(|e| MemoryEntry {
                base: e.base,
                length: e.length,
                entry_type: match e.entry_type {
                    0 => MemoryEntryType::USABLE,
                    1 => MemoryEntryType::RESERVED,
                    2 => MemoryEntryType::ACPI_RECLAIMABLE,
                    3 => MemoryEntryType::ACPI_NVS,
                    4 => MemoryEntryType::BAD_MEMORY,
                    5 => MemoryEntryType::BOOTLOADER_RECLAIMABLE,
                    6 => MemoryEntryType::KERNEL_AND_MODULES,
                    7 => MemoryEntryType::FRAMEBUFFER,
                    _ => MemoryEntryType::RESERVED,
                },
            })
        })
    }
}

/// Memory map entry (simplified for now)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MemoryEntry {
    pub base: u64,
    pub length: u64,
    pub entry_type: MemoryEntryType,
}

/// Memory entry types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum MemoryEntryType {
    USABLE = 0,
    RESERVED = 1,
    ACPI_RECLAIMABLE = 2,
    ACPI_NVS = 3,
    BAD_MEMORY = 4,
    BOOTLOADER_RECLAIMABLE = 5,
    KERNEL_AND_MODULES = 6,
    FRAMEBUFFER = 7,
}

/// Convert Limine memory map entry type to our MemoryRegionKind
pub fn limine_to_region_kind(entry_type: MemoryEntryType) -> Option<crate::memory::MemoryRegionKind> {
    use crate::memory::MemoryRegionKind;

    #[allow(non_upper_case_globals)]
    match entry_type {
        MemoryEntryType::USABLE => Some(MemoryRegionKind::Usable),
        MemoryEntryType::BOOTLOADER_RECLAIMABLE => Some(MemoryRegionKind::Usable),
        MemoryEntryType::KERNEL_AND_MODULES => Some(MemoryRegionKind::InUse),
        MemoryEntryType::FRAMEBUFFER => Some(MemoryRegionKind::InUse),
        MemoryEntryType::ACPI_RECLAIMABLE => Some(MemoryRegionKind::Reserved),
        MemoryEntryType::ACPI_NVS => Some(MemoryRegionKind::Reserved),
        MemoryEntryType::BAD_MEMORY => Some(MemoryRegionKind::Reserved),
        MemoryEntryType::RESERVED => Some(MemoryRegionKind::Reserved),
    }
}

// TODO: Add actual Limine protocol requests when ready to test with real Limine bootloader
// This placeholder allows the code to compile with --features limine
