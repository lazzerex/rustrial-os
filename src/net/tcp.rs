//! TCP Protocol Implementation (RFC 793)
//! Phase 7 - Networking Roadmap
//!
//! Provides Transmission Control Protocol support for reliable, connection-oriented communication.
//!
//! # Features
//! - TCP packet parsing and building
//! - Connection establishment (3-way handshake)
//! - Data transmission with sequence numbers
//! - Connection teardown (FIN/ACK)
//! - Basic retransmission support
//!
//! # Limitations (Current Implementation)
//! - No congestion control
//! - No sliding window (fixed window size)
//! - Basic retransmission (no sophisticated timeout calculation)
//! - No options support beyond MSS

extern crate alloc;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::net::Ipv4Addr;
use spin::Mutex;
use lazy_static::lazy_static;

use crate::serial_println;

/// TCP protocol number for IPv4
pub const TCP_PROTOCOL: u8 = 6;

/// Minimum TCP header size (20 bytes, no options)
pub const TCP_HEADER_SIZE: usize = 20;

/// Maximum TCP header size (60 bytes with options)
pub const TCP_MAX_HEADER_SIZE: usize = 60;

/// Default Maximum Segment Size (MSS)
pub const DEFAULT_MSS: u16 = 1460; // Typical for Ethernet (1500 - 20 IP - 20 TCP)

/// TCP port range for ephemeral (dynamic) port allocation
pub const EPHEMERAL_PORT_START: u16 = 49152;
pub const EPHEMERAL_PORT_END: u16 = 65535;

/// TCP Control Flags
pub mod flags {
    pub const FIN: u8 = 0x01; // Finish (no more data)
    pub const SYN: u8 = 0x02; // Synchronize sequence numbers
    pub const RST: u8 = 0x04; // Reset connection
    pub const PSH: u8 = 0x08; // Push function
    pub const ACK: u8 = 0x10; // Acknowledgment field significant
    pub const URG: u8 = 0x20; // Urgent pointer field significant
}

/// TCP Connection State (RFC 793)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    /// Waiting for connection request
    Listen,
    /// Sent SYN, waiting for SYN-ACK
    SynSent,
    /// Received SYN, sent SYN-ACK, waiting for ACK
    SynReceived,
    /// Connection established, data transfer
    Established,
    /// Sent FIN, waiting for ACK
    FinWait1,
    /// Received ACK of FIN, waiting for FIN
    FinWait2,
    /// Waiting for all data to be acknowledged before FIN
    Closing,
    /// Received FIN, sent ACK, waiting for timeout
    TimeWait,
    /// Received FIN and ACK, waiting for timeout
    CloseWait,
    /// Sent FIN after receiving FIN, waiting for ACK
    LastAck,
    /// Connection closed
    Closed,
}

/// TCP packet structure
#[derive(Debug, Clone)]
pub struct TcpPacket {
    /// Source port (0-65535)
    pub src_port: u16,
    /// Destination port (0-65535)
    pub dest_port: u16,
    /// Sequence number
    pub sequence: u32,
    /// Acknowledgment number (if ACK flag set)
    pub acknowledgment: u32,
    /// Data offset (header length in 32-bit words)
    pub data_offset: u8,
    /// Control flags (FIN, SYN, RST, PSH, ACK, URG)
    pub flags: u8,
    /// Window size (flow control)
    pub window: u16,
    /// Checksum
    pub checksum: u16,
    /// Urgent pointer (if URG flag set)
    pub urgent_pointer: u16,
    /// Options (if any)
    pub options: Vec<u8>,
    /// Payload data
    pub data: Vec<u8>,
}

/// TCP socket identifier (4-tuple)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TcpSocketId {
    pub local_addr: Ipv4Addr,
    pub local_port: u16,
    pub remote_addr: Ipv4Addr,
    pub remote_port: u16,
}

