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
use rustrial_os::net::ethernet::{
    EthernetFrame, EthernetError, ETHERTYPE_IPV4, ETHERTYPE_ARP, 
    BROADCAST_MAC, MIN_PAYLOAD_SIZE, HEADER_SIZE, CRC_SIZE
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
fn test_ethernet_frame_creation() {
    let dest = [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01];
    let src = [0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x02];
    let payload = vec![0x01, 0x02, 0x03, 0x04];

    let frame = EthernetFrame::new(dest, src, ETHERTYPE_IPV4, payload.clone()).unwrap();

    assert_eq!(frame.dest_mac, dest);
    assert_eq!(frame.src_mac, src);
    assert_eq!(frame.ethertype, ETHERTYPE_IPV4);
    assert_eq!(frame.payload, payload);
}

#[test_case]
fn test_ethernet_frame_to_bytes() {
    let dest = [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01];
    let src = [0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x02];
    let payload = vec![0x01, 0x02, 0x03, 0x04];

    let frame = EthernetFrame::new(dest, src, ETHERTYPE_ARP, payload).unwrap();
    let bytes = frame.to_bytes();

    // Check header
    assert_eq!(&bytes[0..6], &dest);
    assert_eq!(&bytes[6..12], &src);
    assert_eq!(u16::from_be_bytes([bytes[12], bytes[13]]), ETHERTYPE_ARP);

    // Check minimum frame size (14 header + 46 payload + 4 CRC = 64 bytes)
    assert_eq!(bytes.len(), HEADER_SIZE + MIN_PAYLOAD_SIZE + CRC_SIZE);
}

#[test_case]
fn test_ethernet_frame_parsing() {
    let dest = [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01];
    let src = [0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x02];
    let payload = vec![0x42; 60]; // 60-byte payload

    let frame = EthernetFrame::new(dest, src, ETHERTYPE_IPV4, payload.clone()).unwrap();
    let bytes = frame.to_bytes();

    let parsed = EthernetFrame::from_bytes(&bytes).unwrap();

    assert_eq!(parsed.dest_mac, dest);
    assert_eq!(parsed.src_mac, src);
    assert_eq!(parsed.ethertype, ETHERTYPE_IPV4);
    assert_eq!(&parsed.payload[..payload.len()], &payload[..]);
}

#[test_case]
fn test_broadcast_detection() {
    let frame = EthernetFrame::new(
        BROADCAST_MAC,
        [0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x02],
        ETHERTYPE_ARP,
        vec![0x01, 0x02, 0x03, 0x04],
    ).unwrap();

    assert!(frame.is_broadcast());
    assert!(!frame.is_unicast());
    assert!(!frame.is_multicast());
}

#[test_case]
fn test_multicast_detection() {
    let frame = EthernetFrame::new(
        [0x01, 0x00, 0x5E, 0x00, 0x00, 0x01], // IPv4 multicast
        [0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x02],
        ETHERTYPE_IPV4,
        vec![0x01, 0x02, 0x03, 0x04],
    ).unwrap();

    assert!(frame.is_multicast());
    assert!(!frame.is_broadcast());
    assert!(!frame.is_unicast());
}

#[test_case]
fn test_unicast_detection() {
    let frame = EthernetFrame::new(
        [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01],
        [0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x02],
        ETHERTYPE_IPV4,
        vec![0x01, 0x02, 0x03, 0x04],
    ).unwrap();

    assert!(frame.is_unicast());
    assert!(!frame.is_broadcast());
    assert!(!frame.is_multicast());
}

#[test_case]
fn test_payload_too_large() {
    use rustrial_os::net::ethernet::MAX_PAYLOAD_SIZE;
    
    let payload = vec![0x42; MAX_PAYLOAD_SIZE + 1];
    let result = EthernetFrame::new(
        [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01],
        [0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x02],
        ETHERTYPE_IPV4,
        payload,
    );

    assert_eq!(result, Err(EthernetError::PayloadTooLarge));
}

#[test_case]
fn test_frame_too_short() {
    let data = vec![0x42; 10]; // Too short to be valid
    let result = EthernetFrame::from_bytes(&data);

    assert_eq!(result, Err(EthernetError::FrameTooShort));
}
