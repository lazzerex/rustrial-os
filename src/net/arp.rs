//! ARP (Address Resolution Protocol) - RFC 826
//! 
//! Maps IPv4 addresses to MAC addresses on local networks.
//! Packet format: [HW Type (2)][Proto Type (2)][HW Len (1)][Proto Len (1)]
//!                [Operation (2)][Sender MAC (6)][Sender IP (4)]
//!                [Target MAC (6)][Target IP (4)]

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use core::net::Ipv4Addr;

/// ARP hardware type for Ethernet
pub const HW_TYPE_ETHERNET: u16 = 1;

/// ARP protocol type for IPv4
pub const PROTO_TYPE_IPV4: u16 = 0x0800;

/// ARP operation codes
pub const ARP_REQUEST: u16 = 1;
pub const ARP_REPLY: u16 = 2;

/// ARP packet size (fixed at 28 bytes)
pub const ARP_PACKET_SIZE: usize = 28;

/// Default ARP cache entry TTL (300 seconds = 5 minutes)
pub const ARP_CACHE_TTL_SECS: u64 = 300;

/// Errors that can occur during ARP operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpError {
    /// Packet is too short to be valid
    PacketTooShort,
    /// Invalid hardware type (not Ethernet)
    InvalidHardwareType,
    /// Invalid protocol type (not IPv4)
    InvalidProtocolType,
    /// Invalid hardware address length
    InvalidHardwareLength,
    /// Invalid protocol address length
    InvalidProtocolLength,
    /// Unknown operation code
    UnknownOperation,
    /// IP address not found in cache
    NotInCache,
}

/// ARP packet structure
#[derive(Debug, Clone, PartialEq)]
pub struct ArpPacket {
    /// Hardware type (1 = Ethernet)
    pub hw_type: u16,
    /// Protocol type (0x0800 = IPv4)
    pub proto_type: u16,
    /// Hardware address length (6 for MAC)
    pub hw_len: u8,
    /// Protocol address length (4 for IPv4)
    pub proto_len: u8,
    /// Operation (1 = request, 2 = reply)
    pub operation: u16,
    /// Sender MAC address
    pub sender_mac: [u8; 6],
    /// Sender IP address
    pub sender_ip: Ipv4Addr,
    /// Target MAC address
    pub target_mac: [u8; 6],
    /// Target IP address
    pub target_ip: Ipv4Addr,
}

impl ArpPacket {
    /// Create a new ARP request packet
    /// 
    /// # Arguments
    /// * `sender_mac` - Our MAC address
    /// * `sender_ip` - Our IP address
    /// * `target_ip` - The IP address we want to resolve
    pub fn new_request(sender_mac: [u8; 6], sender_ip: Ipv4Addr, target_ip: Ipv4Addr) -> Self {
        Self {
            hw_type: HW_TYPE_ETHERNET,
            proto_type: PROTO_TYPE_IPV4,
            hw_len: 6,
            proto_len: 4,
            operation: ARP_REQUEST,
            sender_mac,
            sender_ip,
            target_mac: [0; 6], // Unknown, set to zeros
            target_ip,
        }
    }

    /// Create a new ARP reply packet
    /// 
    /// # Arguments
    /// * `sender_mac` - Our MAC address
    /// * `sender_ip` - Our IP address
    /// * `target_mac` - The MAC address of the requester
    /// * `target_ip` - The IP address of the requester
    pub fn new_reply(
        sender_mac: [u8; 6],
        sender_ip: Ipv4Addr,
        target_mac: [u8; 6],
        target_ip: Ipv4Addr,
    ) -> Self {
        Self {
            hw_type: HW_TYPE_ETHERNET,
            proto_type: PROTO_TYPE_IPV4,
            hw_len: 6,
            proto_len: 4,
            operation: ARP_REPLY,
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        }
    }