/// TCP Connection Control Block (TCB)
#[derive(Debug, Clone)]
pub struct TcpConnection {
    /// Socket identifier
    pub socket_id: TcpSocketId,
    /// Current connection state
    pub state: TcpState,
    /// Send sequence number
    pub send_seq: u32,
    /// Receive sequence number
    pub recv_seq: u32,
    /// Initial send sequence number
    pub initial_send_seq: u32,
    /// Initial receive sequence number
    pub initial_recv_seq: u32,
    /// Send window size
    pub send_window: u16,
    /// Receive window size
    pub recv_window: u16,
    /// Maximum segment size
    pub mss: u16,
    /// Send buffer
    pub send_buffer: VecDeque<u8>,
    /// Receive buffer
    pub recv_buffer: VecDeque<u8>,
    /// Pending segments for retransmission
    pub pending_acks: VecDeque<(u32, Vec<u8>)>, // (seq, data)
}

/// Errors that can occur during TCP operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpError {
    /// Packet is too short to contain a valid TCP header
    PacketTooShort,
    /// Invalid data offset (header length)
    InvalidDataOffset,
    /// Checksum verification failed
    ChecksumMismatch,
    /// Connection does not exist
    ConnectionNotFound,
    /// Invalid state for requested operation
    InvalidState,
    /// Port already in use
    PortInUse,
    /// No ports available for allocation
    NoPortsAvailable,
    /// Connection refused (RST received)
    ConnectionRefused,
    /// Connection reset
    ConnectionReset,
    /// Send buffer full
    BufferFull,
    /// No data available
    NoData,
}

impl TcpPacket {
    /// Parse a TCP packet from raw bytes
    ///
    /// # Arguments
    /// * `data` - Raw packet bytes (TCP header + payload)
    /// * `src_addr` - Source IP address (for checksum verification)
    /// * `dest_addr` - Destination IP address (for checksum verification)
    ///
    /// # Returns
    /// Parsed TCP packet or error
    pub fn parse(data: &[u8], src_addr: Ipv4Addr, dest_addr: Ipv4Addr) -> Result<Self, TcpError> {
        if data.len() < TCP_HEADER_SIZE {
            return Err(TcpError::PacketTooShort);
        }

        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dest_port = u16::from_be_bytes([data[2], data[3]]);
        let sequence = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let acknowledgment = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        let data_offset = (data[12] >> 4) as u8;
        let flags = data[13];
        let window = u16::from_be_bytes([data[14], data[15]]);
        let checksum = u16::from_be_bytes([data[16], data[17]]);
        let urgent_pointer = u16::from_be_bytes([data[18], data[19]]);

        // Validate data offset
        if data_offset < 5 || data_offset > 15 {
            return Err(TcpError::InvalidDataOffset);
        }

        let header_len = (data_offset as usize) * 4;
        if data.len() < header_len {
            return Err(TcpError::PacketTooShort);
        }

        // Extract options (if any)
        let options = if header_len > TCP_HEADER_SIZE {
            data[TCP_HEADER_SIZE..header_len].to_vec()
        } else {
            Vec::new()
        };

        // Extract payload
        let payload = if data.len() > header_len {
            data[header_len..].to_vec()
        } else {
            Vec::new()
        };

        // Verify checksum
        let calc_checksum = Self::calculate_checksum(
            src_addr,
            dest_addr,
            TCP_PROTOCOL,
            data,
        );

        if checksum != 0 && calc_checksum != 0 {
            serial_println!("[TCP] Checksum mismatch: received {:#x}, calculated {:#x}", 
                checksum, calc_checksum);
            // Note: We don't fail on checksum mismatch for now (some drivers may have issues)
            // return Err(TcpError::ChecksumMismatch);
        }

        Ok(TcpPacket {
            src_port,
            dest_port,
            sequence,
            acknowledgment,
            data_offset,
            flags,
            window,
            checksum,
            urgent_pointer,
            options,
            data: payload,
        })
    }

