//! Integration tests for IPv4 implementation

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustrial_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rustrial_os::net::ipv4::{
    Ipv4Header, Ipv4Packet, RoutingTable, ProtocolDispatcher, 
    protocol, Ipv4Error, DEFAULT_TTL, MIN_HEADER_SIZE
};
use core::net::Ipv4Addr;

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

// ========== IPv4 Header Tests ==========

#[test_case]
fn test_ipv4_header_creation() {
    let src_ip = Ipv4Addr::new(192, 168, 1, 100);
    let dest_ip = Ipv4Addr::new(8, 8, 8, 8);
    
    let header = Ipv4Header::new(src_ip, dest_ip, protocol::ICMP, 64);
    
    assert_eq!(header.version, 4);
    assert_eq!(header.ihl, 5); // 20 bytes
    assert_eq!(header.ttl, DEFAULT_TTL);
    assert_eq!(header.protocol, protocol::ICMP);
    assert_eq!(header.src_ip, src_ip);
    assert_eq!(header.dest_ip, dest_ip);
    assert_eq!(header.total_length, MIN_HEADER_SIZE as u16 + 64);
}

#[test_case]
fn test_ipv4_header_serialization_and_parsing() {
    let src_ip = Ipv4Addr::new(10, 0, 2, 15);
    let dest_ip = Ipv4Addr::new(93, 184, 216, 34); // example.com
    
    let mut header = Ipv4Header::new(src_ip, dest_ip, protocol::TCP, 100);
    header.identification = 12345;
    
    let bytes = header.to_bytes();
    
    // Verify serialization
    assert_eq!(bytes.len(), MIN_HEADER_SIZE);
    assert_eq!(bytes[0] >> 4, 4); // Version
    assert_eq!(bytes[0] & 0x0F, 5); // IHL
    assert_eq!(bytes[8], DEFAULT_TTL); // TTL
    assert_eq!(bytes[9], protocol::TCP); // Protocol
    
    // Verify parsing
    let (parsed, offset) = Ipv4Header::from_bytes(&bytes).unwrap();
    assert_eq!(offset, MIN_HEADER_SIZE);
    assert_eq!(parsed.version, 4);
    assert_eq!(parsed.ihl, 5);
    assert_eq!(parsed.src_ip, src_ip);
    assert_eq!(parsed.dest_ip, dest_ip);
    assert_eq!(parsed.protocol, protocol::TCP);
    assert_eq!(parsed.identification, 12345);
    assert_eq!(parsed.total_length, 120);
}

#[test_case]
fn test_ipv4_checksum_calculation_and_verification() {
    let src_ip = Ipv4Addr::new(192, 168, 0, 1);
    let dest_ip = Ipv4Addr::new(192, 168, 0, 2);
    
    let header = Ipv4Header::new(src_ip, dest_ip, protocol::UDP, 50);
    let bytes = header.to_bytes();
    
    // Verify checksum was calculated
    let checksum = u16::from_be_bytes([bytes[10], bytes[11]]);
    assert_ne!(checksum, 0);
    
    // Verify checksum is correct by parsing
    let (parsed, _) = Ipv4Header::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.checksum, checksum);
    
    // Verify checksum validation
    assert!(parsed.verify_checksum(&bytes));
    
    // Corrupt checksum and verify it fails
    let mut corrupted = bytes.clone();
    corrupted[10] ^= 0xFF;
    let result = Ipv4Header::from_bytes(&corrupted);
    assert!(matches!(result, Err(Ipv4Error::ChecksumMismatch)));
}

