//! Tests for Phase 1.1 networking infrastructure

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustrial_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rustrial_os::{allocator, memory, serial_print, serial_println};
use x86_64::VirtAddr;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    rustrial_os::init();
    
    // Initialize memory
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // Initialize heap
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // Initialize DMA
    rustrial_os::memory::dma::init_dma(&mut mapper, &mut frame_allocator, phys_mem_offset)
        .expect("DMA initialization failed");

    test_main();
    rustrial_os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustrial_os::test_panic_handler(info)
}

use rustrial_os::memory::dma;
use rustrial_os::net::buffer::{PacketRingBuffer, BufferError};

#[test_case]
fn test_dma_allocation() {
    serial_print!("dma_allocation... ");
    
    // Allocate a small buffer
    let buffer = dma::allocate_dma_buffer(1024).expect("Failed to allocate DMA buffer");
    
    // Check addresses are valid
    assert!(buffer.virt_addr.as_u64() != 0, "Virtual address should not be zero");
    assert!(buffer.phys_addr.as_u64() != 0, "Physical address should not be zero");
    assert_eq!(buffer.size, 4096, "Size should be page-aligned to 4KB");
    
    serial_println!("[ok]");
}

#[test_case]
fn test_dma_multiple_allocations() {
    serial_print!("dma_multiple_allocations... ");
    
    // Allocate multiple buffers
    let buf1 = dma::allocate_dma_buffer(2048).expect("Failed to allocate buffer 1");
    let buf2 = dma::allocate_dma_buffer(4096).expect("Failed to allocate buffer 2");
    let buf3 = dma::allocate_dma_buffer(8192).expect("Failed to allocate buffer 3");
    
    // Buffers should be distinct
    assert_ne!(buf1.virt_addr, buf2.virt_addr);
    assert_ne!(buf2.virt_addr, buf3.virt_addr);
    assert_ne!(buf1.virt_addr, buf3.virt_addr);
    
    serial_println!("[ok]");
}

#[test_case]
fn test_dma_buffer_access() {
    serial_print!("dma_buffer_access... ");
    
    let mut buffer = dma::allocate_dma_buffer(512).expect("Failed to allocate DMA buffer");
    
    // Write to buffer
    unsafe {
        let slice = buffer.as_slice_mut();
        slice[0] = 0xAA;
        slice[1] = 0xBB;
        slice[2] = 0xCC;
    }
    
    // Read from buffer
    unsafe {
        let slice = buffer.as_slice();
        assert_eq!(slice[0], 0xAA);
        assert_eq!(slice[1], 0xBB);
        assert_eq!(slice[2], 0xCC);
    }
    
    serial_println!("[ok]");
}

#[test_case]
fn test_ring_buffer_push_pop() {
    serial_print!("ring_buffer_push_pop... ");
    
    let mut ring: PacketRingBuffer<4, 2048> = PacketRingBuffer::new();
    
    assert!(ring.is_empty());
    
    // Push a packet
    let packet = [0x01, 0x02, 0x03, 0x04];
    ring.push(&packet).expect("Failed to push packet");
    
    assert!(!ring.is_empty());
    assert_eq!(ring.len(), 1);
    
    // Pop the packet
    let (data, len) = ring.pop().expect("Failed to pop packet");
    assert_eq!(len, 4);
    assert_eq!(&data[..len], &packet);
    
    assert!(ring.is_empty());
    
    serial_println!("[ok]");
}

#[test_case]
fn test_ring_buffer_full() {
    serial_print!("ring_buffer_full... ");
    
    let mut ring: PacketRingBuffer<4, 2048> = PacketRingBuffer::new();
    let packet = [0xFF; 100];
    
    // Fill the buffer
    for _ in 0..4 {
        ring.push(&packet).expect("Failed to push packet");
    }
    
    assert!(ring.is_full());
    
    // Try to push one more (should fail)
    let result = ring.push(&packet);
    assert!(matches!(result, Err(BufferError::Full)));
    
    serial_println!("[ok]");
}

#[test_case]
fn test_ring_buffer_wrap_around() {
    serial_print!("ring_buffer_wrap_around... ");
    
    let mut ring: PacketRingBuffer<3, 2048> = PacketRingBuffer::new();
    
    // Push and pop to advance indices
    for i in 0..5 {
        let packet = [i as u8; 10];
        ring.push(&packet).expect("Failed to push");
        let (data, len) = ring.pop().expect("Failed to pop");
        assert_eq!(data[0], i as u8);
        assert_eq!(len, 10);
    }
    
    assert!(ring.is_empty());
    
    serial_println!("[ok]");
}

#[test_case]
fn test_heap_size_verification() {
    serial_print!("heap_size_verification... ");
    
    use alloc::vec::Vec;
    
    // Allocate a large vector to verify 2MB heap
    let large_vec: Vec<u8> = Vec::with_capacity(1_000_000); // 1MB
    assert_eq!(large_vec.capacity(), 1_000_000);
    
    serial_println!("[ok]");
}
