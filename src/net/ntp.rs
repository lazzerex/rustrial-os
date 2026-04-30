//! NTP (Network Time Protocol) Client Implementation
//!
//! RFC 5905: https://tools.ietf.org/html/rfc5905
//!
//! Basic NTP client for time synchronization. Queries NTP server and derives
//! time offset to seed TCP ISN generation and accurate timekeeping.

extern crate alloc;
use alloc::vec::Vec;
use core::fmt;
use core::net::Ipv4Addr;

use crate::net::udp::{UdpSocket, RecvError};
use crate::task::yield_now;

/// NTP port (UDP 123)
pub const NTP_PORT: u16 = 123;

/// Default test port for a local host-side NTP responder.
pub const NTP_TEST_PORT: u16 = 8123;

/// NTP packet size (minimum 48 bytes)
pub const NTP_PACKET_SIZE: usize = 48;

/// NTP protocol version
pub const NTP_VERSION: u8 = 3;

/// Mode: client (3)
pub const MODE_CLIENT: u8 = 3;

/// Mode: server (4)
pub const MODE_SERVER: u8 = 4;

/// NTP error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtpError {
    /// Packet too short
    PacketTooShort,
    /// Invalid NTP version
    InvalidVersion,
    /// Timeout waiting for response
    Timeout,
    /// Failed to send NTP request
    SendFailed,
    /// Failed to parse NTP response
    ParseError,
    /// Server returned error
    ServerError,
    /// Failed to bind UDP socket
    BindFailed,
}

impl fmt::Display for NtpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NtpError::PacketTooShort => write!(f, "NTP packet too short"),
            NtpError::InvalidVersion => write!(f, "Invalid NTP version"),
            NtpError::Timeout => write!(f, "NTP query timed out"),
            NtpError::SendFailed => write!(f, "Failed to send NTP request"),
            NtpError::ParseError => write!(f, "Failed to parse NTP response"),
            NtpError::ServerError => write!(f, "NTP server error"),
            NtpError::BindFailed => write!(f, "Failed to bind UDP socket"),
        }
    }
}

/// NTP timestamp: 64-bit fixed-point (32-bit seconds, 32-bit fraction)
#[derive(Debug, Clone, Copy)]
pub struct NtpTimestamp {
    /// Seconds since 1900-01-01 00:00:00
    pub seconds: u32,
    /// Fractional part (0..2^32-1)
    pub fraction: u32,
}

impl NtpTimestamp {
    /// Convert NTP timestamp to Unix timestamp (seconds since 1970)
    /// Returns offset from system clock
    pub fn to_offset(&self) -> i64 {
        // NTP epoch is 1900-01-01, Unix epoch is 1970-01-01
        // Difference: 70 years = 2208988800 seconds
        const NTP_UNIX_OFFSET: i64 = 2208988800;
        
        let ntp_seconds = self.seconds as i64;
        let unix_seconds = ntp_seconds - NTP_UNIX_OFFSET;
        
        unix_seconds
    }

    /// Convert from big-endian bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, NtpError> {
        if bytes.len() < 8 {
            return Err(NtpError::PacketTooShort);
        }

        let seconds = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let fraction = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        Ok(NtpTimestamp { seconds, fraction })
    }

    /// Convert to big-endian bytes
    fn to_bytes(&self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[0..4].copy_from_slice(&self.seconds.to_be_bytes());
        bytes[4..8].copy_from_slice(&self.fraction.to_be_bytes());
        bytes
    }
}

/// NTP packet structure
#[derive(Debug, Clone)]
pub struct NtpPacket {
    /// LI (leap indicator): 0=no warning
    pub leap_indicator: u8,
    /// VN (version number): 3 or 4
    pub version: u8,
    /// Mode: 3=client, 4=server
    pub mode: u8,
    /// Stratum: 0=unspecified, 16=invalid
    pub stratum: u8,
    /// Poll interval (log2 seconds)
    pub poll: u8,
    /// Precision (log2 seconds)
    pub precision: u8,
    /// Root delay (fixed-point)
    pub root_delay: u32,
    /// Root dispersion (fixed-point)
    pub root_dispersion: u32,
    /// Reference ID
    pub reference_id: u32,
    /// Reference timestamp
    pub reference_ts: NtpTimestamp,
    /// Originate timestamp
    pub origin_ts: NtpTimestamp,
    /// Receive timestamp
    pub receive_ts: NtpTimestamp,
    /// Transmit timestamp
    pub transmit_ts: NtpTimestamp,
}