    /// Parse an ARP packet from raw bytes
    /// 
    /// # Arguments
    /// * `data` - Raw ARP packet data (must be at least 28 bytes)
    pub fn from_bytes(data: &[u8]) -> Result<Self, ArpError> {
        if data.len() < ARP_PACKET_SIZE {
            return Err(ArpError::PacketTooShort);
        }

        // Parse hardware type (bytes 0-1, big-endian)
        let hw_type = u16::from_be_bytes([data[0], data[1]]);
        if hw_type != HW_TYPE_ETHERNET {
            return Err(ArpError::InvalidHardwareType);
        }

        // Parse protocol type (bytes 2-3, big-endian)
        let proto_type = u16::from_be_bytes([data[2], data[3]]);
        if proto_type != PROTO_TYPE_IPV4 {
            return Err(ArpError::InvalidProtocolType);
        }

        // Parse address lengths
        let hw_len = data[4];
        let proto_len = data[5];
        
        if hw_len != 6 {
            return Err(ArpError::InvalidHardwareLength);
        }
        if proto_len != 4 {
            return Err(ArpError::InvalidProtocolLength);
        }

        // Parse operation (bytes 6-7, big-endian)
        let operation = u16::from_be_bytes([data[6], data[7]]);
        if operation != ARP_REQUEST && operation != ARP_REPLY {
            return Err(ArpError::UnknownOperation);
        }

        // Parse sender MAC (bytes 8-13)
        let mut sender_mac = [0u8; 6];
        sender_mac.copy_from_slice(&data[8..14]);

        // Parse sender IP (bytes 14-17)
        let sender_ip = Ipv4Addr::new(data[14], data[15], data[16], data[17]);

        // Parse target MAC (bytes 18-23)
        let mut target_mac = [0u8; 6];
        target_mac.copy_from_slice(&data[18..24]);

        // Parse target IP (bytes 24-27)
        let target_ip = Ipv4Addr::new(data[24], data[25], data[26], data[27]);

        Ok(Self {
            hw_type,
            proto_type,
            hw_len,
            proto_len,
            operation,
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        })
    }

    /// Convert the ARP packet to bytes for transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(ARP_PACKET_SIZE);

        // Hardware type (2 bytes, big-endian)
        bytes.extend_from_slice(&self.hw_type.to_be_bytes());

        // Protocol type (2 bytes, big-endian)
        bytes.extend_from_slice(&self.proto_type.to_be_bytes());

        // Hardware address length (1 byte)
        bytes.push(self.hw_len);

        // Protocol address length (1 byte)
        bytes.push(self.proto_len);

        // Operation (2 bytes, big-endian)
        bytes.extend_from_slice(&self.operation.to_be_bytes());

        // Sender MAC address (6 bytes)
        bytes.extend_from_slice(&self.sender_mac);

        // Sender IP address (4 bytes)
        bytes.extend_from_slice(&self.sender_ip.octets());

        // Target MAC address (6 bytes)
        bytes.extend_from_slice(&self.target_mac);

        // Target IP address (4 bytes)
        bytes.extend_from_slice(&self.target_ip.octets());

        bytes
    }

    /// Check if this is an ARP request
    pub fn is_request(&self) -> bool {
        self.operation == ARP_REQUEST
    }

    /// Check if this is an ARP reply
    pub fn is_reply(&self) -> bool {
        self.operation == ARP_REPLY
    }
}

/// ARP cache entry with expiration timestamp
#[derive(Debug, Clone, Copy)]
pub struct ArpEntry {
    /// MAC address associated with the IP
    pub mac: [u8; 6],
    /// Timestamp when this entry expires (in seconds since boot)
    pub expires_at: u64,
}

impl ArpEntry {
    /// Create a new ARP cache entry
    /// 
    /// # Arguments
    /// * `mac` - MAC address to cache
    /// * `current_time` - Current time in seconds
    /// * `ttl` - Time-to-live in seconds
    pub fn new(mac: [u8; 6], current_time: u64, ttl: u64) -> Self {
        Self {
            mac,
            expires_at: current_time + ttl,
        }
    }

    /// Check if this entry has expired
    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time >= self.expires_at
    }
}

/// ARP cache for storing IP → MAC address mappings
pub struct ArpCache {
    /// Map of IP addresses to cache entries
    entries: Mutex<BTreeMap<Ipv4Addr, ArpEntry>>,
}

impl ArpCache {
    /// Create a new ARP cache
    pub const fn new() -> Self {
        Self {
            entries: Mutex::new(BTreeMap::new()),
        }
    }

    /// Add or update an entry in the cache
    /// 
    /// # Arguments
    /// * `ip` - IP address
    /// * `mac` - MAC address
    /// * `current_time` - Current time in seconds since boot
    pub fn insert(&self, ip: Ipv4Addr, mac: [u8; 6], current_time: u64) {
        let entry = ArpEntry::new(mac, current_time, ARP_CACHE_TTL_SECS);
        self.entries.lock().insert(ip, entry);
    }

