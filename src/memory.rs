use x86_64::{
    structures::paging::PageTable,
    VirtAddr,

};

use x86_64::{
    PhysAddr,
    structures::paging::{Page, PhysFrame, Mapper, Size4KiB, FrameAllocator}
};
use x86_64::structures::paging::OffsetPageTable;

#[cfg(feature = "bootloader")]
use bootloader::bootinfo::MemoryMap;
#[cfg(feature = "bootloader")]
use bootloader::bootinfo::MemoryRegionType;

// Phase 1.1: DMA memory management
pub mod dma;


/// returns a mutable reference to the active level 4 table.
///
/// unsafe fn since the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset` (which is usually true for higher half kernels maybe)

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr)
    -> Option<PhysAddr>
{
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr)
    -> Option<PhysAddr>
{
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    //here's where we read the active level 4 frame from the CR3 register
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    for &index in &table_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe {&*table_ptr};

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;
    let map_to_result = unsafe {
        //for test only, will be removed later
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

pub struct EmptyFrameAllocator;
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

/// Memory region kind (unified for both bootloaders)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionKind {
    Usable,
    Reserved,
    InUse,
}

#[cfg(feature = "bootloader")]
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

#[cfg(feature = "limine")]
pub struct BootInfoFrameAllocator {
    memory_regions: alloc::vec::Vec<(u64, u64)>, // (start, end) pairs of usable memory
    next: usize,
}

impl BootInfoFrameAllocator {
    #[cfg(feature = "bootloader")]
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    #[cfg(feature = "limine")]
    pub unsafe fn init_from_limine(boot_info: &crate::limine::LimineBootInfo) -> Option<Self> {
        use alloc::vec::Vec;
        
        let mut memory_regions = Vec::new();
        
        if let Some(entries) = boot_info.memory_regions() {
            for entry in entries {
                if entry.entry_type == crate::limine::MemoryEntryType::USABLE {
                    memory_regions.push((entry.base, entry.base + entry.length));
                }
            }
        }
        
        // For now, if no memory regions, provide a default usable range
        // This will be properly populated when using actual Limine bootloader
        if memory_regions.is_empty() {
            // Default: 16MB to 128MB (placeholder for testing)
            memory_regions.push((0x1000000, 0x8000000));
        }
        
        Some(BootInfoFrameAllocator {
            memory_regions,
            next: 0,
        })
    }

    #[cfg(feature = "bootloader")]
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }

    #[cfg(feature = "limine")]
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        self.memory_regions.iter().flat_map(|(start, end)| {
            (*start..*end).step_by(4096)
                .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
        })
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}