//! IPv4 Protocol Implementation
//! 
//! Implements IPv4 packet parsing, building, checksum calculation, and routing.
//! 
//! # References
//! - RFC 791: Internet Protocol (IPv4)
//! - RFC 1071: Computing the Internet Checksum

use alloc::vec::Vec;
use core::fmt;

/// IPv4 Address type (re-export for convenience)
pub use core::net::Ipv4Addr;

/// IPv4 Protocol Numbers (IANA assigned)
pub mod protocol {
    pub const ICMP: u8 = 1;
    pub const TCP: u8 = 6;
    pub const UDP: u8 = 17;
}

/// IPv4 Header Flags
pub mod flags {
    pub const DONT_FRAGMENT: u16 = 0x4000;
    pub const MORE_FRAGMENTS: u16 = 0x2000;
}

/// Default TTL (Time To Live) value
pub const DEFAULT_TTL: u8 = 64;

/// Minimum IPv4 header size (without options)
pub const MIN_HEADER_SIZE: usize = 20;

/// Maximum IPv4 packet size
pub const MAX_PACKET_SIZE: usize = 65535;

/// IPv4 Header Structure
/// 
/// ```text
/// 0                   1                   2                   3
/// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |Version|  IHL  |    DSCP   |ECN|         Total Length          |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |         Identification        |Flags|      Fragment Offset    |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |  Time to Live |    Protocol   |         Header Checksum       |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                       Source Address                          |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                    Destination Address                        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                    Options (if IHL > 5)                       |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv4Header {
    /// IP version (always 4)
    pub version: u8,
    /// Internet Header Length in 32-bit words (5-15)
    pub ihl: u8,
    /// Differentiated Services Code Point
    pub dscp: u8,
    /// Explicit Congestion Notification
    pub ecn: u8,
    /// Total packet length (header + data) in bytes
    pub total_length: u16,
    /// Identification field for fragmentation
    pub identification: u16,
    /// Flags (bit 0: reserved, bit 1: DF, bit 2: MF)
    pub flags: u16,
    /// Fragment offset in 8-byte blocks
    pub fragment_offset: u16,
    /// Time To Live (hops)
    pub ttl: u8,
    /// Protocol number (ICMP=1, TCP=6, UDP=17)
    pub protocol: u8,
    /// Header checksum
    pub checksum: u16,
    /// Source IP address
    pub src_ip: Ipv4Addr,
    /// Destination IP address
    pub dest_ip: Ipv4Addr,
    /// Options (if IHL > 5)
    pub options: Vec<u8>,
}

impl Ipv4Header {
    /// Create a new IPv4 header with common defaults
    /// 
    /// # Arguments
    /// * `src_ip` - Source IP address
    /// * `dest_ip` - Destination IP address
    /// * `protocol` - Protocol number (use `protocol::*` constants)
    /// * `payload_len` - Length of the payload data
    /// 
    /// # Returns
    /// A new IPv4 header with:
    /// - Version: 4
    /// - IHL: 5 (20 bytes, no options)
    /// - DSCP/ECN: 0
    /// - TTL: 64 (DEFAULT_TTL)
    /// - Flags: 0 (no fragmentation)
    /// - Identification: 0 (caller should set if fragmentation used)
    /// - Checksum: 0 (must be calculated with `calculate_checksum()`)
    pub fn new(src_ip: Ipv4Addr, dest_ip: Ipv4Addr, protocol: u8, payload_len: u16) -> Self {
        Self {
            version: 4,
            ihl: 5, // 5 * 4 = 20 bytes (minimum header size)
            dscp: 0,
            ecn: 0,
            total_length: MIN_HEADER_SIZE as u16 + payload_len,
            identification: 0,
            flags: 0,
            fragment_offset: 0,
            ttl: DEFAULT_TTL,
            protocol,
            checksum: 0, // Will be calculated
            src_ip,
            dest_ip,
            options: Vec::new(),
        }
    }

