//! UDP Protocol Implementation (RFC 768)
//! Phase 6.1 - Networking Roadmap
//!
//! Provides User Datagram Protocol support for connectionless, packet-based communication.

extern crate alloc;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::boxed::Box;
use core::net::Ipv4Addr;
use spin::Mutex;
use lazy_static::lazy_static;

use crate::serial_println;

/// UDP protocol number for IPv4
pub const UDP_PROTOCOL: u8 = 17;

/// Minimum UDP header size (8 bytes)
pub const UDP_HEADER_SIZE: usize = 8;

/// UDP port range for ephemeral (dynamic) port allocation
pub const EPHEMERAL_PORT_START: u16 = 49152;
pub const EPHEMERAL_PORT_END: u16 = 65535;

/// UDP packet structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UdpPacket {
    /// Source port (0-65535)
    pub src_port: u16,
    /// Destination port (0-65535)
    pub dest_port: u16,
    /// Length of UDP header + data (minimum 8 bytes)
    pub length: u16,
    /// Checksum (optional for IPv4, can be 0)
    pub checksum: u16,
    /// Payload data
    pub data: Vec<u8>,
}

/// Errors that can occur during UDP operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdpError {
    /// Packet is too short to contain a valid UDP header
    PacketTooShort,
    /// Length field doesn't match actual packet size
    InvalidLength,
    /// Checksum verification failed
    ChecksumMismatch,
    /// Invalid port number
    InvalidPort,
}

/// Socket bind errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindError {
    /// Port is already in use
    PortInUse,
    /// Invalid port number (0 is reserved)
    InvalidPort,
    /// No ephemeral ports available
    NoPortsAvailable,
}

/// Socket send errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendError {
    /// Network device not available
    NoDevice,
    /// Network not configured
    NotConfigured,
    /// Failed to queue packet for transmission
    QueueFull,
}

/// Socket receive errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecvError {
    /// No data available
    WouldBlock,
}

impl UdpPacket {
    /// Create a new UDP packet
    ///
    /// # Arguments
    /// * `src_port` - Source port number
    /// * `dest_port` - Destination port number
    /// * `data` - Payload data
    ///
    /// # Returns
    /// A new UDP packet with calculated length (checksum will be 0)
    pub fn new(src_port: u16, dest_port: u16, data: Vec<u8>) -> Self {
        let length = (UDP_HEADER_SIZE + data.len()) as u16;
        Self {
            src_port,
            dest_port,
            length,
            checksum: 0,
            data,
        }
    }

    /// Parse a UDP packet from raw bytes
    ///
    /// # Arguments
    /// * `bytes` - Raw packet bytes (UDP header + data)
    ///
    /// # Returns
    /// * `Ok(UdpPacket)` - Successfully parsed packet
    /// * `Err(UdpError)` - Parsing failed
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, UdpError> {
        if bytes.len() < UDP_HEADER_SIZE {
            return Err(UdpError::PacketTooShort);
        }

        let src_port = u16::from_be_bytes([bytes[0], bytes[1]]);
        let dest_port = u16::from_be_bytes([bytes[2], bytes[3]]);
        let length = u16::from_be_bytes([bytes[4], bytes[5]]);
        let checksum = u16::from_be_bytes([bytes[6], bytes[7]]);

        // Validate length
        if length < UDP_HEADER_SIZE as u16 || length as usize > bytes.len() {
            return Err(UdpError::InvalidLength);
        }

        let data = bytes[UDP_HEADER_SIZE..length as usize].to_vec();

        Ok(Self {
            src_port,
            dest_port,
            length,
            checksum,
            data,
        })
    }

    /// Serialize the UDP packet to bytes
    ///
    /// # Returns
    /// Raw packet bytes (UDP header + data)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.length as usize);

        // Source port (2 bytes)
        bytes.extend_from_slice(&self.src_port.to_be_bytes());
        
        // Destination port (2 bytes)
        bytes.extend_from_slice(&self.dest_port.to_be_bytes());
        
        // Length (2 bytes)
        bytes.extend_from_slice(&self.length.to_be_bytes());
        
        // Checksum (2 bytes)
        bytes.extend_from_slice(&self.checksum.to_be_bytes());
        
        // Data
        bytes.extend_from_slice(&self.data);

