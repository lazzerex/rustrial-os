//Ethernet Frame Layer (OSI Layer 2)
//
//Handles Ethernet frame parsing, building, and dispatching.
//Frame structure: [Dest MAC (6)][Src MAC (6)][EtherType (2)][Payload (46-1500)][CRC (4)]

extern crate alloc;
use alloc::vec::Vec;

/// EtherType constants
pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const ETHERTYPE_ARP: u16 = 0x0806;
pub const ETHERTYPE_IPV6: u16 = 0x86DD;

/// Broadcast MAC address (FF:FF:FF:FF:FF:FF)
pub const BROADCAST_MAC: [u8; 6] = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

/// Minimum Ethernet payload size (padding required if smaller)
pub const MIN_PAYLOAD_SIZE: usize = 46;

/// Maximum Ethernet payload size (MTU)
pub const MAX_PAYLOAD_SIZE: usize = 1500;

/// Ethernet frame header size (excluding CRC)
pub const HEADER_SIZE: usize = 14;

/// CRC size
pub const CRC_SIZE: usize = 4;

/// Errors that can occur during Ethernet frame operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EthernetError {
    /// Frame is too short to be valid
    FrameTooShort,
    /// Payload exceeds MTU
    PayloadTooLarge,
    /// Invalid CRC checksum
    InvalidCrc,
}

/// Represents an Ethernet frame
#[derive(Debug, Clone, PartialEq)]
pub struct EthernetFrame {
    /// Destination MAC address (6 bytes)
    pub dest_mac: [u8; 6],
    /// Source MAC address (6 bytes)
    pub src_mac: [u8; 6],
    /// EtherType field (2 bytes) - indicates protocol of payload
    pub ethertype: u16,
    /// Payload data (46-1500 bytes)
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    /// # Arguments
    /// * `dest` - Destination MAC address
    /// * `src` - Source MAC address
    /// * `ethertype` - Protocol type (e.g., 0x0800 for IPv4, 0x0806 for ARP)
    /// * `payload` - Frame payload data
    pub fn new(dest: [u8; 6], src: [u8; 6], ethertype: u16, payload: Vec<u8>) -> Result<Self, EthernetError> {
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err(EthernetError::PayloadTooLarge);
        }

        Ok(Self {
            dest_mac: dest,
            src_mac: src,
            ethertype,
            payload,
        })
    }

    /// Parse an Ethernet frame from raw bytes
    /// 
    /// # Arguments
    /// * `data` - Raw frame data (including Ethernet header and payload, optionally CRC)
    /// 
    /// # Returns
    /// Parsed EthernetFrame or error
    pub fn from_bytes(data: &[u8]) -> Result<Self, EthernetError> {
        // Minimum valid frame: 14 bytes header + 46 bytes payload = 60 bytes
        if data.len() < HEADER_SIZE {
            return Err(EthernetError::FrameTooShort);
        }

        // Extract destination MAC (bytes 0-5)
        let mut dest_mac = [0u8; 6];
        dest_mac.copy_from_slice(&data[0..6]);

        // Extract source MAC (bytes 6-11)
        let mut src_mac = [0u8; 6];
        src_mac.copy_from_slice(&data[6..12]);

        // Extract EtherType (bytes 12-13, big-endian)
        let ethertype = u16::from_be_bytes([data[12], data[13]]);

        // Extract payload (everything after header, excluding CRC if present)
        let payload_end = if data.len() >= HEADER_SIZE + MIN_PAYLOAD_SIZE + CRC_SIZE {
            // Frame might include CRC at the end, strip it
            data.len() - CRC_SIZE
        } else {
            data.len()
        };
        
        let payload = data[HEADER_SIZE..payload_end].to_vec();

        Ok(Self {
            dest_mac,
            src_mac,
            ethertype,
            payload,
        })
    }

    /// Convert the Ethernet frame to bytes for transmission
    /// 
    /// Frame format: [Dest MAC][Src MAC][EtherType][Payload][CRC]
    /// Adds padding if payload is less than 46 bytes.
    /// Appends CRC32 checksum.
    /// 
    /// # Returns
    /// Complete frame ready for transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut frame = Vec::with_capacity(HEADER_SIZE + MAX_PAYLOAD_SIZE + CRC_SIZE);

        // Add destination MAC
        frame.extend_from_slice(&self.dest_mac);

        // Add source MAC
        frame.extend_from_slice(&self.src_mac);

        // Add EtherType (big-endian)
        frame.extend_from_slice(&self.ethertype.to_be_bytes());

        // Add payload
        frame.extend_from_slice(&self.payload);

        // Pad payload if necessary (minimum 46 bytes)
        while frame.len() < HEADER_SIZE + MIN_PAYLOAD_SIZE {
            frame.push(0x00);
        }

        // Calculate and append CRC32
        let crc = calculate_crc32(&frame);
        frame.extend_from_slice(&crc.to_le_bytes());

        frame
    }

    /// Check if the frame is a broadcast frame
    pub fn is_broadcast(&self) -> bool {
        self.dest_mac == BROADCAST_MAC
    }

    /// Check if the frame is a multicast frame
    pub fn is_multicast(&self) -> bool {
        (self.dest_mac[0] & 0x01) != 0 && !self.is_broadcast()
    }

    /// Check if the frame is a unicast frame
    pub fn is_unicast(&self) -> bool {
        !self.is_broadcast() && !self.is_multicast()
    }

    /// Get frame size (including headers and CRC)
    pub fn total_size(&self) -> usize {
        HEADER_SIZE + self.payload.len().max(MIN_PAYLOAD_SIZE) + CRC_SIZE
    }

    /// Get the payload as a slice
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// Calculate CRC32 checksum for Ethernet frame
/// 
/// Uses the IEEE 802.3 CRC32 polynomial: 0x04C11DB7
/// 
/// # Arguments
/// * `data` - Data to calculate checksum for
/// 
/// # Returns
/// CRC32 checksum value
fn calculate_crc32(data: &[u8]) -> u32 {
    const CRC32_POLYNOMIAL: u32 = 0xEDB88320; // Reversed polynomial
    
    let mut crc: u32 = 0xFFFFFFFF;
    
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ CRC32_POLYNOMIAL;
            } else {
                crc >>= 1;
            }
        }
    }
    
    !crc
}