#[test_case]
fn test_ipv4_packet_creation_and_round_trip() {
    let src_ip = Ipv4Addr::new(10, 1, 2, 3);
    let dest_ip = Ipv4Addr::new(10, 4, 5, 6);
    let payload = b"Hello, IPv4!".to_vec();
    
    let packet = Ipv4Packet::new(src_ip, dest_ip, protocol::ICMP, payload.clone());
    
    // Serialize
    let bytes = packet.to_bytes();
    assert_eq!(bytes.len(), MIN_HEADER_SIZE + payload.len());
    
    // Parse back
    let parsed = Ipv4Packet::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.header.src_ip, src_ip);
    assert_eq!(parsed.header.dest_ip, dest_ip);
    assert_eq!(parsed.header.protocol, protocol::ICMP);
    assert_eq!(parsed.payload, payload);
}

// ========== Routing Table Tests ==========

#[test_case]
fn test_routing_table_local_subnet_detection() {
    let local_ip = Ipv4Addr::new(192, 168, 1, 100);
    let netmask = Ipv4Addr::new(255, 255, 255, 0);
    let gateway = Some(Ipv4Addr::new(192, 168, 1, 1));
    
    let routing_table = RoutingTable::new(local_ip, netmask, gateway);
    
    // Test local subnet IPs
    assert!(routing_table.is_local(Ipv4Addr::new(192, 168, 1, 1)));
    assert!(routing_table.is_local(Ipv4Addr::new(192, 168, 1, 50)));
    assert!(routing_table.is_local(Ipv4Addr::new(192, 168, 1, 254)));
    
    // Test non-local IPs
    assert!(!routing_table.is_local(Ipv4Addr::new(192, 168, 2, 1)));
    assert!(!routing_table.is_local(Ipv4Addr::new(8, 8, 8, 8)));
    assert!(!routing_table.is_local(Ipv4Addr::new(10, 0, 0, 1)));
}

#[test_case]
fn test_routing_table_next_hop_logic() {
    let local_ip = Ipv4Addr::new(10, 0, 2, 15);
    let netmask = Ipv4Addr::new(255, 255, 255, 0);
    let gateway = Some(Ipv4Addr::new(10, 0, 2, 2));
    
    let routing_table = RoutingTable::new(local_ip, netmask, gateway);
    
    // Local destination: next hop is the destination itself
    let local_dest = Ipv4Addr::new(10, 0, 2, 50);
    assert_eq!(routing_table.next_hop(local_dest), Some(local_dest));
    
    // Remote destination: next hop is the gateway
    let remote_dest = Ipv4Addr::new(8, 8, 8, 8);
    assert_eq!(routing_table.next_hop(remote_dest), Some(Ipv4Addr::new(10, 0, 2, 2)));
    
    // Test without gateway (no route)
    let no_gateway_table = RoutingTable::new(local_ip, netmask, None);
    assert_eq!(no_gateway_table.next_hop(remote_dest), None);
}

#[test_case]
fn test_routing_table_our_ip_check() {
    let local_ip = Ipv4Addr::new(172, 16, 0, 10);
    let netmask = Ipv4Addr::new(255, 255, 0, 0);
    let routing_table = RoutingTable::new(local_ip, netmask, None);
    
    assert!(routing_table.is_our_ip(local_ip));
    assert!(!routing_table.is_our_ip(Ipv4Addr::new(172, 16, 0, 11)));
}

// ========== Protocol Dispatcher Tests ==========

static mut ICMP_CALLED: bool = false;
static mut TCP_CALLED: bool = false;
static mut UDP_CALLED: bool = false;

fn mock_icmp_handler(_header: &Ipv4Header, _payload: &[u8]) {
    unsafe { ICMP_CALLED = true; }
}

fn mock_tcp_handler(_header: &Ipv4Header, _payload: &[u8]) {
    unsafe { TCP_CALLED = true; }
}

fn mock_udp_handler(_header: &Ipv4Header, _payload: &[u8]) {
    unsafe { UDP_CALLED = true; }
}

