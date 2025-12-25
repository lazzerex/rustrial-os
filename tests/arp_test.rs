#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustrial_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use core::net::Ipv4Addr;
use rustrial_os::net::arp::{
    ArpPacket, ArpCache, ArpEntry, ArpError,
    ARP_REQUEST, ARP_REPLY, ARP_PACKET_SIZE,
    HW_TYPE_ETHERNET, PROTO_TYPE_IPV4,
    create_arp_request, create_arp_reply, handle_arp_packet,
};

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
fn test_arp_packet_creation_and_serialization() {
    let sender_mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let sender_ip = Ipv4Addr::new(192, 168, 1, 10);
    let target_ip = Ipv4Addr::new(192, 168, 1, 1);

    // Test request creation
    let request = ArpPacket::new_request(sender_mac, sender_ip, target_ip);
    assert_eq!(request.hw_type, HW_TYPE_ETHERNET);
    assert_eq!(request.proto_type, PROTO_TYPE_IPV4);
    assert_eq!(request.operation, ARP_REQUEST);
    assert!(request.is_request());
    assert!(!request.is_reply());

    // Test serialization
    let bytes = request.to_bytes();
    assert_eq!(bytes.len(), ARP_PACKET_SIZE);
    assert_eq!(u16::from_be_bytes([bytes[0], bytes[1]]), HW_TYPE_ETHERNET);
    assert_eq!(u16::from_be_bytes([bytes[6], bytes[7]]), ARP_REQUEST);

    // Test reply creation
    let target_mac = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
    let reply = ArpPacket::new_reply(sender_mac, sender_ip, target_mac, target_ip);
    assert_eq!(reply.operation, ARP_REPLY);
    assert!(!reply.is_request());
    assert!(reply.is_reply());
}

#[test_case]
fn test_arp_packet_parsing() {
    let sender_mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let sender_ip = Ipv4Addr::new(192, 168, 1, 10);
    let target_ip = Ipv4Addr::new(192, 168, 1, 1);

    let original = ArpPacket::new_request(sender_mac, sender_ip, target_ip);
    let bytes = original.to_bytes();
    let parsed = ArpPacket::from_bytes(&bytes).unwrap();

    assert_eq!(parsed.hw_type, original.hw_type);
    assert_eq!(parsed.proto_type, original.proto_type);
    assert_eq!(parsed.hw_len, original.hw_len);
    assert_eq!(parsed.proto_len, original.proto_len);
    assert_eq!(parsed.operation, original.operation);
    assert_eq!(parsed.sender_mac, original.sender_mac);
    assert_eq!(parsed.sender_ip, original.sender_ip);
    assert_eq!(parsed.target_mac, original.target_mac);
    assert_eq!(parsed.target_ip, original.target_ip);
}

#[test_case]
fn test_arp_packet_too_short() {
    let data = [0u8; 10];
    let result = ArpPacket::from_bytes(&data);
    assert_eq!(result, Err(ArpError::PacketTooShort));
}

#[test_case]
fn test_arp_cache_operations() {
    let cache = ArpCache::new();
    let ip = Ipv4Addr::new(192, 168, 1, 1);
    let mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let current_time = 100;

    // Test insert and lookup
    cache.insert(ip, mac, current_time);
    assert_eq!(cache.lookup(ip, current_time), Some(mac));

    // Test expiration
    assert_eq!(cache.lookup(ip, current_time + 299), Some(mac));
    assert_eq!(cache.lookup(ip, current_time + 300), None);

    // Test clear
    cache.insert(ip, mac, current_time);
    assert_eq!(cache.len(), 1);
    cache.clear();
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[test_case]
fn test_arp_cache_remove_expired() {
    let cache = ArpCache::new();
    let ip1 = Ipv4Addr::new(192, 168, 1, 1);
    let ip2 = Ipv4Addr::new(192, 168, 1, 2);
    let ip3 = Ipv4Addr::new(192, 168, 1, 3);
    let mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    
    cache.insert(ip1, mac, 100);
    cache.insert(ip2, mac, 200);
    cache.insert(ip3, mac, 300);
    
    assert_eq!(cache.len(), 3);
    
    // At time 450, ip1 (expires at 400) should be removed
    let removed = cache.remove_expired(450);
    assert_eq!(removed, 1);
    assert_eq!(cache.len(), 2);
    
    // At time 550, ip2 (expires at 500) should be removed
    let removed = cache.remove_expired(550);
    assert_eq!(removed, 1);
    assert_eq!(cache.len(), 1);
    
    // At time 650, ip3 (expires at 600) should be removed
    let removed = cache.remove_expired(650);
    assert_eq!(removed, 1);
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[test_case]
fn test_arp_cache_clear() {
    let cache = ArpCache::new();
    let ip1 = Ipv4Addr::new(192, 168, 1, 1);
    let ip2 = Ipv4Addr::new(192, 168, 1, 2);
    let mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    
    cache.insert(ip1, mac, 100);
    cache.insert(ip2, mac, 100);
    
    assert_eq!(cache.len(), 2);
    
    cache.clear();
    
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[test_case]
fn test_arp_helpers_and_handlers() {
    // Test helper functions
    let our_mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let our_ip = Ipv4Addr::new(192, 168, 1, 1);
    let target_ip = Ipv4Addr::new(192, 168, 1, 10);

    let request_bytes = create_arp_request(target_ip, our_mac, our_ip);
    assert_eq!(request_bytes.len(), ARP_PACKET_SIZE);

    // Test request handling
    let sender_mac = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
    let sender_ip = Ipv4Addr::new(192, 168, 1, 10);
    let request = ArpPacket::new_request(sender_mac, sender_ip, our_ip);
    let result = handle_arp_packet(&request.to_bytes(), our_ip, our_mac, 100).unwrap();
    
    assert!(result.is_some()); // Should return reply
    
    let reply = ArpPacket::from_bytes(&result.unwrap()).unwrap();
    assert!(reply.is_reply());
    assert_eq!(reply.sender_mac, our_mac);
}