/// Verify CRC32 checksum of a received frame
/// 
/// # Arguments
/// * `data` - Complete frame including CRC
/// 
/// # Returns
/// true if CRC is valid, false otherwise
pub fn verify_crc32(data: &[u8]) -> bool {
    if data.len() < CRC_SIZE {
        return false;
    }
    
    let data_len = data.len() - CRC_SIZE;
    let frame_data = &data[..data_len];
    let received_crc = u32::from_le_bytes([
        data[data_len],
        data[data_len + 1],
        data[data_len + 2],
        data[data_len + 3],
    ]);
    
    let calculated_crc = calculate_crc32(frame_data);
    received_crc == calculated_crc
}

/// Protocol handler trait for frame dispatching
pub trait ProtocolHandler: Send + Sync {
    /// Handle a received frame
    fn handle_frame(&self, frame: &EthernetFrame);
}

/// Frame dispatcher - routes frames to appropriate protocol handlers
pub struct FrameDispatcher {
    /// Handler for IPv4 packets (EtherType 0x0800)
    ipv4_handler: Option<&'static dyn ProtocolHandler>,
    /// Handler for ARP packets (EtherType 0x0806)
    arp_handler: Option<&'static dyn ProtocolHandler>,
}

impl FrameDispatcher {
    /// Create a new frame dispatcher
    pub const fn new() -> Self {
        Self {
            ipv4_handler: None,
            arp_handler: None,
        }
    }

    /// Register a handler for IPv4 packets
    pub fn register_ipv4_handler(&mut self, handler: &'static dyn ProtocolHandler) {
        self.ipv4_handler = Some(handler);
    }

    /// Register a handler for ARP packets
    pub fn register_arp_handler(&mut self, handler: &'static dyn ProtocolHandler) {
        self.arp_handler = Some(handler);
    }

    /// Dispatch a received frame to the appropriate handler
    /// 
    /// # Arguments
    /// * `frame` - The Ethernet frame to dispatch
    /// * `our_mac` - Our MAC address (for filtering)
    /// 
    /// # Returns
    /// true if the frame was handled, false if it was filtered or no handler exists
    pub fn dispatch(&mut self, frame: &EthernetFrame, our_mac: [u8; 6]) -> bool {
        // Frame filtering: only process unicast to us, broadcast, or multicast
        if frame.is_unicast() && frame.dest_mac != our_mac {
            return false; // Not for us
        }

        // Dispatch based on EtherType
        match frame.ethertype {
            ETHERTYPE_IPV4 => {
                if let Some(handler) = &mut self.ipv4_handler {
                    handler.handle_frame(frame);
                    true
                } else {
                    false
                }
            }
            ETHERTYPE_ARP => {
                if let Some(handler) = &mut self.arp_handler {
                    handler.handle_frame(frame);
                    true
                } else {
                    false
                }
            }
            _ => {
                // Unknown protocol, drop frame
                false
            }
        }
    }
}