    /// Parse an IPv4 header from raw bytes
    /// 
    /// # Arguments
    /// * `data` - Raw packet bytes (must be at least 20 bytes)
    /// 
    /// # Returns
    /// - `Ok((Ipv4Header, payload_offset))` - Parsed header and offset to payload data
    /// - `Err(Ipv4Error)` - Parse error
    /// 
    /// # Validation
    /// - Checks minimum length
    /// - Validates version field (must be 4)
    /// - Validates IHL (must be >= 5)
    /// - Validates total_length (must be >= header length)
    /// - Verifies header checksum
    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize), Ipv4Error> {
        if data.len() < MIN_HEADER_SIZE {
            return Err(Ipv4Error::PacketTooShort);
        }

        // Parse first byte: version (4 bits) + IHL (4 bits)
        let version_ihl = data[0];
        let version = version_ihl >> 4;
        let ihl = version_ihl & 0x0F;

        if version != 4 {
            return Err(Ipv4Error::InvalidVersion(version));
        }

        if ihl < 5 {
            return Err(Ipv4Error::InvalidIhl(ihl));
        }

        let header_len = (ihl as usize) * 4;
        if data.len() < header_len {
            return Err(Ipv4Error::PacketTooShort);
        }

        // Parse second byte: DSCP (6 bits) + ECN (2 bits)
        let dscp_ecn = data[1];
        let dscp = dscp_ecn >> 2;
        let ecn = dscp_ecn & 0x03;

        // Parse total length (big-endian)
        let total_length = u16::from_be_bytes([data[2], data[3]]);
        
        if (total_length as usize) < header_len {
            return Err(Ipv4Error::InvalidLength);
        }

        // Parse identification
        let identification = u16::from_be_bytes([data[4], data[5]]);

        // Parse flags (3 bits) + fragment offset (13 bits)
        let flags_frag = u16::from_be_bytes([data[6], data[7]]);
        let flags = (flags_frag >> 13) & 0x07;
        let fragment_offset = flags_frag & 0x1FFF;

        // Parse TTL, protocol, checksum
        let ttl = data[8];
        let protocol = data[9];
        let checksum = u16::from_be_bytes([data[10], data[11]]);

        // Parse IP addresses
        let src_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
        let dest_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);

        // Parse options (if any)
        let options = if ihl > 5 {
            data[MIN_HEADER_SIZE..header_len].to_vec()
        } else {
            Vec::new()
        };

        let header = Self {
            version,
            ihl,
            dscp,
            ecn,
            total_length,
            identification,
            flags,
            fragment_offset,
            ttl,
            protocol,
            checksum,
            src_ip,
            dest_ip,
            options,
        };

        // Verify checksum
        if !header.verify_checksum(data) {
            return Err(Ipv4Error::ChecksumMismatch);
        }

        Ok((header, header_len))
    }

    /// Serialize the IPv4 header to bytes
    /// 
    /// # Returns
    /// Raw header bytes (20+ bytes depending on options)
    /// 
    /// # Note
    /// The checksum field will be calculated and inserted automatically.
    pub fn to_bytes(&self) -> Vec<u8> {
        let header_len = (self.ihl as usize) * 4;
        let mut bytes = Vec::with_capacity(header_len);

        // Byte 0: Version (4 bits) + IHL (4 bits)
        bytes.push((self.version << 4) | (self.ihl & 0x0F));

        // Byte 1: DSCP (6 bits) + ECN (2 bits)
        bytes.push((self.dscp << 2) | (self.ecn & 0x03));

        // Bytes 2-3: Total Length
        bytes.extend_from_slice(&self.total_length.to_be_bytes());

        // Bytes 4-5: Identification
        bytes.extend_from_slice(&self.identification.to_be_bytes());

        // Bytes 6-7: Flags (3 bits) + Fragment Offset (13 bits)
        let flags_frag = ((self.flags & 0x07) << 13) | (self.fragment_offset & 0x1FFF);
        bytes.extend_from_slice(&flags_frag.to_be_bytes());

        // Byte 8: TTL
        bytes.push(self.ttl);

        // Byte 9: Protocol
        bytes.push(self.protocol);

        // Bytes 10-11: Checksum (placeholder, will be calculated)
        bytes.push(0);
        bytes.push(0);

        // Bytes 12-15: Source IP
        bytes.extend_from_slice(&self.src_ip.octets());

        // Bytes 16-19: Destination IP
        bytes.extend_from_slice(&self.dest_ip.octets());

        // Options (if any)
        if !self.options.is_empty() {
            bytes.extend_from_slice(&self.options);
        }

        // Calculate and insert checksum
        let checksum = Self::calculate_checksum(&bytes);
        bytes[10] = (checksum >> 8) as u8;
        bytes[11] = checksum as u8;

        bytes
    }

    /// Calculate the IPv4 header checksum (RFC 1071)
    /// 
    /// The checksum is the 16-bit one's complement of the one's complement sum
    /// of all 16-bit words in the header (with checksum field set to 0).
    /// 
    /// # Arguments
    /// * `header` - Header bytes (checksum field must be 0)
    /// 
    /// # Returns
    /// 16-bit checksum value
    pub fn calculate_checksum(header: &[u8]) -> u16 {
        let mut sum: u32 = 0;

        // Sum all 16-bit words
        for chunk in header.chunks(2) {
            let word = if chunk.len() == 2 {
                u16::from_be_bytes([chunk[0], chunk[1]])
            } else {
                // Odd length: pad with 0
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

    /// Verify the IPv4 header checksum
    /// 
    /// # Arguments
    /// * `data` - Raw packet bytes
    /// 
    /// # Returns
    /// `true` if checksum is valid, `false` otherwise
    pub fn verify_checksum(&self, data: &[u8]) -> bool {
        let header_len = (self.ihl as usize) * 4;
        if data.len() < header_len {
            return false;
        }

        // Create a copy of the header with checksum field zeroed
        let mut header_copy = data[..header_len].to_vec();
        header_copy[10] = 0;
        header_copy[11] = 0;

        let calculated = Self::calculate_checksum(&header_copy);
        calculated == self.checksum
    }

    /// Get the payload data from a packet
    /// 
    /// # Arguments
    /// * `data` - Raw packet bytes
    /// 
    /// # Returns
    /// Payload slice (or empty if invalid)
    pub fn payload<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        let header_len = (self.ihl as usize) * 4;
        let total_len = self.total_length as usize;
        
        if data.len() < total_len {
            &data[header_len..]
        } else {
            &data[header_len..total_len]
        }
    }

    /// Check if this packet is fragmented
    pub fn is_fragmented(&self) -> bool {
        (self.flags & flags::MORE_FRAGMENTS) != 0 || self.fragment_offset != 0
    }

    /// Get the header length in bytes
    pub fn header_length(&self) -> usize {
        (self.ihl as usize) * 4
    }
}

/// IPv4 Packet (Header + Payload)
#[derive(Debug, Clone)]
pub struct Ipv4Packet {
    pub header: Ipv4Header,
    pub payload: Vec<u8>,
}

impl Ipv4Packet {
    /// Create a new IPv4 packet
    pub fn new(src_ip: Ipv4Addr, dest_ip: Ipv4Addr, protocol: u8, payload: Vec<u8>) -> Self {
        let header = Ipv4Header::new(src_ip, dest_ip, protocol, payload.len() as u16);
        Self { header, payload }
    }

    /// Parse an IPv4 packet from raw bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, Ipv4Error> {
        let (header, offset) = Ipv4Header::from_bytes(data)?;
        let payload = header.payload(data).to_vec();
        Ok(Self { header, payload })
    }

    /// Serialize the packet to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header.to_bytes();
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}

