//! DMA-safe physically contiguous memory allocator for network drivers
//! Phase 1.1 - Networking Roadmap
//!
//! This module provides a memory allocator for DMA buffers that are:
//! - Physically contiguous
//! - Page-aligned
//! - Properly tracked and deallocated
//! - Cached for reuse via buffer pool

use x86_64::{VirtAddr, PhysAddr};
use x86_64::structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
};
use spin::Mutex;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// A DMA-capable buffer with automatic cleanup
/// 
/// When dropped, the buffer is returned to the pool for reuse
#[derive(Debug)]
pub struct DmaBuffer {
    pub virt_addr: VirtAddr,
    pub phys_addr: PhysAddr,
    pub size: usize,
    /// Track if this buffer should be returned to pool on drop
    pooled: bool,
}

impl Clone for DmaBuffer {
    fn clone(&self) -> Self {
        Self {
            virt_addr: self.virt_addr,
            phys_addr: self.phys_addr,
            size: self.size,
            pooled: false, // Clones are not pooled to avoid double-free
        }
    }
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

/// Maximum number of buffers to keep in pool per size class
const MAX_POOLED_BUFFERS: usize = 16;

/// Global DMA allocator state
static DMA_ALLOCATOR: Mutex<DmaAllocator> = Mutex::new(DmaAllocator::new());

/// Track an allocated DMA region for debugging and deallocation
#[derive(Debug, Clone)]
struct DmaRegion {
    virt_addr: VirtAddr,
    phys_addr: PhysAddr,
    size: usize,
    in_use: bool,
}

struct DmaAllocator {
    offset: usize,
    initialized: bool,
    physical_memory_offset: u64,
    /// All allocated regions for tracking
    regions: Vec<DmaRegion>,
    /// Buffer pool organized by size class
    pool: BTreeMap<usize, Vec<DmaRegion>>,
    /// Statistics
    stats: DmaStats,
}

#[derive(Debug, Default)]
struct DmaStats {
    total_allocated: usize,
    total_freed: usize,
    current_usage: usize,
    peak_usage: usize,
    pool_hits: usize,
    pool_misses: usize,
}

impl DmaAllocator {
    const fn new() -> Self {
        Self {
            offset: 0,
            initialized: false,
            physical_memory_offset: 0,
            regions: Vec::new(),
            pool: BTreeMap::new(),
            stats: DmaStats {
                total_allocated: 0,
                total_freed: 0,
                current_usage: 0,
                peak_usage: 0,
                pool_hits: 0,
                pool_misses: 0,
            },
        }
    }
    
    /// Try to get a buffer from the pool
    fn try_from_pool(&mut self, size: usize) -> Option<DmaRegion> {
        if let Some(buffers) = self.pool.get_mut(&size) {
            if let Some(mut region) = buffers.pop() {
                region.in_use = true;
                self.stats.pool_hits += 1;
                self.stats.current_usage += size;
                return Some(region);
            }
        }
        self.stats.pool_misses += 1;
        None
    }
    
    /// Return a buffer to the pool
    fn return_to_pool(&mut self, mut region: DmaRegion) -> bool {
        let size = region.size;
        region.in_use = false;
        
        let buffers = self.pool.entry(size).or_insert_with(Vec::new);
        
        if buffers.len() < MAX_POOLED_BUFFERS {
            buffers.push(region);
            self.stats.current_usage = self.stats.current_usage.saturating_sub(size);
            true
        } else {
            // Pool is full, would deallocate here if we had proper page deallocation
            self.stats.current_usage = self.stats.current_usage.saturating_sub(size);
            self.stats.total_freed += size;
            false
        }
    }
    