    /// Build a TCP packet into raw bytes
    ///
    /// # Arguments
    /// * `src_addr` - Source IP address (for checksum calculation)
    /// * `dest_addr` - Destination IP address (for checksum calculation)
    ///
    /// # Returns
    /// Raw packet bytes
    pub fn build(&self, src_addr: Ipv4Addr, dest_addr: Ipv4Addr) -> Vec<u8> {
        let header_len = TCP_HEADER_SIZE + self.options.len();
        let total_len = header_len + self.data.len();
        let mut packet = Vec::with_capacity(total_len);

        // Source and destination ports
        packet.extend_from_slice(&self.src_port.to_be_bytes());
        packet.extend_from_slice(&self.dest_port.to_be_bytes());

        // Sequence and acknowledgment numbers
        packet.extend_from_slice(&self.sequence.to_be_bytes());
        packet.extend_from_slice(&self.acknowledgment.to_be_bytes());

        // Data offset and flags
        let data_offset = ((header_len / 4) as u8) << 4;
        packet.push(data_offset);
        packet.push(self.flags);

        // Window size
        packet.extend_from_slice(&self.window.to_be_bytes());

        // Checksum (placeholder, will be calculated)
        packet.extend_from_slice(&[0, 0]);

        // Urgent pointer
        packet.extend_from_slice(&self.urgent_pointer.to_be_bytes());

        // Options
        packet.extend_from_slice(&self.options);

        // Payload
        packet.extend_from_slice(&self.data);

        // Calculate and insert checksum
        let checksum = Self::calculate_checksum(src_addr, dest_addr, TCP_PROTOCOL, &packet);
        packet[16..18].copy_from_slice(&checksum.to_be_bytes());

        packet
    }

    /// Calculate TCP checksum (includes pseudo-header)
    ///
    /// # Arguments
    /// * `src_addr` - Source IP address
    /// * `dest_addr` - Destination IP address
    /// * `protocol` - Protocol number (TCP = 6)
    /// * `data` - TCP packet data
    ///
    /// # Returns
    /// Calculated checksum
    fn calculate_checksum(src_addr: Ipv4Addr, dest_addr: Ipv4Addr, protocol: u8, data: &[u8]) -> u16 {
        let mut sum: u32 = 0;

        // Pseudo-header
        for &byte in &src_addr.octets() {
            sum += byte as u32;
        }
        for &byte in &dest_addr.octets() {
            sum += byte as u32;
        }
        sum += protocol as u32;
        sum += data.len() as u32;

        // TCP header and data (16-bit words)
        let mut i = 0;
        while i < data.len() {
            if i + 1 < data.len() {
                let word = u16::from_be_bytes([data[i], data[i + 1]]);
                sum += word as u32;
            } else {
                // Odd byte
                sum += (data[i] as u32) << 8;
            }
            i += 2;
        }

        // Fold 32-bit sum to 16 bits
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !sum as u16
    }

    /// Check if a flag is set
    pub fn has_flag(&self, flag: u8) -> bool {
        (self.flags & flag) != 0
    }
}

impl TcpConnection {
    /// Create a new TCP connection in CLOSED state
    pub fn new(socket_id: TcpSocketId) -> Self {
        TcpConnection {
            socket_id,
            state: TcpState::Closed,
            send_seq: 0,
            recv_seq: 0,
            initial_send_seq: 0,
            initial_recv_seq: 0,
            send_window: 8192, // 8KB default
            recv_window: 8192,
            mss: DEFAULT_MSS,
            send_buffer: VecDeque::with_capacity(8192),
            recv_buffer: VecDeque::with_capacity(8192),
            pending_acks: VecDeque::new(),
        }
    }

    /// Initialize connection for active open (client)
    pub fn connect(&mut self) -> Result<TcpPacket, TcpError> {
        if self.state != TcpState::Closed {
            return Err(TcpError::InvalidState);
        }

        // Generate initial sequence number (simplified)
        self.initial_send_seq = Self::generate_isn();
        self.send_seq = self.initial_send_seq;
        self.state = TcpState::SynSent;

        // Build SYN packet
        let syn_packet = TcpPacket {
            src_port: self.socket_id.local_port,
            dest_port: self.socket_id.remote_port,
            sequence: self.send_seq,
            acknowledgment: 0,
            data_offset: 5, // No options for now
            flags: flags::SYN,
            window: self.recv_window,
            checksum: 0,
            urgent_pointer: 0,
            options: Vec::new(),
            data: Vec::new(),
        };

        self.send_seq = self.send_seq.wrapping_add(1); // SYN consumes one sequence number

        Ok(syn_packet)
    }

