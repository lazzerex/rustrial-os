//! Network Stack Integration
//!
//! This module provides the core network stack that coordinates all protocol layers.
//! It manages RX/TX processing, ARP resolution, routing, and network configuration.

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::net::Ipv4Addr;
use spin::Mutex;

use crate::{println, serial_println};
use crate::drivers::net::{has_network_device, get_network_device, transmit_packet, get_mac_address};
use crate::net::arp::{arp_cache, create_arp_request, handle_arp_packet};
use crate::net::ethernet::{EthernetFrame, ETHERTYPE_ARP, ETHERTYPE_IPV4};
use crate::net::ipv4::{Ipv4Header, RoutingTable, protocol};
use crate::net::icmp::{IcmpPacket, IcmpType};

/// Network configuration
#[derive(Debug, Clone, Copy)]
pub struct NetworkConfig {
    /// Local IP address
    pub ip_addr: Ipv4Addr,
    /// Subnet mask
    pub netmask: Ipv4Addr,
    /// Default gateway
    pub gateway: Option<Ipv4Addr>,
}

impl NetworkConfig {
    /// Create a new network configuration
    pub fn new(ip_addr: Ipv4Addr, netmask: Ipv4Addr, gateway: Option<Ipv4Addr>) -> Self {
        Self {
            ip_addr,
            netmask,
            gateway,
        }
    }

    /// Create a default configuration (no network)
    pub fn default() -> Self {
        Self {
            ip_addr: Ipv4Addr::new(0, 0, 0, 0),
            netmask: Ipv4Addr::new(0, 0, 0, 0),
            gateway: None,
        }
    }

    /// Check if configuration is valid (IP is not 0.0.0.0)
    pub fn is_valid(&self) -> bool {
        self.ip_addr != Ipv4Addr::new(0, 0, 0, 0)
    }
}

/// Global network configuration
static NETWORK_CONFIG: Mutex<NetworkConfig> = Mutex::new(NetworkConfig {
    ip_addr: Ipv4Addr::new(0, 0, 0, 0),
    netmask: Ipv4Addr::new(0, 0, 0, 0),
    gateway: None,
});

/// Transmit queue for outgoing packets
static TX_QUEUE: Mutex<VecDeque<TxPacket>> = Mutex::new(VecDeque::new());

/// Maximum TX queue size
const MAX_TX_QUEUE_SIZE: usize = 64;

/// Packet to be transmitted
#[derive(Debug, Clone)]
struct TxPacket {
    /// Destination IP address
    dest_ip: Ipv4Addr,
    /// Protocol number (1=ICMP, 6=TCP, 17=UDP)
    protocol: u8,
    /// Payload data
    payload: Vec<u8>,
}

/// Set the network configuration
///
/// # Arguments
/// * `config` - The new network configuration
pub fn set_network_config(config: NetworkConfig) {
    *NETWORK_CONFIG.lock() = config;
    println!("Network configured: {} / {} gateway {:?}",
             config.ip_addr, config.netmask, config.gateway);
}

/// Get the current network configuration
pub fn get_network_config() -> NetworkConfig {
    *NETWORK_CONFIG.lock()
}

/// Get the routing table based on current configuration
pub fn get_routing_table() -> Option<RoutingTable> {
    let config = get_network_config();
    if config.is_valid() {
        Some(RoutingTable::new(config.ip_addr, config.netmask, config.gateway))
    } else {
        None
    }
}

/// Queue a packet for transmission
///
/// # Arguments
/// * `dest_ip` - Destination IP address
/// * `protocol` - Protocol number (1=ICMP, 6=TCP, 17=UDP)
/// * `payload` - Packet payload
///
/// # Returns
/// * `Ok(())` - Packet queued successfully
/// * `Err(())` - Queue is full
pub fn queue_tx_packet(dest_ip: Ipv4Addr, protocol: u8, payload: Vec<u8>) -> Result<(), ()> {
    serial_println!("TX: Queuing packet for {}, protocol {}, {} bytes", dest_ip, protocol, payload.len());
    
    let mut queue = TX_QUEUE.lock();
    
    if queue.len() >= MAX_TX_QUEUE_SIZE {
        serial_println!("TX: Queue full!");
        return Err(());
    }
    
    queue.push_back(TxPacket {
        dest_ip,
        protocol,
        payload,
    });
    
    serial_println!("TX: Packet queued successfully, queue size: {}", queue.len());
    
    Ok(())
}