        bytes
    }

    /// Calculate UDP checksum with pseudo-header
    ///
    /// # Arguments
    /// * `src_ip` - Source IPv4 address
    /// * `dest_ip` - Destination IPv4 address
    ///
    /// # Returns
    /// Calculated checksum value
    pub fn calculate_checksum(&self, src_ip: Ipv4Addr, dest_ip: Ipv4Addr) -> u16 {
        let mut sum: u32 = 0;

        // Pseudo-header: source IP (4 bytes)
        let src_octets = src_ip.octets();
        sum += u16::from_be_bytes([src_octets[0], src_octets[1]]) as u32;
        sum += u16::from_be_bytes([src_octets[2], src_octets[3]]) as u32;

        // Pseudo-header: destination IP (4 bytes)
        let dest_octets = dest_ip.octets();
        sum += u16::from_be_bytes([dest_octets[0], dest_octets[1]]) as u32;
        sum += u16::from_be_bytes([dest_octets[2], dest_octets[3]]) as u32;

        // Pseudo-header: zero (1 byte) + protocol (1 byte)
        sum += UDP_PROTOCOL as u32;

        // Pseudo-header: UDP length (2 bytes)
        sum += self.length as u32;

        // UDP header and data
        let packet_bytes = self.to_bytes();
        for chunk in packet_bytes.chunks(2) {
            let word = if chunk.len() == 2 {
                u16::from_be_bytes([chunk[0], chunk[1]])
            } else {
                u16::from_be_bytes([chunk[0], 0])
            };
            sum += word as u32;
        }

        // Fold 32-bit sum to 16 bits
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        // One's complement
        !sum as u16
    }

    /// Verify UDP checksum
    ///
    /// # Arguments
    /// * `src_ip` - Source IPv4 address
    /// * `dest_ip` - Destination IPv4 address
    ///
    /// # Returns
    /// `true` if checksum is valid or checksum is 0 (checksum disabled)
    pub fn verify_checksum(&self, src_ip: Ipv4Addr, dest_ip: Ipv4Addr) -> bool {
        // Checksum of 0 means checksum is disabled (valid)
        if self.checksum == 0 {
            return true;
        }

        let calculated = self.calculate_checksum(src_ip, dest_ip);
        calculated == 0
    }
}

/// UDP socket for sending and receiving datagrams
pub struct UdpSocket {
    /// Local port this socket is bound to
    local_port: u16,
    /// Queue of received packets: (source_ip, source_port, data)
    rx_queue: Mutex<VecDeque<(Ipv4Addr, u16, Vec<u8>)>>,
    /// Maximum receive queue size
    max_queue_size: usize,
}

impl UdpSocket {
    /// Bind a UDP socket to a specific port
    ///
    /// # Arguments
    /// * `port` - Port number to bind to (0 = allocate ephemeral port)
    ///
    /// # Returns
    /// * `Ok(UdpSocket)` - Successfully bound socket
    /// * `Err(BindError)` - Binding failed
    pub fn bind(port: u16) -> Result<Self, BindError> {
        let mut registry = PORT_REGISTRY.lock();

        let actual_port = if port == 0 {
            // Allocate ephemeral port
            registry.allocate_ephemeral()?
        } else {
            // Bind to specific port
            if registry.is_bound(port) {
                return Err(BindError::PortInUse);
            }
            registry.bind(port)?;
            port
        };

        serial_println!("UDP: Bound socket to port {}", actual_port);

        Ok(Self {
            local_port: actual_port,
            rx_queue: Mutex::new(VecDeque::with_capacity(64)),
            max_queue_size: 64,
        })
    }

    /// Get the local port this socket is bound to
    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    /// Send data to a remote address
    ///
    /// # Arguments
    /// * `data` - Data to send
    /// * `dest_ip` - Destination IP address
    /// * `dest_port` - Destination port
    ///
    /// # Returns
    /// * `Ok(())` - Data queued for transmission
    /// * `Err(SendError)` - Send failed
    pub fn send_to(&self, data: &[u8], dest_ip: Ipv4Addr, dest_port: u16) -> Result<(), SendError> {
        serial_println!("UDP: Sending {} bytes from port {} to {}:{}", 
                       data.len(), self.local_port, dest_ip, dest_port);

        // Get network configuration
        let config = crate::net::stack::get_network_config();
        if !config.is_valid() {
            return Err(SendError::NotConfigured);
        }

        // Create UDP packet
        let mut packet = UdpPacket::new(self.local_port, dest_port, data.to_vec());
        
        // Calculate and set checksum (optional for IPv4, but we'll include it)
        packet.checksum = packet.calculate_checksum(config.ip_addr, dest_ip);

        // Serialize to bytes
        let udp_bytes = packet.to_bytes();

        // Queue for transmission via IP layer
        crate::net::stack::queue_tx_packet(dest_ip, UDP_PROTOCOL, udp_bytes)
            .map_err(|_| SendError::QueueFull)
    }

    /// Receive data from the socket (non-blocking)
    ///
    /// # Returns
    /// * `Ok((data, source_ip, source_port))` - Received datagram
    /// * `Err(RecvError::WouldBlock)` - No data available
    pub fn recv_from(&self) -> Result<(Vec<u8>, Ipv4Addr, u16), RecvError> {
        let mut queue = self.rx_queue.lock();
        queue.pop_front()
            .map(|(src_ip, src_port, data)| (data, src_ip, src_port))
            .ok_or(RecvError::WouldBlock)
    }

