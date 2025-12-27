//! DNS (Domain Name System) Client Implementation
//!
//! This module implements a basic DNS client for resolving hostnames to IPv4 addresses.
//! It supports DNS queries and parsing of A records (IPv4 address records).
//!
//! # RFC References
//! - RFC 1035: Domain Names - Implementation and Specification
//!
//! # Example
//! ```rust
//! let ip = resolve("google.com").await?;
//! println!("google.com resolved to {}", ip);
//! ```

use alloc::vec::Vec;
use core::fmt;
use crate::net::ipv4::Ipv4Addr;
use crate::net::udp::{UdpSocket, RecvError};
use crate::task::yield_now;

/// DNS error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsError {
    /// Invalid domain name format
    InvalidDomain,
    /// DNS query timed out
    Timeout,
    /// Failed to send DNS query
    SendFailed,
    /// Failed to parse DNS response
    ParseError,
    /// No A records found in response
    NoRecords,
    /// Server returned error
    ServerError,
    /// Failed to bind UDP socket
    BindFailed,
}

impl fmt::Display for DnsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DnsError::InvalidDomain => write!(f, "Invalid domain name"),
            DnsError::Timeout => write!(f, "DNS query timed out"),
            DnsError::SendFailed => write!(f, "Failed to send DNS query"),
            DnsError::ParseError => write!(f, "Failed to parse DNS response"),
            DnsError::NoRecords => write!(f, "No A records found"),
            DnsError::ServerError => write!(f, "DNS server returned error"),
            DnsError::BindFailed => write!(f, "Failed to bind UDP socket"),
        }
    }
}

/// DNS header structure (12 bytes)
#[derive(Debug, Clone, Copy)]
struct DnsHeader {
    id: u16,           // Transaction ID
    flags: u16,        // Flags (QR, Opcode, AA, TC, RD, RA, Z, RCODE)
    qdcount: u16,      // Number of questions
    ancount: u16,      // Number of answers
    nscount: u16,      // Number of authority records
    arcount: u16,      // Number of additional records
}

impl DnsHeader {
    /// Create a new DNS query header
    fn new_query(id: u16) -> Self {
        Self {
            id,
            flags: 0x0100,  // Standard query with recursion desired (RD=1)
            qdcount: 1,     // One question
            ancount: 0,
            nscount: 0,
            arcount: 0,
        }
    }

    /// Parse DNS header from bytes
    fn from_bytes(data: &[u8]) -> Result<Self, DnsError> {
        if data.len() < 12 {
            return Err(DnsError::ParseError);
        }

        Ok(Self {
            id: u16::from_be_bytes([data[0], data[1]]),
            flags: u16::from_be_bytes([data[2], data[3]]),
            qdcount: u16::from_be_bytes([data[4], data[5]]),
            ancount: u16::from_be_bytes([data[6], data[7]]),
            nscount: u16::from_be_bytes([data[8], data[9]]),
            arcount: u16::from_be_bytes([data[10], data[11]]),
        })
    }

    /// Convert header to bytes
    fn to_bytes(&self) -> [u8; 12] {
        let mut bytes = [0u8; 12];
        bytes[0..2].copy_from_slice(&self.id.to_be_bytes());
        bytes[2..4].copy_from_slice(&self.flags.to_be_bytes());
        bytes[4..6].copy_from_slice(&self.qdcount.to_be_bytes());
        bytes[6..8].copy_from_slice(&self.ancount.to_be_bytes());
        bytes[8..10].copy_from_slice(&self.nscount.to_be_bytes());
        bytes[10..12].copy_from_slice(&self.arcount.to_be_bytes());
        bytes
    }

    /// Check if this is a response (QR bit set)
    fn is_response(&self) -> bool {
        (self.flags & 0x8000) != 0
    }

    /// Get response code (RCODE)
    fn rcode(&self) -> u8 {
        (self.flags & 0x000F) as u8
    }

    /// Check if response is successful (RCODE = 0)
    fn is_success(&self) -> bool {
        self.rcode() == 0
    }
}

/// DNS question structure
#[derive(Debug, Clone)]
struct DnsQuestion {
    qname: Vec<u8>,    // Encoded domain name
    qtype: u16,        // Query type (1 = A record)
    qclass: u16,       // Query class (1 = IN for Internet)
}

impl DnsQuestion {
    /// Create a new DNS question for an A record
    fn new_a_query(domain: &str) -> Result<Self, DnsError> {
        let qname = encode_domain_name(domain)?;
        Ok(Self {
            qname,
            qtype: 1,      // A record
            qclass: 1,     // IN (Internet)
        })
    }

    /// Convert question to bytes
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.qname);
        bytes.extend_from_slice(&self.qtype.to_be_bytes());
        bytes.extend_from_slice(&self.qclass.to_be_bytes());
        bytes
    }
}

