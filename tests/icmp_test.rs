//Integration tests for ICMP (Internet Control Message Protocol)
//
// Tests echo request/reply, checksum calculation, and ping statistics.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustrial_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rustrial_os::net::icmp::{IcmpPacket, IcmpType, IcmpError, PingStats};
use alloc::vec;

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
    rustrial_os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustrial_os::test_panic_handler(info)
}

#[test_case]
fn test_icmp_type_conversion() {
    assert_eq!(u8::from(IcmpType::EchoRequest), 8);
    assert_eq!(u8::from(IcmpType::EchoReply), 0);
    assert_eq!(u8::from(IcmpType::DestinationUnreachable), 3);
    assert_eq!(IcmpType::from(8), IcmpType::EchoRequest);
    assert_eq!(IcmpType::from(0), IcmpType::EchoReply);
}

#[test_case]
fn test_echo_request_creation() {
    let data = vec![0x61, 0x62, 0x63, 0x64]; // "abcd"
    let packet = IcmpPacket::new_echo_request(0x1234, 1, data.clone());

    assert_eq!(packet.icmp_type, IcmpType::EchoRequest);
    assert_eq!(packet.code, 0);
    assert_eq!(packet.identifier, 0x1234);
    assert_eq!(packet.sequence, 1);
    assert_eq!(packet.data, data);
}

#[test_case]
fn test_echo_request_to_bytes() {
    let data = vec![0x61, 0x62, 0x63, 0x64];
    let packet = IcmpPacket::new_echo_request(0x1234, 1, data);
    let bytes = packet.to_bytes();

    assert_eq!(bytes[0], 8); // Type: Echo Request
    assert_eq!(bytes[1], 0); // Code: 0
    assert_eq!(bytes[4], 0x12); // Identifier high byte
    assert_eq!(bytes[5], 0x34); // Identifier low byte
    assert_eq!(bytes[6], 0x00); // Sequence high byte
    assert_eq!(bytes[7], 0x01); // Sequence low byte
    assert_eq!(&bytes[8..], &[0x61, 0x62, 0x63, 0x64]); // Data
}

#[test_case]
fn test_checksum_calculation() {
    // Create a packet and verify checksum
    let data = vec![0x61, 0x62, 0x63, 0x64]; // "abcd"
    let packet = IcmpPacket::new_echo_request(0x1234, 1, data);
    let bytes = packet.to_bytes();
    
    // Parse it back and verify checksum is valid
    let parsed = IcmpPacket::from_bytes(&bytes);
    assert!(parsed.is_ok());
}

#[test_case]
fn test_packet_parsing() {
    let packet = IcmpPacket::new_echo_request(0x5678, 42, vec![1, 2, 3, 4]);
    let bytes = packet.to_bytes();
    
    let parsed = IcmpPacket::from_bytes(&bytes).unwrap();
    
    assert_eq!(parsed.icmp_type, IcmpType::EchoRequest);
    assert_eq!(parsed.code, 0);
    assert_eq!(parsed.identifier, 0x5678);
    assert_eq!(parsed.sequence, 42);
    assert_eq!(parsed.data, vec![1, 2, 3, 4]);
}

#[test_case]
fn test_echo_reply_creation() {
    let request = IcmpPacket::new_echo_request(0xABCD, 10, vec![5, 6, 7, 8]);
    let reply = IcmpPacket::create_echo_reply(&request);

    assert_eq!(reply.icmp_type, IcmpType::EchoReply);
    assert_eq!(reply.code, request.code);
    assert_eq!(reply.identifier, request.identifier);
    assert_eq!(reply.sequence, request.sequence);
    assert_eq!(reply.data, request.data);
}

#[test_case]
fn test_packet_too_short() {
    let short_data = vec![0x08, 0x00, 0x00, 0x00, 0x12, 0x34, 0x00]; // Only 7 bytes
    let result = IcmpPacket::from_bytes(&short_data);
    assert_eq!(result, Err(IcmpError::PacketTooShort));
}