    /// Internal method to deliver received data to this socket
    pub(crate) fn deliver(&self, src_ip: Ipv4Addr, src_port: u16, data: Vec<u8>) {
        let mut queue = self.rx_queue.lock();
        
        if queue.len() >= self.max_queue_size {
            serial_println!("UDP: Socket port {} RX queue full, dropping packet", self.local_port);
            return;
        }

        let data_len = data.len();
        queue.push_back((src_ip, src_port, data));
        serial_println!("UDP: Delivered {} bytes to socket port {}", data_len, self.local_port);
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        // Unbind port when socket is dropped
        PORT_REGISTRY.lock().unbind(self.local_port);
        serial_println!("UDP: Unbound port {}", self.local_port);
    }
}

/// Global UDP port registry
struct PortRegistry {
    /// Set of bound ports
    bound_ports: BTreeMap<u16, ()>,
    /// Next ephemeral port to try
    next_ephemeral: u16,
}

impl PortRegistry {
    const fn new() -> Self {
        Self {
            bound_ports: BTreeMap::new(),
            next_ephemeral: EPHEMERAL_PORT_START,
        }
    }

    /// Check if a port is bound
    fn is_bound(&self, port: u16) -> bool {
        self.bound_ports.contains_key(&port)
    }

    /// Bind a specific port
    fn bind(&mut self, port: u16) -> Result<(), BindError> {
        if port == 0 {
            return Err(BindError::InvalidPort);
        }

        if self.is_bound(port) {
            return Err(BindError::PortInUse);
        }

        self.bound_ports.insert(port, ());
        Ok(())
    }

    /// Unbind a port
    fn unbind(&mut self, port: u16) {
        self.bound_ports.remove(&port);
    }

    /// Allocate an ephemeral port
    fn allocate_ephemeral(&mut self) -> Result<u16, BindError> {
        let start = self.next_ephemeral;
        
        loop {
            let port = self.next_ephemeral;
            
            // Advance to next port
            self.next_ephemeral += 1;
            if self.next_ephemeral > EPHEMERAL_PORT_END {
                self.next_ephemeral = EPHEMERAL_PORT_START;
            }

            // Check if this port is available
            if !self.is_bound(port) {
                self.bind(port)?;
                return Ok(port);
            }

            // If we've wrapped around, no ports available
            if self.next_ephemeral == start {
                return Err(BindError::NoPortsAvailable);
            }
        }
    }
}

lazy_static! {
    /// Global port registry for tracking bound UDP ports
    static ref PORT_REGISTRY: Mutex<PortRegistry> = Mutex::new(PortRegistry::new());
    
    /// Global registry of active UDP sockets by port
    static ref SOCKET_REGISTRY: Mutex<BTreeMap<u16, Box<UdpSocket>>> = Mutex::new(BTreeMap::new());
}

/// Register a UDP socket in the global registry
///
/// This allows the network stack to deliver incoming packets to the correct socket.
pub fn register_socket(socket: Box<UdpSocket>) {
    let port = socket.local_port();
    SOCKET_REGISTRY.lock().insert(port, socket);
    serial_println!("UDP: Registered socket on port {}", port);
}

/// Unregister a UDP socket from the global registry
pub fn unregister_socket(port: u16) {
    SOCKET_REGISTRY.lock().remove(&port);
    serial_println!("UDP: Unregistered socket on port {}", port);
}

/// Handle incoming UDP packet
///
/// Called by the network stack when a UDP packet is received.
///
/// # Arguments
/// * `src_ip` - Source IPv4 address
/// * `dest_ip` - Destination IPv4 address
/// * `data` - UDP packet bytes
pub fn handle_udp_packet(src_ip: Ipv4Addr, dest_ip: Ipv4Addr, data: &[u8]) {
    serial_println!("UDP: Received packet from {} ({} bytes)", src_ip, data.len());

    // Parse UDP packet
    let packet = match UdpPacket::from_bytes(data) {
        Ok(p) => p,
        Err(e) => {
            serial_println!("UDP: Failed to parse packet: {:?}", e);
            return;
        }
    };

    serial_println!("UDP: Packet from {}:{} to {}:{} ({} bytes data)",
                   src_ip, packet.src_port, dest_ip, packet.dest_port, packet.data.len());

    // Verify checksum if present
    if packet.checksum != 0 && !packet.verify_checksum(src_ip, dest_ip) {
        serial_println!("UDP: Checksum verification failed");
        return;
    }

    // Deliver to socket if bound
    let registry = SOCKET_REGISTRY.lock();
    if let Some(socket) = registry.get(&packet.dest_port) {
        socket.deliver(src_ip, packet.src_port, packet.data);
    } else {
        serial_println!("UDP: No socket bound to port {}, dropping packet", packet.dest_port);
    }
}
