#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustrial_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use alloc::vec;
use alloc::vec::Vec;
use rustrial_os::net::loopback::LoopbackDevice;
use rustrial_os::drivers::net::{NetworkDevice, TransmitError, LinkStatus};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use rustrial_os::allocator;
    use rustrial_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    rustrial_os::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");
    
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustrial_os::test_panic_handler(info)
}

#[test_case]
fn test_loopback_device_creation() {
    // Test that we can create a loopback device
    let loopback = LoopbackDevice::default();
    
    // Verify MAC address is all zeros
    assert_eq!(loopback.mac_address(), [0, 0, 0, 0, 0, 0]);
    
    // Verify device name
    assert_eq!(loopback.device_name(), "lo (loopback)");
    
    // Verify link status is always up
    assert_eq!(loopback.link_status(), LinkStatus::Up);
    
    // Verify device is ready
    assert!(loopback.is_ready());
}

#[test_case]
fn test_loopback_echo_single_packet() {
    let mut loopback = LoopbackDevice::default();
    
    let test_packet = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    
    // Transmit a packet
    assert!(loopback.transmit(&test_packet).is_ok());
    
    // Receive should return the same packet
    let received = loopback.receive();
    assert!(received.is_some());
    
    let received_packet = received.unwrap();
    assert_eq!(received_packet.len(), test_packet.len());
    assert_eq!(received_packet, test_packet);
}

#[test_case]
fn test_loopback_multiple_packets_fifo() {
    let mut loopback = LoopbackDevice::default();
    
    // Send multiple packets with different data
    for i in 0..5 {
        let mut packet = Vec::new();
        for _ in 0..10 {
            packet.push(i);
        }
        assert!(loopback.transmit(&packet).is_ok());
    }
    
    // Receive them in order (FIFO)
    for i in 0..5 {
        let received = loopback.receive();
        assert!(received.is_some());
        
        let packet = received.unwrap();
        assert_eq!(packet.len(), 10);
        
        // All bytes should be 'i'
        for byte in &packet {
            assert_eq!(*byte, i);
        }
    }
    
    // Queue should be empty now
    assert!(loopback.receive().is_none());
}

#[test_case]
fn test_loopback_buffer_full() {
    let mut loopback = LoopbackDevice::new(2); // Small queue of 2
    
    let packet = vec![0xAA; 64];
    
    // Fill the queue
    assert!(loopback.transmit(&packet).is_ok());
    assert!(loopback.transmit(&packet).is_ok());
    
    // Third transmit should fail (buffer full)
    let result = loopback.transmit(&packet);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), TransmitError::BufferFull);
}

#[test_case]
fn test_loopback_empty_packet() {
    let mut loopback = LoopbackDevice::default();
    
    // Try to send an empty packet
    let empty_packet = vec![];
    assert!(loopback.transmit(&empty_packet).is_ok());
    
    // Should receive an empty packet back
    let received = loopback.receive();
    assert!(received.is_some());
    assert_eq!(received.unwrap().len(), 0);
}

#[test_case]
fn test_loopback_large_packet() {
    let mut loopback = LoopbackDevice::default();
    
    // Create a large packet (1500 bytes - typical MTU)
    let large_packet = vec![0xBB; 1500];
    
    // Should be able to transmit
    assert!(loopback.transmit(&large_packet).is_ok());
    
    // Should receive the full packet back
    let received = loopback.receive();
    assert!(received.is_some());
    
    let received_packet = received.unwrap();
    assert_eq!(received_packet.len(), 1500);
    assert_eq!(received_packet, large_packet);
}

#[test_case]
fn test_loopback_receive_empty_queue() {
    let mut loopback = LoopbackDevice::default();
    
    // Try to receive from empty queue
    assert!(loopback.receive().is_none());
    
    // Multiple receives should all return None
    assert!(loopback.receive().is_none());
    assert!(loopback.receive().is_none());
}

#[test_case]
fn test_loopback_custom_queue_size() {
    let mut loopback = LoopbackDevice::new(10);
    
    let packet = vec![0x55; 32];
    
    // Should be able to queue up to 10 packets
    for _ in 0..10 {
        assert!(loopback.transmit(&packet).is_ok());
    }
    
    // 11th packet should fail
    assert_eq!(loopback.transmit(&packet).unwrap_err(), TransmitError::BufferFull);
    
    // Drain the queue
    for _ in 0..10 {
        assert!(loopback.receive().is_some());
    }
    
    // Queue should be empty
    assert!(loopback.receive().is_none());
}