impl NtpPacket {
    /// Create a new NTP client request packet
    pub fn new_request() -> Self {
        NtpPacket {
            leap_indicator: 0,   // No warning
            version: NTP_VERSION, // Version 3
            mode: MODE_CLIENT,    // Client
            stratum: 0,
            poll: 0,
            precision: 0,
            root_delay: 0,
            root_dispersion: 0,
            reference_id: 0,
            reference_ts: NtpTimestamp { seconds: 0, fraction: 0 },
            origin_ts: NtpTimestamp { seconds: 0, fraction: 0 },
            receive_ts: NtpTimestamp { seconds: 0, fraction: 0 },
            transmit_ts: NtpTimestamp { seconds: 0, fraction: 0 },
        }
    }

    /// Parse NTP packet from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, NtpError> {
        if data.len() < NTP_PACKET_SIZE {
            return Err(NtpError::PacketTooShort);
        }

        // Byte 0: LI (2 bits), VN (3 bits), Mode (3 bits)
        let byte0 = data[0];
        let leap_indicator = (byte0 >> 6) & 0x03;
        let version = (byte0 >> 3) & 0x07;
        let mode = byte0 & 0x07;

        if version != NTP_VERSION && version != 4 {
            return Err(NtpError::InvalidVersion);
        }

        let stratum = data[1];
        let poll = data[2] as u8;
        let precision = data[3] as i8 as u8;

        let root_delay = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let root_dispersion = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        let reference_id = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);

        let reference_ts = NtpTimestamp::from_bytes(&data[16..24])?;
        let origin_ts = NtpTimestamp::from_bytes(&data[24..32])?;
        let receive_ts = NtpTimestamp::from_bytes(&data[32..40])?;
        let transmit_ts = NtpTimestamp::from_bytes(&data[40..48])?;

        Ok(NtpPacket {
            leap_indicator,
            version,
            mode,
            stratum,
            poll,
            precision,
            root_delay,
            root_dispersion,
            reference_id,
            reference_ts,
            origin_ts,
            receive_ts,
            transmit_ts,
        })
    }

    /// Serialize NTP packet to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(NTP_PACKET_SIZE);

        // Byte 0: LI (2 bits), VN (3 bits), Mode (3 bits)
        let byte0 = ((self.leap_indicator & 0x03) << 6)
                  | ((self.version & 0x07) << 3)
                  | (self.mode & 0x07);
        bytes.push(byte0);

        bytes.push(self.stratum);
        bytes.push(self.poll);
        bytes.push(self.precision);

        bytes.extend_from_slice(&self.root_delay.to_be_bytes());
        bytes.extend_from_slice(&self.root_dispersion.to_be_bytes());
        bytes.extend_from_slice(&self.reference_id.to_be_bytes());

        bytes.extend_from_slice(&self.reference_ts.to_bytes());
        bytes.extend_from_slice(&self.origin_ts.to_bytes());
        bytes.extend_from_slice(&self.receive_ts.to_bytes());
        bytes.extend_from_slice(&self.transmit_ts.to_bytes());

        bytes
    }

    /// Check if this is a valid server response
    pub fn is_valid_response(&self) -> bool {
        self.mode == MODE_SERVER && self.stratum > 0 && self.stratum < 16
    }
}

/// Global time offset in seconds
static mut TIME_OFFSET: i64 = 0;

/// Get current time offset (NTP adjustment)
pub fn get_time_offset() -> i64 {
    unsafe { TIME_OFFSET }
}

/// Set time offset from NTP response
pub fn set_time_offset(offset: i64) {
    unsafe {
        TIME_OFFSET = offset;
    }
    crate::serial_println!("[NTP] Time offset set to {} seconds", offset);
}