#[test_case]
fn test_protocol_dispatcher_routing() {
    unsafe {
        ICMP_CALLED = false;
        TCP_CALLED = false;
        UDP_CALLED = false;
    }
    
    let mut dispatcher = ProtocolDispatcher::new();
    dispatcher.register_icmp(mock_icmp_handler);
    dispatcher.register_tcp(mock_tcp_handler);
    dispatcher.register_udp(mock_udp_handler);
    
    let src_ip = Ipv4Addr::new(10, 0, 0, 1);
    let dest_ip = Ipv4Addr::new(10, 0, 0, 2);
    
    // Test ICMP dispatch
    let icmp_header = Ipv4Header::new(src_ip, dest_ip, protocol::ICMP, 0);
    assert!(dispatcher.dispatch(&icmp_header, &[]));
    assert!(unsafe { ICMP_CALLED });
    
    // Test TCP dispatch
    unsafe { TCP_CALLED = false; }
    let tcp_header = Ipv4Header::new(src_ip, dest_ip, protocol::TCP, 0);
    assert!(dispatcher.dispatch(&tcp_header, &[]));
    assert!(unsafe { TCP_CALLED });
    
    // Test UDP dispatch
    unsafe { UDP_CALLED = false; }
    let udp_header = Ipv4Header::new(src_ip, dest_ip, protocol::UDP, 0);
    assert!(dispatcher.dispatch(&udp_header, &[]));
    assert!(unsafe { UDP_CALLED });
    
    // Test unknown protocol (should return false)
    let unknown_header = Ipv4Header::new(src_ip, dest_ip, 99, 0);
    assert!(!dispatcher.dispatch(&unknown_header, &[]));
}

// ========== Error Handling Tests ==========

#[test_case]
fn test_ipv4_parse_errors() {
    // Test packet too short
    let short_packet = [0u8; 10];
    assert!(matches!(
        Ipv4Header::from_bytes(&short_packet),
        Err(Ipv4Error::PacketTooShort)
    ));
    
    // Test invalid version
    let mut invalid_version = [0u8; 20];
    invalid_version[0] = 0x60; // Version 6, not 4
    assert!(matches!(
        Ipv4Header::from_bytes(&invalid_version),
        Err(Ipv4Error::InvalidVersion(6))
    ));
    
    // Test invalid IHL
    let mut invalid_ihl = [0u8; 20];
    invalid_ihl[0] = 0x43; // Version 4, IHL 3 (too small)
    assert!(matches!(
        Ipv4Header::from_bytes(&invalid_ihl),
        Err(Ipv4Error::InvalidIhl(3))
    ));
}

#[test_case]
fn test_ipv4_header_length_calculation() {
    let src_ip = Ipv4Addr::new(1, 2, 3, 4);
    let dest_ip = Ipv4Addr::new(5, 6, 7, 8);
    
    let header = Ipv4Header::new(src_ip, dest_ip, protocol::ICMP, 100);
    assert_eq!(header.header_length(), 20);
    
    // Test with options
    let mut header_with_opts = header.clone();
    header_with_opts.ihl = 6; // 24 bytes
    header_with_opts.options = alloc::vec![0u8; 4];
    assert_eq!(header_with_opts.header_length(), 24);
}

#[test_case]
fn test_ipv4_fragmentation_detection() {
    let src_ip = Ipv4Addr::new(1, 1, 1, 1);
    let dest_ip = Ipv4Addr::new(2, 2, 2, 2);
    
    // Non-fragmented packet
    let normal = Ipv4Header::new(src_ip, dest_ip, protocol::UDP, 50);
    assert!(!normal.is_fragmented());
    
    // Fragmented packet (with offset)
    let mut fragmented = Ipv4Header::new(src_ip, dest_ip, protocol::UDP, 50);
    fragmented.fragment_offset = 100;
    assert!(fragmented.is_fragmented());
}

#[test_case]
fn test_ipv4_protocol_constants() {
    // Verify standard protocol numbers
    assert_eq!(protocol::ICMP, 1);
    assert_eq!(protocol::TCP, 6);
    assert_eq!(protocol::UDP, 17);
}