/// RX Processing Task
///
/// This async function continuously polls the network device for incoming packets,
/// parses Ethernet frames, and dispatches them to the appropriate protocol handlers.
pub async fn rx_processing_task() {
    serial_println!("RX: Task started");
    let mut packet_count = 0;
    
    loop {
        // Check if network device is available
        if !has_network_device() {
            // No device, sleep and retry
            crate::task::yield_now().await;
            continue;
        }

        // Try to receive a packet
        let packet = {
            let mut device_opt = get_network_device().lock();
            if let Some(ref mut device) = *device_opt {
                device.receive()
            } else {
                None
            }
        };

        if let Some(packet_data) = packet {
            packet_count += 1;
            serial_println!("RX: Received packet #{}, {} bytes", packet_count, packet_data.len());
            
            // Parse Ethernet frame
            match EthernetFrame::from_bytes(&packet_data) {
                Ok(frame) => {
                    serial_println!("RX: EtherType: 0x{:04X}", frame.ethertype);
                    handle_rx_frame(frame);
                }
                Err(e) => {
                    serial_println!("RX: Failed to parse Ethernet frame: {:?}", e);
                }
            }
        }

        // Yield to allow other tasks to run
        crate::task::yield_now().await;
    }
}

/// Handle received Ethernet frame
fn handle_rx_frame(frame: EthernetFrame) {
    match frame.ethertype {
        ETHERTYPE_ARP => {
            // Handle ARP packet
            handle_rx_arp(&frame.payload);
        }
        ETHERTYPE_IPV4 => {
            // Handle IPv4 packet
            handle_rx_ipv4(&frame.payload);
        }
        _ => {
            // Unknown EtherType, ignore
            serial_println!("RX: Unknown EtherType: 0x{:04X}", frame.ethertype);
        }
    }
}

/// Handle received ARP packet
fn handle_rx_arp(data: &[u8]) {
    let config = get_network_config();
    if !config.is_valid() {
        return;
    }

    // Get our MAC address
    let our_mac = match get_mac_address() {
        Some(mac) => mac,
        None => return,
    };

    // Get current time (simplified - use 0 for now, proper timer TBD)
    let current_time = 0;

    // Handle ARP packet (updates cache and generates replies)
    match handle_arp_packet(data, config.ip_addr, our_mac, current_time) {
        Ok(Some(reply_packet)) => {
            // We need to send an ARP reply
            // Wrap in Ethernet frame and transmit
            if let Ok(request_frame) = EthernetFrame::from_bytes(data) {
                match EthernetFrame::new(
                    request_frame.src_mac,
                    our_mac,
                    ETHERTYPE_ARP,
                    reply_packet,
                ) {
                    Ok(reply_frame) => {
                        let frame_bytes = reply_frame.to_bytes();
                        let _ = transmit_packet(&frame_bytes);
                    }
                    Err(_) => {}
                }
            }
        }
        Ok(None) => {}
        Err(_) => {}
    }
}

/// Handle received IPv4 packet
fn handle_rx_ipv4(data: &[u8]) {
    let config = get_network_config();
    if !config.is_valid() {
        return;
    }

    // Parse IPv4 header
    let (header, payload_offset) = match Ipv4Header::from_bytes(data) {
        Ok(result) => result,
        Err(e) => {
            serial_println!("RX: Invalid IPv4 packet: {:?}", e);
            return;
        }
    };

    // Check if packet is for us
    let routing_table = match get_routing_table() {
        Some(rt) => rt,
        None => return,
    };

    if !routing_table.is_our_ip(header.dest_ip) {
        // Not for us, ignore
        return;
    }

    // Extract payload
    let payload = &data[payload_offset..];

    // Dispatch based on protocol
    match header.protocol {
        protocol::ICMP => {
            handle_rx_icmp(&header, payload);
        }
        _ => {
            serial_println!("RX: Unsupported IPv4 protocol: {}", header.protocol);
        }
    }
}