    /// Look up a MAC address for an IP address
    /// 
    /// # Arguments
    /// * `ip` - IP address to look up
    /// * `current_time` - Current time in seconds since boot
    /// 
    /// # Returns
    /// Some(MAC address) if found and not expired, None otherwise
    pub fn lookup(&self, ip: Ipv4Addr, current_time: u64) -> Option<[u8; 6]> {
        let entries = self.entries.lock();
        
        if let Some(entry) = entries.get(&ip) {
            if !entry.is_expired(current_time) {
                return Some(entry.mac);
            }
        }
        
        None
    }

    /// Remove an entry from the cache
    pub fn remove(&self, ip: Ipv4Addr) -> Option<ArpEntry> {
        self.entries.lock().remove(&ip)
    }

    /// Remove all expired entries from the cache
    /// 
    /// # Arguments
    /// * `current_time` - Current time in seconds since boot
    /// 
    /// # Returns
    /// Number of entries removed
    pub fn remove_expired(&self, current_time: u64) -> usize {
        let mut entries = self.entries.lock();
        let initial_count = entries.len();
        
        entries.retain(|_, entry| !entry.is_expired(current_time));
        
        initial_count - entries.len()
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.entries.lock().len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.lock().is_empty()
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        self.entries.lock().clear();
    }

    /// Get all entries as a vector (for debugging/display)
    pub fn entries(&self) -> Vec<(Ipv4Addr, [u8; 6], u64)> {
        self.entries
            .lock()
            .iter()
            .map(|(ip, entry)| (*ip, entry.mac, entry.expires_at))
            .collect()
    }
}

/// Global ARP cache instance
static ARP_CACHE: ArpCache = ArpCache::new();

/// Get a reference to the global ARP cache
pub fn arp_cache() -> &'static ArpCache {
    &ARP_CACHE
}

/// Send an ARP request for an IP address
/// 
/// # Arguments
/// * `target_ip` - The IP address to resolve
/// * `our_mac` - Our MAC address
/// * `our_ip` - Our IP address
/// 
/// # Returns
/// The raw ARP request packet bytes (ready to be wrapped in Ethernet frame)
pub fn create_arp_request(target_ip: Ipv4Addr, our_mac: [u8; 6], our_ip: Ipv4Addr) -> Vec<u8> {
    let packet = ArpPacket::new_request(our_mac, our_ip, target_ip);
    packet.to_bytes()
}

/// Create an ARP reply packet
/// 
/// # Arguments
/// * `target_ip` - The IP address of the requester
/// * `target_mac` - The MAC address of the requester
/// * `our_mac` - Our MAC address
/// * `our_ip` - Our IP address
/// 
/// # Returns
/// The raw ARP reply packet bytes (ready to be wrapped in Ethernet frame)
pub fn create_arp_reply(
    target_ip: Ipv4Addr,
    target_mac: [u8; 6],
    our_mac: [u8; 6],
    our_ip: Ipv4Addr,
) -> Vec<u8> {
    let packet = ArpPacket::new_reply(our_mac, our_ip, target_mac, target_ip);
    packet.to_bytes()
}

/// Handle an incoming ARP packet
/// 
/// # Arguments
/// * `packet_data` - Raw ARP packet data
/// * `our_ip` - Our IP address (to check if the request is for us)
/// * `our_mac` - Our MAC address (for sending replies)
/// * `current_time` - Current time in seconds since boot
/// 
/// # Returns
/// - Ok(Some(reply_bytes)) if we should send an ARP reply
/// - Ok(None) if the packet was processed but no reply is needed
/// - Err(ArpError) if there was an error parsing the packet
pub fn handle_arp_packet(
    packet_data: &[u8],
    our_ip: Ipv4Addr,
    our_mac: [u8; 6],
    current_time: u64,
) -> Result<Option<Vec<u8>>, ArpError> {
    let packet = ArpPacket::from_bytes(packet_data)?;

    // Always update cache with sender's info (gratuitous ARP support)
    arp_cache().insert(packet.sender_ip, packet.sender_mac, current_time);

    if packet.is_request() {
        // Check if the request is for our IP address
        if packet.target_ip == our_ip {
            // Send a reply
            let reply = create_arp_reply(packet.sender_ip, packet.sender_mac, our_mac, our_ip);
            return Ok(Some(reply));
        }
    } else if packet.is_reply() {
        // Reply received - cache was already updated above
        // No response needed
    }

    Ok(None)
}

/// Format a MAC address for display
pub fn format_mac(mac: &[u8; 6]) -> alloc::string::String {
    use alloc::format;
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}
