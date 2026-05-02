//! ICMP (Internet Control Message Protocol) Implementation
//! RFC 792 - https://www.rfc-editor.org/rfc/rfc792
//!
//! This module implements ICMP echo request/reply (ping) functionality.

use alloc::vec::Vec;
use core::fmt;

/// ICMP message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IcmpType {
    /// Echo Reply (Type 0)
    EchoReply = 0,
    /// Destination Unreachable (Type 3)
    DestinationUnreachable = 3,
    /// Source Quench (Type 4)
    SourceQuench = 4,
    /// Redirect (Type 5)
    Redirect = 5,
    /// Echo Request (Type 8)
    EchoRequest = 8,
    /// Time Exceeded (Type 11)
    TimeExceeded = 11,
    /// Parameter Problem (Type 12)
    ParameterProblem = 12,
    /// Unknown type
    Unknown(u8),
}

impl From<u8> for IcmpType {
    fn from(value: u8) -> Self {
        match value {
            0 => IcmpType::EchoReply,
            3 => IcmpType::DestinationUnreachable,
            4 => IcmpType::SourceQuench,
            5 => IcmpType::Redirect,
            8 => IcmpType::EchoRequest,
            11 => IcmpType::TimeExceeded,
            12 => IcmpType::ParameterProblem,
            other => IcmpType::Unknown(other),
        }
    }
}

impl From<IcmpType> for u8 {
    fn from(icmp_type: IcmpType) -> Self {
        match icmp_type {
            IcmpType::EchoReply => 0,
            IcmpType::DestinationUnreachable => 3,
            IcmpType::SourceQuench => 4,
            IcmpType::Redirect => 5,
            IcmpType::EchoRequest => 8,
            IcmpType::TimeExceeded => 11,
            IcmpType::ParameterProblem => 12,
            IcmpType::Unknown(val) => val,
        }
    }
}

impl fmt::Display for IcmpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IcmpType::EchoReply => write!(f, "Echo Reply"),
            IcmpType::DestinationUnreachable => write!(f, "Destination Unreachable"),
            IcmpType::SourceQuench => write!(f, "Source Quench"),
            IcmpType::Redirect => write!(f, "Redirect"),
            IcmpType::EchoRequest => write!(f, "Echo Request"),
            IcmpType::TimeExceeded => write!(f, "Time Exceeded"),
            IcmpType::ParameterProblem => write!(f, "Parameter Problem"),
            IcmpType::Unknown(val) => write!(f, "Unknown({})", val),
        }
    }
}

/// ICMP packet structure
///
/// Format:
/// ```text
/// [Type (1)][Code (1)][Checksum (2)]
/// [Identifier (2)][Sequence (2)][Data (variable)]
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IcmpPacket {
    /// ICMP message type
    pub icmp_type: IcmpType,
    /// ICMP code (subtype)
    pub code: u8,
    /// Checksum (calculated over entire ICMP packet)
    pub checksum: u16,
    /// Identifier (for echo request/reply)
    pub identifier: u16,
    /// Sequence number (for echo request/reply)
    pub sequence: u16,
    /// Payload data
    pub data: Vec<u8>,
}

/// ICMP parsing errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcmpError {
    /// Packet too short (minimum 8 bytes)
    PacketTooShort,
    /// Invalid checksum
    InvalidChecksum,
}

impl fmt::Display for IcmpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IcmpError::PacketTooShort => write!(f, "ICMP packet too short"),
            IcmpError::InvalidChecksum => write!(f, "Invalid ICMP checksum"),
        }
    }
}

impl IcmpPacket {
    /// Minimum ICMP packet size (header only)
    pub const MIN_SIZE: usize = 8;

    /// Parse an ICMP packet from raw bytes
    ///
    /// # Arguments
    /// * `data` - Raw ICMP packet bytes (minimum 8 bytes)
    ///
    /// # Returns
    /// * `Ok(IcmpPacket)` - Successfully parsed packet
    /// * `Err(IcmpError)` - Parse error
    pub fn from_bytes(data: &[u8]) -> Result<Self, IcmpError> {
        if data.len() < Self::MIN_SIZE {
            return Err(IcmpError::PacketTooShort);
        }

        let icmp_type = IcmpType::from(data[0]);
        let code = data[1];
        let checksum = u16::from_be_bytes([data[2], data[3]]);
        let identifier = u16::from_be_bytes([data[4], data[5]]);
        let sequence = u16::from_be_bytes([data[6], data[7]]);
        let payload = data[8..].to_vec();

        let packet = IcmpPacket {
            icmp_type,
            code,
            checksum,
            identifier,
            sequence,
            data: payload,
        };

        // Verify checksum
        if !packet.verify_checksum() {
            return Err(IcmpError::InvalidChecksum);
        }

        Ok(packet)
    }

    /// Convert ICMP packet to raw bytes
    ///
    /// Automatically calculates and sets the checksum.
    ///
    /// # Returns
    /// Raw ICMP packet bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::MIN_SIZE + self.data.len());

        // Header fields
        bytes.push(self.icmp_type.into());
        bytes.push(self.code);
        
        // Placeholder for checksum (will be calculated)
        bytes.extend_from_slice(&[0, 0]);
        
        // Identifier and sequence
        bytes.extend_from_slice(&self.identifier.to_be_bytes());
        bytes.extend_from_slice(&self.sequence.to_be_bytes());
        
        // Data
        bytes.extend_from_slice(&self.data);

        // Calculate and insert checksum
        let checksum = Self::calculate_checksum(&bytes);
        bytes[2..4].copy_from_slice(&checksum.to_be_bytes());