/// Query NTP server and sync time
///
/// # Arguments
/// * `server_ip` - IP address of NTP server (default: QEMU NTP at 10.0.2.2)
///
/// # Returns
/// * `Ok(offset)` - Time offset in seconds
/// * `Err(NtpError)` - Query failed
pub async fn query_ntp(server_ip: Option<Ipv4Addr>) -> Result<i64, NtpError> {
    query_ntp_with_port(server_ip, NTP_PORT).await
}

/// Query NTP server on a custom UDP port.
pub async fn query_ntp_with_port(server_ip: Option<Ipv4Addr>, port: u16) -> Result<i64, NtpError> {
    let server = server_ip.unwrap_or(Ipv4Addr::new(10, 0, 2, 2));
    const TIMEOUT_ITERATIONS: u32 = 1000;

    crate::serial_println!("[NTP] Querying {} for time sync...", server);

    // Bind to ephemeral port
    let socket = UdpSocket::bind(0).map_err(|_| NtpError::BindFailed)?;

    // Create NTP request
    let request = NtpPacket::new_request();
    let request_bytes = request.to_bytes();

    // Send request
    socket
        .send_to(&request_bytes, server, port)
        .map_err(|_| NtpError::SendFailed)?;

    crate::serial_println!("[NTP] Request sent, waiting for response...");

    // Wait for response with timeout
    for _ in 0..TIMEOUT_ITERATIONS {
        match socket.recv_from() {
            Ok((data, _src_ip, _src_port)) => {
                // Parse response
                let response = NtpPacket::from_bytes(&data)?;

                if !response.is_valid_response() {
                    crate::serial_println!("[NTP] Invalid response from server");
                    return Err(NtpError::ServerError);
                }

                let offset = response.transmit_ts.to_offset();
                crate::serial_println!("[NTP] Received timestamp: {} seconds since Unix epoch", offset);

                set_time_offset(offset);
                return Ok(offset);
            }
            Err(RecvError::WouldBlock) => {
                // No data yet, yield and try again
                yield_now().await;
            }
        }
    }

    crate::serial_println!("[NTP] Query timeout");
    Err(NtpError::Timeout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ntp_timestamp_to_offset() {
        // Unix epoch in NTP: 2208988800 seconds
        let ts = NtpTimestamp {
            seconds: 2208988800,
            fraction: 0,
        };
        assert_eq!(ts.to_offset(), 0);

        // One day after Unix epoch
        let ts2 = NtpTimestamp {
            seconds: 2208988800 + 86400,
            fraction: 0,
        };
        assert_eq!(ts2.to_offset(), 86400);
    }

    #[test]
    fn test_ntp_packet_new_request() {
        let packet = NtpPacket::new_request();
        assert_eq!(packet.version, NTP_VERSION);
        assert_eq!(packet.mode, MODE_CLIENT);
        assert_eq!(packet.leap_indicator, 0);
    }

    #[test]
    fn test_ntp_packet_serialization() {
        let packet = NtpPacket::new_request();
        let bytes = packet.to_bytes();
        assert_eq!(bytes.len(), NTP_PACKET_SIZE);

        // Check first byte
        let byte0 = bytes[0];
        assert_eq!((byte0 >> 6) & 0x03, 0);         // LI
        assert_eq!((byte0 >> 3) & 0x07, NTP_VERSION); // VN
        assert_eq!(byte0 & 0x07, MODE_CLIENT);       // Mode
    }

    #[test]
    fn test_ntp_packet_deserialization() {
        let mut data = vec![0u8; NTP_PACKET_SIZE];
        data[0] = (0 << 6) | (NTP_VERSION << 3) | MODE_SERVER; // LI=0, VN=3, Mode=4 (server)
        data[1] = 2; // Stratum

        let packet = NtpPacket::from_bytes(&data).unwrap();
        assert_eq!(packet.leap_indicator, 0);
        assert_eq!(packet.version, NTP_VERSION);
        assert_eq!(packet.mode, MODE_SERVER);
        assert_eq!(packet.stratum, 2);
    }
}