/// Handle received ICMP packet
fn handle_rx_icmp(ip_header: &Ipv4Header, data: &[u8]) {
    // Parse ICMP packet
    let packet = match IcmpPacket::from_bytes(data) {
        Ok(pkt) => pkt,
        Err(e) => {
            serial_println!("RX: Invalid ICMP packet: {:?}", e);
            return;
        }
    };

    match packet.icmp_type {
        IcmpType::EchoRequest => {
            // Generate echo reply
            let reply = IcmpPacket::create_echo_reply(&packet);
            let reply_bytes = reply.to_bytes();
            
            // Queue reply for transmission
            let _ = queue_tx_packet(ip_header.src_ip, protocol::ICMP, reply_bytes);
            
            serial_println!("RX: ICMP Echo Request from {}, sending reply", ip_header.src_ip);
        }
        IcmpType::EchoReply => {
            serial_println!("RX: ICMP Echo Reply from {} (seq={})", 
                          ip_header.src_ip, packet.sequence);
        }
        _ => {
            serial_println!("RX: ICMP type {} from {}", packet.icmp_type, ip_header.src_ip);
        }
    }
}

/// TX Processing Task
///
/// This async function processes the transmit queue, performs ARP resolution,
/// builds complete packets, and transmits them via the network device.
pub async fn tx_processing_task() {
    serial_println!("TX: Task started");
    
    loop {
        // Check if network device is available
        if !has_network_device() {
            crate::task::yield_now().await;
            continue;
        }

        // Get network configuration
        let config = get_network_config();
        if !config.is_valid() {
            crate::task::yield_now().await;
            continue;
        }

        // Check if there are packets to transmit
        let packet = {
            let mut queue = TX_QUEUE.lock();
            queue.pop_front()
        };

        if let Some(tx_packet) = packet {
            // Process the packet
            serial_println!("TX: Processing packet for {}", tx_packet.dest_ip);
            if let Err(e) = process_tx_packet(tx_packet, config).await {
                serial_println!("TX: Failed to transmit packet: {:?}", e);
            }
        }

        // Yield to allow other tasks to run
        crate::task::yield_now().await;
    }
}

/// TX error types
#[derive(Debug)]
enum TxError {
    NoDevice,
    NoRouting,
    ArpFailed,
    TransmitFailed,
}

/// Process a single TX packet
async fn process_tx_packet(packet: TxPacket, config: NetworkConfig) -> Result<(), TxError> {
    // Get routing table
    let routing_table = RoutingTable::new(config.ip_addr, config.netmask, config.gateway);

    // Determine next hop
    let next_hop_opt = routing_table.next_hop(packet.dest_ip);
    let next_hop = match next_hop_opt {
        Some(ip) => ip,
        None => packet.dest_ip,
    };

    // Resolve MAC address via ARP
    let dest_mac = resolve_mac_address(next_hop).await?;

    // Get our MAC address
    let our_mac = match get_mac_address() {
        Some(mac) => mac,
        None => return Err(TxError::NoDevice),
    };

    // Build IPv4 packet
    let ip_header = Ipv4Header::new(
        config.ip_addr,
        packet.dest_ip,
        packet.protocol,
        packet.payload.len() as u16,
    );

    let mut ip_packet = ip_header.to_bytes();
    ip_packet.extend_from_slice(&packet.payload);

    // Build Ethernet frame
    let eth_frame = EthernetFrame::new(
        dest_mac,
        our_mac,
        ETHERTYPE_IPV4,
        ip_packet,
    ).map_err(|_| TxError::TransmitFailed)?;

    let frame_bytes = eth_frame.to_bytes();

    // Transmit
    transmit_packet(&frame_bytes).map_err(|_| TxError::TransmitFailed)?;

    Ok(())
}

/// Resolve MAC address for an IP address using ARP
///
/// First checks the ARP cache. If not found, sends an ARP request and waits.
async fn resolve_mac_address(ip: Ipv4Addr) -> Result<[u8; 6], TxError> {
    let current_time = 0; // Simplified timer, TBD
    
    // Check ARP cache first
    let cache = arp_cache();
    if let Some(mac) = cache.lookup(ip, current_time) {
        return Ok(mac);
    }

    // Not in cache, send ARP request
    send_arp_request(ip)?;

    // Wait for ARP reply (with timeout)
    for _ in 0..100 {
        crate::task::yield_now().await;

        // Check cache again
        if let Some(mac) = cache.lookup(ip, current_time) {
            return Ok(mac);
        }
    }

    // Timeout
    serial_println!("ARP: Timeout resolving {}", ip);
    Err(TxError::ArpFailed)
}