        bytes
    }

    /// Create a new ICMP echo request packet
    ///
    /// # Arguments
    /// * `identifier` - Request identifier (typically process ID)
    /// * `sequence` - Sequence number (increments with each ping)
    /// * `data` - Optional payload data
    ///
    /// # Returns
    /// ICMP echo request packet
    pub fn new_echo_request(identifier: u16, sequence: u16, data: Vec<u8>) -> Self {
        IcmpPacket {
            icmp_type: IcmpType::EchoRequest,
            code: 0,
            checksum: 0, // Will be calculated in to_bytes()
            identifier,
            sequence,
            data,
        }
    }

    /// Create an ICMP echo reply from an echo request
    ///
    /// This swaps the type from EchoRequest (8) to EchoReply (0)
    /// and preserves the identifier, sequence, and data.
    ///
    /// # Arguments
    /// * `request` - The original echo request packet
    ///
    /// # Returns
    /// ICMP echo reply packet
    pub fn create_echo_reply(request: &IcmpPacket) -> Self {
        IcmpPacket {
            icmp_type: IcmpType::EchoReply,
            code: request.code,
            checksum: 0, // Will be recalculated in to_bytes()
            identifier: request.identifier,
            sequence: request.sequence,
            data: request.data.clone(),
        }
    }

    /// Calculate ICMP checksum (RFC 1071)
    ///
    /// The checksum is the 16-bit one's complement of the one's complement
    /// sum of the ICMP message starting with the ICMP Type.
    ///
    /// # Arguments
    /// * `data` - ICMP packet bytes (with checksum field set to 0)
    ///
    /// # Returns
    /// Calculated checksum
    fn calculate_checksum(data: &[u8]) -> u16 {
        let mut sum: u32 = 0;

        // Sum all 16-bit words
        for chunk in data.chunks(2) {
            let word = if chunk.len() == 2 {
                u16::from_be_bytes([chunk[0], chunk[1]])
            } else {
                // Odd length - pad with zero
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

    /// Verify the packet's checksum
    ///
    /// # Returns
    /// `true` if checksum is valid, `false` otherwise
    pub fn verify_checksum(&self) -> bool {
        // Create packet with checksum set to 0
        let mut bytes = Vec::with_capacity(Self::MIN_SIZE + self.data.len());
        bytes.push(self.icmp_type.into());
        bytes.push(self.code);
        bytes.extend_from_slice(&[0, 0]); // Zero checksum
        bytes.extend_from_slice(&self.identifier.to_be_bytes());
        bytes.extend_from_slice(&self.sequence.to_be_bytes());
        bytes.extend_from_slice(&self.data);

        let calculated = Self::calculate_checksum(&bytes);
        calculated == self.checksum
    }

    /// Check if this is an echo request
    pub fn is_echo_request(&self) -> bool {
        self.icmp_type == IcmpType::EchoRequest
    }

    /// Check if this is an echo reply
    pub fn is_echo_reply(&self) -> bool {
        self.icmp_type == IcmpType::EchoReply
    }
}

impl fmt::Display for IcmpPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ICMP {} (code={}, id={}, seq={}, {} bytes data)",
            self.icmp_type,
            self.code,
            self.identifier,
            self.sequence,
            self.data.len()
        )
    }
}

/// Ping statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PingStats {
    /// Number of packets sent
    pub sent: u16,
    /// Number of packets received
    pub received: u16,
    /// Average round-trip time in milliseconds
    pub avg_rtt_ms: u32,
    /// Minimum RTT in milliseconds
    pub min_rtt_ms: u32,
    /// Maximum RTT in milliseconds
    pub max_rtt_ms: u32,
}

impl PingStats {
    /// Create new ping statistics
    pub fn new() -> Self {
        Self {
            sent: 0,
            received: 0,
            avg_rtt_ms: 0,
            min_rtt_ms: u32::MAX,
            max_rtt_ms: 0,
        }
    }

    /// Calculate packet loss percentage
    pub fn packet_loss(&self) -> f32 {
        if self.sent == 0 {
            return 0.0;
        }
        let lost = self.sent.saturating_sub(self.received);
        (lost as f32 / self.sent as f32) * 100.0
    }

    /// Update statistics with a new RTT measurement
    pub fn add_measurement(&mut self, rtt_ms: u32) {
        self.received += 1;
        
        // Update min/max
        if rtt_ms < self.min_rtt_ms {
            self.min_rtt_ms = rtt_ms;
        }
        if rtt_ms > self.max_rtt_ms {
            self.max_rtt_ms = rtt_ms;
        }

        // Update average (running average)
        let total = (self.avg_rtt_ms as u64) * ((self.received - 1) as u64) + (rtt_ms as u64);
        self.avg_rtt_ms = (total / (self.received as u64)) as u32;
    }
}

impl fmt::Display for PingStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} packets transmitted, {} received, {:.1}% packet loss\n\
             rtt min/avg/max = {}/{}/{} ms",
            self.sent,
            self.received,
            self.packet_loss(),
            self.min_rtt_ms,
            self.avg_rtt_ms,
            self.max_rtt_ms
        )
    }
}

/// Ping error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingError {
    /// Network device not available
    NoNetworkDevice,
    /// Failed to send packet
    SendFailed,
    /// Timeout waiting for reply
    Timeout,
    /// Invalid response received
    InvalidResponse,
}

impl fmt::Display for PingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PingError::NoNetworkDevice => write!(f, "No network device available"),
            PingError::SendFailed => write!(f, "Failed to send packet"),
            PingError::Timeout => write!(f, "Request timeout"),
            PingError::InvalidResponse => write!(f, "Invalid response received"),
        }
    }
}