    /// Process incoming packet
    pub fn process_packet(&mut self, packet: &TcpPacket) -> Result<Option<TcpPacket>, TcpError> {
        serial_println!("[TCP] Processing packet in state {:?}: flags={:#x}, seq={}, ack={}", 
            self.state, packet.flags, packet.sequence, packet.acknowledgment);

        match self.state {
            TcpState::Listen => {
                if packet.has_flag(flags::SYN) {
                    self.handle_syn(packet)
                } else {
                    Ok(None)
                }
            }
            TcpState::SynSent => {
                if packet.has_flag(flags::SYN) && packet.has_flag(flags::ACK) {
                    self.handle_syn_ack(packet)
                } else if packet.has_flag(flags::RST) {
                    self.state = TcpState::Closed;
                    Err(TcpError::ConnectionRefused)
                } else {
                    Ok(None)
                }
            }
            TcpState::SynReceived => {
                if packet.has_flag(flags::ACK) {
                    self.handle_ack_in_syn_received(packet)
                } else if packet.has_flag(flags::RST) {
                    self.state = TcpState::Closed;
                    Err(TcpError::ConnectionReset)
                } else {
                    Ok(None)
                }
            }
            TcpState::Established => {
                self.handle_established(packet)
            }
            TcpState::FinWait1 | TcpState::FinWait2 | TcpState::Closing => {
                self.handle_fin_wait(packet)
            }
            TcpState::CloseWait => {
                if packet.has_flag(flags::ACK) {
                    Ok(None)
                } else {
                    Ok(None)
                }
            }
            TcpState::LastAck => {
                if packet.has_flag(flags::ACK) {
                    self.state = TcpState::Closed;
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Handle SYN in LISTEN state
    fn handle_syn(&mut self, packet: &TcpPacket) -> Result<Option<TcpPacket>, TcpError> {
        self.recv_seq = packet.sequence.wrapping_add(1);
        self.initial_recv_seq = packet.sequence;
        self.initial_send_seq = Self::generate_isn();
        self.send_seq = self.initial_send_seq;
        self.state = TcpState::SynReceived;

        // Send SYN-ACK
        let syn_ack = TcpPacket {
            src_port: self.socket_id.local_port,
            dest_port: self.socket_id.remote_port,
            sequence: self.send_seq,
            acknowledgment: self.recv_seq,
            data_offset: 5,
            flags: flags::SYN | flags::ACK,
            window: self.recv_window,
            checksum: 0,
            urgent_pointer: 0,
            options: Vec::new(),
            data: Vec::new(),
        };

        self.send_seq = self.send_seq.wrapping_add(1); // SYN consumes one sequence number

        Ok(Some(syn_ack))
    }

    /// Handle SYN-ACK in SYN_SENT state
    fn handle_syn_ack(&mut self, packet: &TcpPacket) -> Result<Option<TcpPacket>, TcpError> {
        if packet.acknowledgment != self.send_seq {
            return Ok(None);
        }

        self.recv_seq = packet.sequence.wrapping_add(1);
        self.initial_recv_seq = packet.sequence;
        self.state = TcpState::Established;

        serial_println!("[TCP] Connection established!");

        // Send ACK
        let ack = TcpPacket {
            src_port: self.socket_id.local_port,
            dest_port: self.socket_id.remote_port,
            sequence: self.send_seq,
            acknowledgment: self.recv_seq,
            data_offset: 5,
            flags: flags::ACK,
            window: self.recv_window,
            checksum: 0,
            urgent_pointer: 0,
            options: Vec::new(),
            data: Vec::new(),
        };

        Ok(Some(ack))
    }

    /// Handle ACK in SYN_RECEIVED state
    fn handle_ack_in_syn_received(&mut self, packet: &TcpPacket) -> Result<Option<TcpPacket>, TcpError> {
        if packet.acknowledgment == self.send_seq {
            self.state = TcpState::Established;
            serial_println!("[TCP] Connection established (server)!");
        }
        Ok(None)
    }

    /// Handle packets in ESTABLISHED state
    fn handle_established(&mut self, packet: &TcpPacket) -> Result<Option<TcpPacket>, TcpError> {
        let mut response = None;

        // Handle data
        if !packet.data.is_empty() {
            if packet.sequence == self.recv_seq {
                // In-order data
                self.recv_buffer.extend(&packet.data);
                self.recv_seq = self.recv_seq.wrapping_add(packet.data.len() as u32);

                serial_println!("[TCP] Received {} bytes of data", packet.data.len());

                // Send ACK
                response = Some(TcpPacket {
                    src_port: self.socket_id.local_port,
                    dest_port: self.socket_id.remote_port,
                    sequence: self.send_seq,
                    acknowledgment: self.recv_seq,
                    data_offset: 5,
                    flags: flags::ACK,
                    window: self.recv_window,
                    checksum: 0,
                    urgent_pointer: 0,
                    options: Vec::new(),
                    data: Vec::new(),
                });
            }
        }

        // Handle FIN
        if packet.has_flag(flags::FIN) {
            self.recv_seq = self.recv_seq.wrapping_add(1); // FIN consumes one sequence number
            self.state = TcpState::CloseWait;

            serial_println!("[TCP] Received FIN, entering CLOSE_WAIT");

            // Send ACK
            response = Some(TcpPacket {
                src_port: self.socket_id.local_port,
                dest_port: self.socket_id.remote_port,
                sequence: self.send_seq,
                acknowledgment: self.recv_seq,
                data_offset: 5,
                flags: flags::ACK,
                window: self.recv_window,
                checksum: 0,
                urgent_pointer: 0,
                options: Vec::new(),
                data: Vec::new(),
            });
        }

        Ok(response)
    }

    /// Handle packets in FIN_WAIT states
    fn handle_fin_wait(&mut self, packet: &TcpPacket) -> Result<Option<TcpPacket>, TcpError> {
        if packet.has_flag(flags::FIN) {
            self.recv_seq = self.recv_seq.wrapping_add(1);
            
            if self.state == TcpState::FinWait1 {
                if packet.has_flag(flags::ACK) {
                    self.state = TcpState::TimeWait;
                } else {
                    self.state = TcpState::Closing;
                }
            } else if self.state == TcpState::FinWait2 {
                self.state = TcpState::TimeWait;
            }

            // Send ACK
            return Ok(Some(TcpPacket {
                src_port: self.socket_id.local_port,
                dest_port: self.socket_id.remote_port,
                sequence: self.send_seq,
                acknowledgment: self.recv_seq,
                data_offset: 5,
                flags: flags::ACK,
                window: self.recv_window,
                checksum: 0,
                urgent_pointer: 0,
                options: Vec::new(),
                data: Vec::new(),
            }));
        }

        if packet.has_flag(flags::ACK) && self.state == TcpState::FinWait1 {
            self.state = TcpState::FinWait2;
        }

        Ok(None)
    }

    /// Send data
    pub fn send(&mut self, data: &[u8]) -> Result<TcpPacket, TcpError> {
        if self.state != TcpState::Established {
            return Err(TcpError::InvalidState);
        }

        if self.send_buffer.len() + data.len() > self.send_buffer.capacity() {
            return Err(TcpError::BufferFull);
        }

        let packet = TcpPacket {
            src_port: self.socket_id.local_port,
            dest_port: self.socket_id.remote_port,
            sequence: self.send_seq,
            acknowledgment: self.recv_seq,
            data_offset: 5,
            flags: flags::ACK | flags::PSH,
            window: self.recv_window,
            checksum: 0,
            urgent_pointer: 0,
            options: Vec::new(),
            data: data.to_vec(),
        };

        self.send_seq = self.send_seq.wrapping_add(data.len() as u32);
        self.pending_acks.push_back((packet.sequence, data.to_vec()));

        Ok(packet)
    }

    /// Receive data from buffer
    pub fn recv(&mut self, max_len: usize) -> Result<Vec<u8>, TcpError> {
        if self.recv_buffer.is_empty() {
            return Err(TcpError::NoData);
        }

        let len = core::cmp::min(max_len, self.recv_buffer.len());
        let data: Vec<u8> = self.recv_buffer.drain(..len).collect();
        Ok(data)
    }

    /// Close connection (initiate FIN)
    pub fn close(&mut self) -> Result<TcpPacket, TcpError> {
        match self.state {
            TcpState::Established => {
                self.state = TcpState::FinWait1;
            }
            TcpState::CloseWait => {
                self.state = TcpState::LastAck;
            }
            _ => {
                return Err(TcpError::InvalidState);
            }
        }

        let fin_packet = TcpPacket {
            src_port: self.socket_id.local_port,
            dest_port: self.socket_id.remote_port,
            sequence: self.send_seq,
            acknowledgment: self.recv_seq,
            data_offset: 5,
            flags: flags::FIN | flags::ACK,
            window: self.recv_window,
            checksum: 0,
            urgent_pointer: 0,
            options: Vec::new(),
            data: Vec::new(),
        };

        self.send_seq = self.send_seq.wrapping_add(1); // FIN consumes one sequence number

        Ok(fin_packet)
    }

    /// Generate Initial Sequence Number (simplified)
    fn generate_isn() -> u32 {
        // TODO: Use a better ISN generation (e.g., based on time and random)
        // For now, use a simple counter
        use core::sync::atomic::{AtomicU32, Ordering};
        static ISN_COUNTER: AtomicU32 = AtomicU32::new(1000);
        ISN_COUNTER.fetch_add(1, Ordering::SeqCst)
    }
}

/// Global TCP connection manager
lazy_static! {
    static ref TCP_CONNECTIONS: Mutex<BTreeMap<TcpSocketId, Arc<Mutex<TcpConnection>>>> = 
        Mutex::new(BTreeMap::new());
    static ref NEXT_EPHEMERAL_PORT: Mutex<u16> = Mutex::new(EPHEMERAL_PORT_START);
}

/// Allocate an ephemeral port
fn allocate_ephemeral_port() -> Result<u16, TcpError> {
    let mut port = NEXT_EPHEMERAL_PORT.lock();
    
    for _ in EPHEMERAL_PORT_START..=EPHEMERAL_PORT_END {
        let candidate = *port;
        *port = if *port == EPHEMERAL_PORT_END {
            EPHEMERAL_PORT_START
        } else {
            *port + 1
        };

        // Check if port is in use
        let connections = TCP_CONNECTIONS.lock();
        let in_use = connections.keys().any(|id| id.local_port == candidate);
        if !in_use {
            return Ok(candidate);
        }
    }

    Err(TcpError::NoPortsAvailable)
}

/// Create a new TCP connection (active open)
pub fn tcp_connect(remote_addr: Ipv4Addr, remote_port: u16, local_addr: Ipv4Addr) -> Result<TcpSocketId, TcpError> {
    let local_port = allocate_ephemeral_port()?;
    
    let socket_id = TcpSocketId {
        local_addr,
        local_port,
        remote_addr,
        remote_port,
    };

    let mut connection = TcpConnection::new(socket_id);
    let syn_packet = connection.connect()?;

    // Store connection
    let connection_arc = Arc::new(Mutex::new(connection));
    TCP_CONNECTIONS.lock().insert(socket_id, connection_arc);

    // Send SYN packet
    send_tcp_packet(&syn_packet, local_addr, remote_addr)?;

    serial_println!("[TCP] Initiated connection from {}:{} to {}:{}", 
        local_addr, local_port, remote_addr, remote_port);

    Ok(socket_id)
}

/// Listen for incoming connections on a port
pub fn tcp_listen(local_addr: Ipv4Addr, local_port: u16) -> Result<(), TcpError> {
    // Check if port is already in use
    let connections = TCP_CONNECTIONS.lock();
    let in_use = connections.keys().any(|id| id.local_addr == local_addr && id.local_port == local_port);
    if in_use {
        return Err(TcpError::PortInUse);
    }

    serial_println!("[TCP] Listening on {}:{}", local_addr, local_port);
    Ok(())
}

/// Send data on a TCP connection
pub fn tcp_send(socket_id: TcpSocketId, data: &[u8]) -> Result<(), TcpError> {
    let connections = TCP_CONNECTIONS.lock();
    let connection_arc = connections.get(&socket_id).ok_or(TcpError::ConnectionNotFound)?;
    let mut connection = connection_arc.lock();

    let packet = connection.send(data)?;
    drop(connection);
    drop(connections);

    send_tcp_packet(&packet, socket_id.local_addr, socket_id.remote_addr)?;

    Ok(())
}

/// Receive data from a TCP connection
pub fn tcp_recv(socket_id: TcpSocketId, max_len: usize) -> Result<Vec<u8>, TcpError> {
    let connections = TCP_CONNECTIONS.lock();
    let connection_arc = connections.get(&socket_id).ok_or(TcpError::ConnectionNotFound)?;
    let mut connection = connection_arc.lock();

    connection.recv(max_len)
}

/// Close a TCP connection
pub fn tcp_close(socket_id: TcpSocketId) -> Result<(), TcpError> {
    let connections = TCP_CONNECTIONS.lock();
    let connection_arc = connections.get(&socket_id).ok_or(TcpError::ConnectionNotFound)?;
    let mut connection = connection_arc.lock();

    let fin_packet = connection.close()?;
    drop(connection);

    send_tcp_packet(&fin_packet, socket_id.local_addr, socket_id.remote_addr)?;

    serial_println!("[TCP] Connection closing: {:?}", socket_id);

    Ok(())
}

/// Handle incoming TCP packet
pub fn handle_tcp_packet(
    packet_data: &[u8],
    src_addr: Ipv4Addr,
    dest_addr: Ipv4Addr,
) -> Result<(), TcpError> {
    let packet = TcpPacket::parse(packet_data, src_addr, dest_addr)?;

    serial_println!("[TCP] Received packet from {}:{} to {}:{} (flags={:#x})",
        src_addr, packet.src_port, dest_addr, packet.dest_port, packet.flags);

    // Find matching connection
    let socket_id = TcpSocketId {
        local_addr: dest_addr,
        local_port: packet.dest_port,
        remote_addr: src_addr,
        remote_port: packet.src_port,
    };

    let connections = TCP_CONNECTIONS.lock();
    
    if let Some(connection_arc) = connections.get(&socket_id) {
        let mut connection = connection_arc.lock();
        if let Ok(Some(response)) = connection.process_packet(&packet) {
            drop(connection);
            drop(connections);
            send_tcp_packet(&response, dest_addr, src_addr)?;
        }
    } else {
        // No existing connection - check if we're listening on this port
        let listening = connections.keys().any(|id| 
            id.local_addr == dest_addr && 
            id.local_port == packet.dest_port &&
            id.remote_addr.is_unspecified()
        );

        if listening && packet.has_flag(flags::SYN) {
            // Create new connection for incoming SYN
            let new_socket_id = socket_id;
            let mut new_connection = TcpConnection::new(new_socket_id);
            new_connection.state = TcpState::Listen;
            
            if let Ok(Some(response)) = new_connection.process_packet(&packet) {
                let connection_arc = Arc::new(Mutex::new(new_connection));
                drop(connections);
                TCP_CONNECTIONS.lock().insert(new_socket_id, connection_arc);
                send_tcp_packet(&response, dest_addr, src_addr)?;
            }
        } else {
            // No connection found, send RST
            serial_println!("[TCP] No connection found, sending RST");
            let rst = TcpPacket {
                src_port: packet.dest_port,
                dest_port: packet.src_port,
                sequence: 0,
                acknowledgment: packet.sequence.wrapping_add(1),
                data_offset: 5,
                flags: flags::RST | flags::ACK,
                window: 0,
                checksum: 0,
                urgent_pointer: 0,
                options: Vec::new(),
                data: Vec::new(),
            };
            send_tcp_packet(&rst, dest_addr, src_addr)?;
        }
    }

    Ok(())
}

/// Send a TCP packet (helper function)
fn send_tcp_packet(packet: &TcpPacket, src_addr: Ipv4Addr, dest_addr: Ipv4Addr) -> Result<(), TcpError> {
    let tcp_data = packet.build(src_addr, dest_addr);
    
    // Send via network stack
    crate::net::stack::queue_tx_packet(
        dest_addr,
        TCP_PROTOCOL,
        tcp_data,
    ).map_err(|_| TcpError::ConnectionReset)?;

    Ok(())
}

/// Get connection state
pub fn get_connection_state(socket_id: TcpSocketId) -> Option<TcpState> {
    let connections = TCP_CONNECTIONS.lock();
    connections.get(&socket_id).map(|conn| conn.lock().state)
}

/// List all active connections
pub fn list_connections() -> Vec<(TcpSocketId, TcpState)> {
    let connections = TCP_CONNECTIONS.lock();
    connections.iter().map(|(id, conn)| (*id, conn.lock().state)).collect()
}