/// Send an ARP request
fn send_arp_request(target_ip: Ipv4Addr) -> Result<(), TxError> {
    serial_println!("ARP: send_arp_request called for {}", target_ip);
    
    let config = get_network_config();
    if !config.is_valid() {
        serial_println!("ARP: Network config invalid!");
        return Err(TxError::NoRouting);
    }

    // Get our MAC address
    let our_mac = match get_mac_address() {
        Some(mac) => mac,
        None => {
            serial_println!("ARP: No MAC address available!");
            return Err(TxError::NoDevice);
        }
    };

    serial_println!("ARP: Our MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", 
                    our_mac[0], our_mac[1], our_mac[2], our_mac[3], our_mac[4], our_mac[5]);

    // Create ARP request
    let arp_request = create_arp_request(config.ip_addr, our_mac, target_ip);
    serial_println!("ARP: Created request packet, {} bytes", arp_request.len());

    // Wrap in Ethernet frame (broadcast)
    let broadcast_mac = [0xFF; 6];
    let eth_frame = EthernetFrame::new(
        broadcast_mac,
        our_mac,
        ETHERTYPE_ARP,
        arp_request,
    ).map_err(|e| {
        serial_println!("ARP: Failed to create Ethernet frame: {:?}", e);
        TxError::TransmitFailed
    })?;

    let frame_bytes = eth_frame.to_bytes();
    serial_println!("ARP: Ethernet frame built, {} bytes total", frame_bytes.len());

    // Transmit
    transmit_packet(&frame_bytes).map_err(|e| {
        serial_println!("ARP: Transmit failed: {:?}", e);
        TxError::TransmitFailed
    })?;

    serial_println!("ARP: Successfully sent request for {}", target_ip);

    Ok(())
}

/// Send an ICMP echo request (ping)
///
/// # Arguments
/// * `dest_ip` - Destination IP address
/// * `identifier` - Ping identifier (typically process ID)
/// * `sequence` - Sequence number
/// * `data` - Optional payload data
///
/// # Returns
/// * `Ok(())` - Packet queued successfully
/// * `Err(())` - Failed to queue packet
pub fn send_ping(dest_ip: Ipv4Addr, identifier: u16, sequence: u16, data: Vec<u8>) -> Result<(), ()> {
    // Create ICMP echo request
    let icmp_packet = IcmpPacket::new_echo_request(identifier, sequence, data);
    let icmp_bytes = icmp_packet.to_bytes();

    // Queue for transmission
    queue_tx_packet(dest_ip, protocol::ICMP, icmp_bytes)
}

/// Initialize the network stack
///
/// This should be called after the network device is initialized.
/// It starts the RX and TX processing tasks.
///
/// # Arguments
/// * `executor` - The task executor to spawn network tasks on
pub fn init(executor: &mut crate::task::executor::Executor) {
    println!("Network stack initializing...");
    
    // Spawn RX processing task
    executor.spawn(crate::task::Task::new(rx_processing_task()));
    
    // Spawn TX processing task
    executor.spawn(crate::task::Task::new(tx_processing_task()));
    
    println!("Network stack initialized");
}

/// Display network configuration
pub fn display_config() {
    let config = get_network_config();
    
    if !config.is_valid() {
        println!("Network: Not configured");
        return;
    }

    println!("Network Configuration:");
    println!("  IP Address: {}", config.ip_addr);
    println!("  Netmask:    {}", config.netmask);
    
    if let Some(gateway) = config.gateway {
        println!("  Gateway:    {}", gateway);
    } else {
        println!("  Gateway:    None");
    }

    // Display MAC address if device is available
    if let Some(mac) = get_mac_address() {
        println!("  MAC:        {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                 mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    }
}

/// Display ARP cache
pub fn display_arp_cache() {
    let cache = arp_cache();
    let entries = cache.entries();
    
    if entries.is_empty() {
        println!("ARP cache is empty");
        return;
    }

    println!("ARP Cache:");
    println!("  IP Address       MAC Address");
    println!("  ----------------------------------------");
    
    for (ip, mac, _expires_at) in entries {
        println!("  {:15}  {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                 ip, mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    }
}