/// IPv4 Routing Table
/// 
/// Determines if a destination IP is local (same subnet) or requires a gateway.
#[derive(Debug, Clone)]
pub struct RoutingTable {
    /// Our local IP address
    pub local_ip: Ipv4Addr,
    /// Network mask (e.g., 255.255.255.0)
    pub netmask: Ipv4Addr,
    /// Default gateway (if any)
    pub gateway: Option<Ipv4Addr>,
}

impl RoutingTable {
    /// Create a new routing table
    pub fn new(local_ip: Ipv4Addr, netmask: Ipv4Addr, gateway: Option<Ipv4Addr>) -> Self {
        Self {
            local_ip,
            netmask,
            gateway,
        }
    }

    /// Determine the next hop for a destination IP
    /// 
    /// # Returns
    /// - `Some(ip)` - Next hop IP (either destination if local, or gateway)
    /// - `None` - No route available
    pub fn next_hop(&self, dest_ip: Ipv4Addr) -> Option<Ipv4Addr> {
        if self.is_local(dest_ip) {
            // Destination is on local subnet, send directly
            Some(dest_ip)
        } else if let Some(gw) = self.gateway {
            // Send to gateway
            Some(gw)
        } else {
            // No route available
            None
        }
    }

    /// Check if a destination IP is on the local subnet
    pub fn is_local(&self, dest_ip: Ipv4Addr) -> bool {
        let local_octets = self.local_ip.octets();
        let dest_octets = dest_ip.octets();
        let mask_octets = self.netmask.octets();

        for i in 0..4 {
            if (local_octets[i] & mask_octets[i]) != (dest_octets[i] & mask_octets[i]) {
                return false;
            }
        }
        true
    }