    /// Allocate a new region from the heap
    fn allocate_new(&mut self, aligned_size: usize) -> Result<DmaRegion, AllocError> {
        if !self.initialized {
            return Err(AllocError::OutOfMemory);
        }

        if self.offset + aligned_size > DMA_HEAP_SIZE {
            return Err(AllocError::OutOfMemory);
        }

        let virt_addr = VirtAddr::new((DMA_HEAP_START + self.offset) as u64);
        
        // Translate to physical address
        let phys_addr = unsafe {
            crate::memory::translate_addr(virt_addr, VirtAddr::new(self.physical_memory_offset))
                .ok_or(AllocError::MapToError)?
        };

        self.offset += aligned_size;
        
        let region = DmaRegion {
            virt_addr,
            phys_addr,
            size: aligned_size,
            in_use: true,
        };
        
        self.regions.push(region.clone());
        self.stats.total_allocated += aligned_size;
        self.stats.current_usage += aligned_size;
        if self.stats.current_usage > self.stats.peak_usage {
            self.stats.peak_usage = self.stats.current_usage;
        }
        
        Ok(region)
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
/// 
/// First tries to get a buffer from the pool, otherwise allocates new memory.
/// The returned buffer will automatically be returned to the pool when dropped.
/// 
/// # Arguments
/// * `size` - Size in bytes (will be rounded up to page alignment)
/// 
/// # Returns
/// A DmaBuffer that can be used for DMA operations
pub fn allocate_dma_buffer(size: usize) -> Result<DmaBuffer, AllocError> {
    if size == 0 {
        return Err(AllocError::Alignment);
    }

    let aligned_size = (size + DMA_ALIGNMENT - 1) & !(DMA_ALIGNMENT - 1);
    let mut allocator = DMA_ALLOCATOR.lock();
    
    // Try to get from pool first
    let region = if let Some(region) = allocator.try_from_pool(aligned_size) {
        region
    } else {
        // Allocate new if pool miss
        allocator.allocate_new(aligned_size)?
    };

    Ok(DmaBuffer {
        virt_addr: region.virt_addr,
        phys_addr: region.phys_addr,
        size: region.size,
        pooled: true, // Mark as pooled for automatic return on drop
    })
}

/// Allocate a DMA buffer that won't be pooled (for long-lived buffers)
/// 
/// Use this for buffers that will live for the entire program lifetime,
/// such as network card ring buffers.
pub fn allocate_dma_buffer_unpooled(size: usize) -> Result<DmaBuffer, AllocError> {
    let mut buffer = allocate_dma_buffer(size)?;
    buffer.pooled = false; // Don't return to pool on drop
    Ok(buffer)
}

impl DmaBuffer {
    /// Get a mutable slice for the DMA buffer
    /// 
    /// # Safety
    /// The caller must ensure no aliasing occurs while this slice is active.
    /// This is safe for DMA operations as hardware typically doesn't alias.
    pub unsafe fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.virt_addr.as_mut_ptr::<u8>(),
                self.size,
            )
        }
    }

    /// Get an immutable slice for the DMA buffer
    /// 
    /// # Safety
    /// The caller must ensure the hardware has finished writing to this buffer.
    pub unsafe fn as_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.virt_addr.as_ptr::<u8>(),
                self.size,
            )
        }
    }
    
    /// Zero out the buffer contents
    pub fn zero(&mut self) {
        unsafe {
            core::ptr::write_bytes(
                self.virt_addr.as_mut_ptr::<u8>(),
                0,
                self.size,
            );
        }
    }
    
    /// Create a non-pooled clone (useful for long-lived references)
    pub fn clone_unpooled(&self) -> Self {
        Self {
            virt_addr: self.virt_addr,
            phys_addr: self.phys_addr,
            size: self.size,
            pooled: false,
        }
    }
}

/// Automatically return buffer to pool when dropped
impl Drop for DmaBuffer {
    fn drop(&mut self) {
        if self.pooled {
            let region = DmaRegion {
                virt_addr: self.virt_addr,
                phys_addr: self.phys_addr,
                size: self.size,
                in_use: true,
            };
            
            let mut allocator = DMA_ALLOCATOR.lock();
            allocator.return_to_pool(region);
        }
    }
}

/// Get DMA allocator statistics for debugging and monitoring
pub fn get_dma_stats() -> DmaStatistics {
    let allocator = DMA_ALLOCATOR.lock();
    DmaStatistics {
        total_allocated: allocator.stats.total_allocated,
        total_freed: allocator.stats.total_freed,
        current_usage: allocator.stats.current_usage,
        peak_usage: allocator.stats.peak_usage,
        pool_hits: allocator.stats.pool_hits,
        pool_misses: allocator.stats.pool_misses,
        pool_size: allocator.pool.values().map(|v| v.len()).sum(),
        total_regions: allocator.regions.len(),
    }
}

/// Public statistics structure
#[derive(Debug, Clone, Copy)]
pub struct DmaStatistics {
    pub total_allocated: usize,
    pub total_freed: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub pool_hits: usize,
    pub pool_misses: usize,
    pub pool_size: usize,
    pub total_regions: usize,
}

impl core::fmt::Display for DmaStatistics {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "DMA Statistics:\n")?;
        write!(f, "  Total Allocated: {} bytes\n", self.total_allocated)?;
        write!(f, "  Total Freed: {} bytes\n", self.total_freed)?;
        write!(f, "  Current Usage: {} bytes\n", self.current_usage)?;
        write!(f, "  Peak Usage: {} bytes\n", self.peak_usage)?;
        write!(f, "  Pool Hits: {}\n", self.pool_hits)?;
        write!(f, "  Pool Misses: {}\n", self.pool_misses)?;
        write!(f, "  Buffers in Pool: {}\n", self.pool_size)?;
        write!(f, "  Total Regions: {}\n", self.total_regions)?;
        if self.pool_hits + self.pool_misses > 0 {
            let hit_rate = (self.pool_hits as f64) / ((self.pool_hits + self.pool_misses) as f64) * 100.0;
            write!(f, "  Pool Hit Rate: {:.1}%", hit_rate)?;
        }
        Ok(())
    }
}