#[test_case]
fn test_invalid_checksum() {
    let packet = IcmpPacket::new_echo_request(0x1111, 5, vec![9, 10, 11]);
    let mut bytes = packet.to_bytes();
    
    // Corrupt the checksum
    bytes[2] ^= 0xFF;
    
    let result = IcmpPacket::from_bytes(&bytes);
    assert_eq!(result, Err(IcmpError::InvalidChecksum));
}

#[test_case]
fn test_ping_stats_basic() {
    let stats = PingStats::new();
    assert_eq!(stats.sent, 0);
    assert_eq!(stats.received, 0);
    assert_eq!(stats.avg_rtt_ms, 0);
}

#[test_case]
fn test_ping_stats_measurements() {
    let mut stats = PingStats::new();
    stats.sent = 5;
    
    stats.add_measurement(10);
    stats.add_measurement(20);
    stats.add_measurement(15);
    
    assert_eq!(stats.received, 3);
    assert_eq!(stats.avg_rtt_ms, 15); // (10 + 20 + 15) / 3 = 15
    assert_eq!(stats.min_rtt_ms, 10);
    assert_eq!(stats.max_rtt_ms, 20);
}

#[test_case]
fn test_ping_stats_packet_loss() {
    let mut stats = PingStats::new();
    stats.sent = 10;
    stats.received = 7;
    
    let loss = stats.packet_loss();
    assert!(loss > 29.9 && loss < 30.1); // 30% loss (3/10)
}

#[test_case]
fn test_is_echo_methods() {
    let request = IcmpPacket::new_echo_request(1, 1, vec![]);
    assert!(request.is_echo_request());
    assert!(!request.is_echo_reply());

    let reply = IcmpPacket::create_echo_reply(&request);
    assert!(!reply.is_echo_request());
    assert!(reply.is_echo_reply());
}

#[test_case]
fn test_empty_data_packet() {
    let packet = IcmpPacket::new_echo_request(0x0000, 0, vec![]);
    let bytes = packet.to_bytes();
    
    assert_eq!(bytes.len(), 8); // Header only
    
    let parsed = IcmpPacket::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.data.len(), 0);
}

#[test_case]
fn test_large_data_packet() {
    let large_data = vec![0xAA; 512]; // 512 bytes of data
    let packet = IcmpPacket::new_echo_request(0xFFFF, 999, large_data.clone());
    let bytes = packet.to_bytes();
    
    let parsed = IcmpPacket::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.data.len(), 512);
    assert_eq!(parsed.data, large_data);
}

#[test_case]
fn test_checksum_odd_length() {
    // Test checksum with odd-length data
    let data = vec![0x61, 0x62, 0x63]; // 3 bytes (odd)
    let packet = IcmpPacket::new_echo_request(0x1234, 1, data.clone());
    let bytes = packet.to_bytes();
    
    let parsed = IcmpPacket::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.data, data);
}

#[test_case]
fn test_multiple_echo_sequence() {
    // Test creating multiple echo requests with incrementing sequence
    for seq in 1..=10 {
        let packet = IcmpPacket::new_echo_request(0x9999, seq, vec![0xBB; 32]);
        assert_eq!(packet.sequence, seq);
        assert!(packet.is_echo_request());
        
        let bytes = packet.to_bytes();
        let parsed = IcmpPacket::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.sequence, seq);
    }
}

#[test_case]
fn test_roundtrip_conversion() {
    // Test that packet -> bytes -> packet preserves all fields
    let original = IcmpPacket::new_echo_request(0x4321, 123, vec![1, 2, 3, 4, 5]);
    let bytes = original.to_bytes();
    let parsed = IcmpPacket::from_bytes(&bytes).unwrap();
    
    assert_eq!(parsed.icmp_type, original.icmp_type);
    assert_eq!(parsed.code, original.code);
    assert_eq!(parsed.identifier, original.identifier);
    assert_eq!(parsed.sequence, original.sequence);
    assert_eq!(parsed.data, original.data);
}
