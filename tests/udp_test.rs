#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustrial_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use core::net::Ipv4Addr;
use alloc::vec;
use rustrial_os::net::udp::{UdpPacket, UdpError, UDP_HEADER_SIZE, UDP_PROTOCOL};

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
fn test_udp_packet_creation() {
    let src_port = 12345;
    let dest_port = 53;
    let data = vec![0x01, 0x02, 0x03, 0x04];
    
    let packet = UdpPacket::new(src_port, dest_port, data.clone());
    
    assert_eq!(packet.src_port, src_port);
    assert_eq!(packet.dest_port, dest_port);
    assert_eq!(packet.length, (UDP_HEADER_SIZE + data.len()) as u16);
    assert_eq!(packet.data, data);
}

#[test_case]
fn test_udp_packet_serialization() {
    let packet = UdpPacket::new(5000, 8000, vec![0xAA, 0xBB, 0xCC]);
    let bytes = packet.to_bytes();
    
    // Check header fields
    assert_eq!(u16::from_be_bytes([bytes[0], bytes[1]]), 5000); // src port
    assert_eq!(u16::from_be_bytes([bytes[2], bytes[3]]), 8000); // dest port
    assert_eq!(u16::from_be_bytes([bytes[4], bytes[5]]), 11);   // length (8 + 3)
    
    // Check data
    assert_eq!(bytes[8], 0xAA);
    assert_eq!(bytes[9], 0xBB);
    assert_eq!(bytes[10], 0xCC);
}

#[test_case]
fn test_udp_packet_parsing() {
    let data = vec![
        0x13, 0x88, // src port: 5000
        0x1F, 0x40, // dest port: 8000
        0x00, 0x0C, // length: 12
        0x00, 0x00, // checksum: 0 (disabled)
        0xDE, 0xAD, 0xBE, 0xEF, // data
    ];
    
    let packet = UdpPacket::from_bytes(&data).expect("Failed to parse packet");
    
    assert_eq!(packet.src_port, 5000);
    assert_eq!(packet.dest_port, 8000);
    assert_eq!(packet.length, 12);
    assert_eq!(packet.checksum, 0);
    assert_eq!(packet.data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
}

#[test_case]
fn test_udp_packet_too_short() {
    let data = vec![0x00, 0x01, 0x00, 0x02]; // Only 4 bytes (need 8)
    
    let result = UdpPacket::from_bytes(&data);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), UdpError::PacketTooShort);
}

#[test_case]
fn test_udp_packet_invalid_length() {
    let data = vec![
        0x13, 0x88, // src port
        0x1F, 0x40, // dest port
        0x00, 0x05, // length: 5 (less than minimum 8)
        0x00, 0x00, // checksum
    ];
    
    let result = UdpPacket::from_bytes(&data);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), UdpError::InvalidLength);
}

#[test_case]
fn test_udp_checksum_calculation() {
    let src_ip = Ipv4Addr::new(192, 168, 1, 100);
    let dest_ip = Ipv4Addr::new(192, 168, 1, 1);
    
    let packet = UdpPacket::new(12345, 53, vec![0x01, 0x02, 0x03, 0x04]);
    let checksum = packet.calculate_checksum(src_ip, dest_ip);
    
    // Checksum should be non-zero for non-empty packet
    assert_ne!(checksum, 0);
}

#[test_case]
fn test_udp_checksum_zero_disabled() {
    let src_ip = Ipv4Addr::new(10, 0, 0, 1);
    let dest_ip = Ipv4Addr::new(10, 0, 0, 2);
    
    let mut packet = UdpPacket::new(1234, 5678, vec![0xAA]);
    packet.checksum = 0; // Explicitly disable checksum
    
    // Zero checksum means checksum is disabled (always valid)
    assert!(packet.verify_checksum(src_ip, dest_ip));
}

#[test_case]
fn test_udp_checksum_with_pseudo_header() {
    let src_ip = Ipv4Addr::new(192, 168, 1, 10);
    let dest_ip = Ipv4Addr::new(8, 8, 8, 8);
    
    let mut packet = UdpPacket::new(5000, 53, vec![0x01, 0x02, 0x03]);
    
    // Calculate and set checksum
    packet.checksum = packet.calculate_checksum(src_ip, dest_ip);
    
    // Verify should pass
    assert!(packet.verify_checksum(src_ip, dest_ip));
}

#[test_case]
fn test_udp_empty_data() {
    let packet = UdpPacket::new(1000, 2000, vec![]);
    
    assert_eq!(packet.data.len(), 0);
    assert_eq!(packet.length, UDP_HEADER_SIZE as u16);
    
    let bytes = packet.to_bytes();
    assert_eq!(bytes.len(), UDP_HEADER_SIZE);
}

#[test_case]
fn test_udp_large_data() {
    let large_data = vec![0x55; 1400]; // Near MTU size
    let packet = UdpPacket::new(8000, 9000, large_data.clone());
    
    assert_eq!(packet.data.len(), 1400);
    assert_eq!(packet.length, (UDP_HEADER_SIZE + 1400) as u16);
    
    let bytes = packet.to_bytes();
    assert_eq!(bytes.len(), UDP_HEADER_SIZE + 1400);
    
    // Verify data integrity
    assert_eq!(&bytes[UDP_HEADER_SIZE..], &large_data[..]);
}

#[test_case]
fn test_udp_round_trip() {
    let original = UdpPacket::new(54321, 80, vec![0x12, 0x34, 0x56, 0x78]);
    let bytes = original.to_bytes();
    let parsed = UdpPacket::from_bytes(&bytes).expect("Failed to parse");
    
    assert_eq!(parsed.src_port, original.src_port);
    assert_eq!(parsed.dest_port, original.dest_port);
    assert_eq!(parsed.length, original.length);
    assert_eq!(parsed.data, original.data);
}

#[test_case]
fn test_udp_protocol_constant() {
    // UDP protocol number should be 17 (IANA assigned)
    assert_eq!(UDP_PROTOCOL, 17);
}