/// DNS resource record (answer)
#[derive(Debug, Clone)]
struct DnsRecord {
    name: Vec<u8>,     // Domain name (may be compressed)
    rtype: u16,        // Record type
    rclass: u16,       // Record class
    ttl: u32,          // Time to live
    rdlength: u16,     // Data length
    rdata: Vec<u8>,    // Record data
}

impl DnsRecord {
    /// Parse DNS record from bytes, starting at offset
    fn from_bytes(data: &[u8], offset: &mut usize) -> Result<Self, DnsError> {
        // Parse name (may be compressed)
        let name = parse_domain_name(data, offset)?;

        // Check we have enough bytes for the rest of the record
        if *offset + 10 > data.len() {
            return Err(DnsError::ParseError);
        }

        let rtype = u16::from_be_bytes([data[*offset], data[*offset + 1]]);
        *offset += 2;

        let rclass = u16::from_be_bytes([data[*offset], data[*offset + 1]]);
        *offset += 2;

        let ttl = u32::from_be_bytes([
            data[*offset],
            data[*offset + 1],
            data[*offset + 2],
            data[*offset + 3],
        ]);
        *offset += 4;

        let rdlength = u16::from_be_bytes([data[*offset], data[*offset + 1]]);
        *offset += 2;

        // Check we have enough bytes for rdata
        if *offset + rdlength as usize > data.len() {
            return Err(DnsError::ParseError);
        }

        let rdata = data[*offset..*offset + rdlength as usize].to_vec();
        *offset += rdlength as usize;

        Ok(Self {
            name,
            rtype,
            rclass,
            ttl,
            rdlength,
            rdata,
        })
    }

    /// Extract IPv4 address from A record
    fn as_ipv4(&self) -> Option<Ipv4Addr> {
        if self.rtype == 1 && self.rdata.len() == 4 {
            Some(Ipv4Addr::new(
                self.rdata[0],
                self.rdata[1],
                self.rdata[2],
                self.rdata[3],
            ))
        } else {
            None
        }
    }
}

/// Encode domain name in DNS format
/// Example: "google.com" -> [6]google[3]com[0]
fn encode_domain_name(domain: &str) -> Result<Vec<u8>, DnsError> {
    let mut encoded = Vec::new();

    // Split domain into labels
    let labels: Vec<&str> = domain.split('.').collect();

    for label in labels {
        if label.is_empty() || label.len() > 63 {
            return Err(DnsError::InvalidDomain);
        }

        // Write label length
        encoded.push(label.len() as u8);

        // Write label characters
        encoded.extend_from_slice(label.as_bytes());
    }

    // Terminate with zero-length label
    encoded.push(0);

    Ok(encoded)
}

/// Parse domain name from DNS packet (supports compression)
fn parse_domain_name(data: &[u8], offset: &mut usize) -> Result<Vec<u8>, DnsError> {
    let mut name = Vec::new();
    let mut jumped = false;
    let mut jump_offset = *offset;
    let max_jumps = 5;
    let mut jumps = 0;

    loop {
        if jump_offset >= data.len() {
            return Err(DnsError::ParseError);
        }

        let length = data[jump_offset];

        // Check for compression (top 2 bits set)
        if (length & 0xC0) == 0xC0 {
            // Pointer to another location
            if jump_offset + 1 >= data.len() {
                return Err(DnsError::ParseError);
            }

            let pointer = u16::from_be_bytes([length & 0x3F, data[jump_offset + 1]]) as usize;

            if !jumped {
                *offset = jump_offset + 2;
            }

            jump_offset = pointer;
            jumped = true;
            jumps += 1;

            if jumps > max_jumps {
                return Err(DnsError::ParseError);
            }

            continue;
        }

        if length == 0 {
            // End of name
            if !jumped {
                *offset = jump_offset + 1;
            }
            break;
        }

        // Regular label
        if jump_offset + 1 + length as usize > data.len() {
            return Err(DnsError::ParseError);
        }

        // Add length byte and label
        name.push(length);
        name.extend_from_slice(&data[jump_offset + 1..jump_offset + 1 + length as usize]);

        jump_offset += 1 + length as usize;
    }

    // Add terminating zero if not empty
    if !name.is_empty() {
        name.push(0);
    }

    Ok(name)
}

/// Build a DNS query packet
fn build_query(domain: &str, id: u16) -> Result<Vec<u8>, DnsError> {
    let header = DnsHeader::new_query(id);
    let question = DnsQuestion::new_a_query(domain)?;

    let mut packet = Vec::new();
    packet.extend_from_slice(&header.to_bytes());
    packet.extend_from_slice(&question.to_bytes());

    Ok(packet)
}

