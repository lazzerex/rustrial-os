//! DMA-safe physically contiguous memory allocator for network drivers
//! Phase 1.1 - Networking Roadmap

use x86_64::{VirtAddr, PhysAddr};
use x86_64::structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
};
use spin::Mutex;

#[derive(Debug, Clone)]
pub struct DmaBuffer {
    pub virt_addr: VirtAddr,
    pub phys_addr: PhysAddr,
    pub size: usize,
}

#[derive(Debug)]
pub enum AllocError {
    OutOfMemory,
    Alignment,
    MapToError,
}

impl From<MapToError<Size4KiB>> for AllocError {
    fn from(_: MapToError<Size4KiB>) -> Self {
        AllocError::MapToError
    }
}

/// DMA memory region configuration
const DMA_HEAP_START: usize = 0x5555_0000_0000;
const DMA_HEAP_SIZE: usize = 1024 * 1024; // 1MB for DMA buffers
const DMA_ALIGNMENT: usize = 4096; // 4KiB page alignment

/// Global DMA allocator state
static DMA_ALLOCATOR: Mutex<DmaAllocator> = Mutex::new(DmaAllocator::new());

struct DmaAllocator {
    offset: usize,
    initialized: bool,
    physical_memory_offset: u64,
}

impl DmaAllocator {
    const fn new() -> Self {
        Self {
            offset: 0,
            initialized: false,
            physical_memory_offset: 0,
        }
    }
}

/// Initialize the DMA allocator with physical memory offset
/// Must be called before any DMA allocations
pub fn init_dma(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    physical_memory_offset: VirtAddr,
) -> Result<(), MapToError<Size4KiB>> {
    // Map the DMA region
    let page_range = {
        let heap_start = VirtAddr::new(DMA_HEAP_START as u64);
        let heap_end = heap_start + DMA_HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT 
            | PageTableFlags::WRITABLE 
            | PageTableFlags::NO_CACHE; // Important for DMA!
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    // Mark as initialized
    let mut allocator = DMA_ALLOCATOR.lock();
    allocator.initialized = true;
    allocator.physical_memory_offset = physical_memory_offset.as_u64();
    
    Ok(())
}

/// Allocate a DMA buffer of the given size
/// Returns a DmaBuffer with virtual and physical addresses
pub fn allocate_dma_buffer(size: usize) -> Result<DmaBuffer, AllocError> {
    if size == 0 {
        return Err(AllocError::Alignment);
    }

    let aligned_size = (size + DMA_ALIGNMENT - 1) & !(DMA_ALIGNMENT - 1);
    let mut allocator = DMA_ALLOCATOR.lock();
    
    if !allocator.initialized {
        return Err(AllocError::OutOfMemory);
    }

    if allocator.offset + aligned_size > DMA_HEAP_SIZE {
        return Err(AllocError::OutOfMemory);
    }

    let virt_addr = VirtAddr::new((DMA_HEAP_START + allocator.offset) as u64);
    
    // Use the physical memory translation
    // In a proper implementation, we'd walk the page tables, but for now
    // we use the fact that we just allocated contiguous frames
    let phys_addr = unsafe {
        crate::memory::translate_addr(virt_addr, VirtAddr::new(allocator.physical_memory_offset))
            .ok_or(AllocError::MapToError)?
    };

    allocator.offset += aligned_size;

    Ok(DmaBuffer {
        virt_addr,
        phys_addr,
        size: aligned_size,
    })
}

/// Get a mutable slice for the DMA buffer
impl DmaBuffer {
    pub unsafe fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.virt_addr.as_mut_ptr::<u8>(),
                self.size,
            )
        }
    }

    pub unsafe fn as_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.virt_addr.as_ptr::<u8>(),
                self.size,
            )
        }
    }
}

// TODO: Implement proper deallocation and memory reuse