    /// Check if an IP is our local address
    pub fn is_our_ip(&self, ip: Ipv4Addr) -> bool {
        ip == self.local_ip
    }
}

/// Protocol dispatcher for routing packets to upper-layer handlers
pub struct ProtocolDispatcher {
    icmp_handler: Option<fn(&Ipv4Header, &[u8])>,
    tcp_handler: Option<fn(&Ipv4Header, &[u8])>,
    udp_handler: Option<fn(&Ipv4Header, &[u8])>,
}

impl ProtocolDispatcher {
    /// Create a new protocol dispatcher
    pub fn new() -> Self {
        Self {
            icmp_handler: None,
            tcp_handler: None,
            udp_handler: None,
        }
    }

    /// Register an ICMP handler
    pub fn register_icmp(&mut self, handler: fn(&Ipv4Header, &[u8])) {
        self.icmp_handler = Some(handler);
    }

    /// Register a TCP handler
    pub fn register_tcp(&mut self, handler: fn(&Ipv4Header, &[u8])) {
        self.tcp_handler = Some(handler);
    }

    /// Register a UDP handler
    pub fn register_udp(&mut self, handler: fn(&Ipv4Header, &[u8])) {
        self.udp_handler = Some(handler);
    }

    /// Dispatch a packet to the appropriate protocol handler
    /// 
    /// # Returns
    /// `true` if a handler was found and called, `false` otherwise
    pub fn dispatch(&self, header: &Ipv4Header, payload: &[u8]) -> bool {
        match header.protocol {
            protocol::ICMP => {
                if let Some(handler) = self.icmp_handler {
                    handler(header, payload);
                    true
                } else {
                    false
                }
            }
            protocol::TCP => {
                if let Some(handler) = self.tcp_handler {
                    handler(header, payload);
                    true
                } else {
                    false
                }
            }
            protocol::UDP => {
                if let Some(handler) = self.udp_handler {
                    handler(header, payload);
                    true
                } else {
                    false
                }
            }
            _ => false, // Unknown protocol
        }
    }
}

impl Default for ProtocolDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// IPv4 Error Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ipv4Error {
    /// Packet is too short to contain a valid header
    PacketTooShort,
    /// Invalid IP version (not 4)
    InvalidVersion(u8),
    /// Invalid IHL (Internet Header Length < 5)
    InvalidIhl(u8),
    /// Invalid total length field
    InvalidLength,
    /// Header checksum mismatch
    ChecksumMismatch,
    /// Fragmentation not supported
    FragmentationNotSupported,
}

impl fmt::Display for Ipv4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ipv4Error::PacketTooShort => write!(f, "Packet too short"),
            Ipv4Error::InvalidVersion(v) => write!(f, "Invalid IP version: {}", v),
            Ipv4Error::InvalidIhl(ihl) => write!(f, "Invalid IHL: {}", ihl),
            Ipv4Error::InvalidLength => write!(f, "Invalid total length"),
            Ipv4Error::ChecksumMismatch => write!(f, "Checksum mismatch"),
            Ipv4Error::FragmentationNotSupported => write!(f, "Fragmentation not supported"),
        }
    }
}

/// Global packet ID counter for generating unique identification values
static mut PACKET_ID_COUNTER: u16 = 0;

/// Get the next packet ID for fragmentation
pub fn next_packet_id() -> u16 {
    unsafe {
        let id = PACKET_ID_COUNTER;
        PACKET_ID_COUNTER = PACKET_ID_COUNTER.wrapping_add(1);
        id
    }
}
