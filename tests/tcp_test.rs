//! tcp protocol tests

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
    
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    test_main();
    rustrial_os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustrial_os::test_panic_handler(info)
}

use core::net::Ipv4Addr;
use rustrial_os::net::tcp::{TcpPacket, TcpConnection, TcpSocketId, TcpState, flags};

#[test_case]
fn test_tcp_packet_parse() {
    serial_print!("tcp_packet_parse... ");
    
    let mut data = alloc::vec![0u8; 20];
    data[0..2].copy_from_slice(&80u16.to_be_bytes());
    data[2..4].copy_from_slice(&8080u16.to_be_bytes());
    data[4..8].copy_from_slice(&1000u32.to_be_bytes());
    data[8..12].copy_from_slice(&0u32.to_be_bytes());
    data[12] = 5 << 4;
    data[13] = flags::SYN;
    data[14..16].copy_from_slice(&8192u16.to_be_bytes());
    
    let src = Ipv4Addr::new(192, 168, 1, 1);
    let dst = Ipv4Addr::new(192, 168, 1, 2);
    
    let packet = TcpPacket::parse(&data, src, dst).unwrap();
    assert_eq!(packet.src_port, 80);
    assert_eq!(packet.dest_port, 8080);
    assert_eq!(packet.sequence, 1000);
    assert_eq!(packet.has_flag(flags::SYN), true);
    assert_eq!(packet.window, 8192);
    
    serial_println!("[ok]");
}

#[test_case]
fn test_tcp_connection_state() {
    serial_print!("tcp_connection_state... ");
    
    let socket_id = TcpSocketId {
        local_addr: Ipv4Addr::new(192, 168, 1, 1),
        local_port: 8080,
        remote_addr: Ipv4Addr::new(192, 168, 1, 2),
        remote_port: 80,
    };
    
    let conn = TcpConnection::new(socket_id);
    assert_eq!(conn.state, TcpState::Closed);
    assert_eq!(conn.cwnd, 2920);
    assert_eq!(conn.ssthresh, 65535);
    
    serial_println!("[ok]");
}

#[test_case]
fn test_tcp_isn_generation() {
    serial_print!("tcp_isn_generation... ");
    
    let socket_id = TcpSocketId {
        local_addr: Ipv4Addr::new(192, 168, 1, 1),
        local_port: 8080,
        remote_addr: Ipv4Addr::new(192, 168, 1, 2),
        remote_port: 80,
    };
    
    let mut conn = TcpConnection::new(socket_id);
    let _syn_packet = conn.connect().unwrap();
    
    assert_eq!(conn.state, TcpState::SynSent);
    assert!(conn.initial_send_seq > 0);
    
    serial_println!("[ok]");
}

#[test_case]
fn test_tcp_sliding_window() {
    serial_print!("tcp_sliding_window... ");
    
    let socket_id = TcpSocketId {
        local_addr: Ipv4Addr::new(192, 168, 1, 1),
        local_port: 8080,
        remote_addr: Ipv4Addr::new(192, 168, 1, 2),
        remote_port: 80,
    };
    
    let mut conn = TcpConnection::new(socket_id);
    conn.state = TcpState::Established;
    conn.send_seq = 1000;
    conn.recv_seq = 2000;
    
    let initial_cwnd = conn.cwnd;
    assert!(initial_cwnd > 0);
    

    serial_println!("[ok]");
}