/// Parse DNS response and extract A records
fn parse_response(data: &[u8]) -> Result<Vec<Ipv4Addr>, DnsError> {
    if data.len() < 12 {
        return Err(DnsError::ParseError);
    }

    let header = DnsHeader::from_bytes(data)?;

    if !header.is_response() {
        return Err(DnsError::ParseError);
    }

    if !header.is_success() {
        return Err(DnsError::ServerError);
    }

    let mut offset = 12;
    let mut addresses = Vec::new();

    // Skip questions
    for _ in 0..header.qdcount {
        // Parse domain name
        parse_domain_name(data, &mut offset)?;

        // Skip QTYPE (2 bytes) and QCLASS (2 bytes)
        if offset + 4 > data.len() {
            return Err(DnsError::ParseError);
        }
        offset += 4;
    }

    // Parse answers
    for _ in 0..header.ancount {
        let record = DnsRecord::from_bytes(data, &mut offset)?;

        // Extract IPv4 address from A records
        if let Some(ip) = record.as_ipv4() {
            addresses.push(ip);
        }
    }

    if addresses.is_empty() {
        Err(DnsError::NoRecords)
    } else {
        Ok(addresses)
    }
}

/// Resolve a hostname to an IPv4 address using DNS
///
/// Sends a DNS query to 8.8.8.8 (Google Public DNS) and returns the first A record found.
///
/// # Arguments
/// * `hostname` - The hostname to resolve (e.g., "google.com")
///
/// # Returns
/// * `Ok(Ipv4Addr)` - The resolved IPv4 address
/// * `Err(DnsError)` - If resolution fails
///
/// # Example
/// ```rust
/// let ip = resolve("google.com").await?;
/// println!("Resolved to: {}", ip);
/// ```
pub async fn resolve(hostname: &str) -> Result<Ipv4Addr, DnsError> {
    const DNS_SERVER: Ipv4Addr = Ipv4Addr::new(8, 8, 8, 8);
    const DNS_PORT: u16 = 53;
    const TIMEOUT_ITERATIONS: u32 = 1000;  // ~10 seconds with 10ms yields

    // Bind to ephemeral port
    let socket = UdpSocket::bind(0).map_err(|_| DnsError::BindFailed)?;

    // Generate transaction ID (simple incrementing counter)
    static mut NEXT_ID: u16 = 1;
    let id = unsafe {
        let current = NEXT_ID;
        NEXT_ID = NEXT_ID.wrapping_add(1);
        current
    };

    // Build DNS query
    let query = build_query(hostname, id)?;

    // Send query
    socket
        .send_to(&query, DNS_SERVER, DNS_PORT)
        .map_err(|_| DnsError::SendFailed)?;

    // Wait for response with timeout
    for _ in 0..TIMEOUT_ITERATIONS {
        match socket.recv_from() {
            Ok((data, _src_ip, _src_port)) => {
                // Parse response
                let addresses = parse_response(&data)?;

                // Return first address
                return Ok(addresses[0]);
            }
            Err(RecvError::WouldBlock) => {
                // No data yet, yield and try again
                yield_now().await;
            }
        }
    }

    Err(DnsError::Timeout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_domain_name() {
        let encoded = encode_domain_name("google.com").unwrap();
        assert_eq!(
            encoded,
            vec![6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0]
        );
    }

    #[test]
    fn test_encode_single_label() {
        let encoded = encode_domain_name("localhost").unwrap();
        assert_eq!(
            encoded,
            vec![9, b'l', b'o', b'c', b'a', b'l', b'h', b'o', b's', b't', 0]
        );
    }

    #[test]
    fn test_dns_header() {
        let header = DnsHeader::new_query(0x1234);
        let bytes = header.to_bytes();

        assert_eq!(bytes[0..2], [0x12, 0x34]); // ID
        assert_eq!(bytes[2..4], [0x01, 0x00]); // Flags (RD=1)
        assert_eq!(bytes[4..6], [0x00, 0x01]); // QDCOUNT=1
    }

    #[test]
    fn test_build_query() {
        let query = build_query("example.com", 0x1234).unwrap();

        // Should have header (12 bytes) + question
        assert!(query.len() > 12);

        // Check header
        assert_eq!(query[0..2], [0x12, 0x34]); // ID
        assert_eq!(query[4..6], [0x00, 0x01]); // QDCOUNT=1
    }

    #[test]
    fn test_parse_response_header() {
        // Minimal DNS response header
        let data = [
            0x12, 0x34, // ID
            0x81, 0x80, // Flags (response, no error)
            0x00, 0x01, // QDCOUNT
            0x00, 0x01, // ANCOUNT
            0x00, 0x00, // NSCOUNT
            0x00, 0x00, // ARCOUNT
        ];

        let header = DnsHeader::from_bytes(&data).unwrap();
        assert_eq!(header.id, 0x1234);
        assert!(header.is_response());
        assert!(header.is_success());
        assert_eq!(header.ancount, 1);
    }
}
