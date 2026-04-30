//! DHCP (Dynamic Host Configuration Protocol) Client Implementation
//!
//! RFC 2131: https://tools.ietf.org/html/rfc2131
//!
//! Basic DHCP client for automatic IP address configuration. Handles discover,
//! offer, request, and acknowledgment phases for leasing IP addresses.

extern crate alloc;
use alloc::vec::Vec;
use core::fmt;
use core::net::Ipv4Addr;

use crate::net::udp::UdpSocket;
use crate::task::yield_now;

/// DHCP client port (UDP 68)
pub const DHCP_CLIENT_PORT: u16 = 68;

/// DHCP server port (UDP 67)
pub const DHCP_SERVER_PORT: u16 = 67;

/// DHCP broadcast address
pub const DHCP_BROADCAST: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 255);

/// DHCP magic cookie (identifies DHCP options)
pub const DHCP_MAGIC_COOKIE: u32 = 0x63825363;

/// DHCP message type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpMessageType {
    Discover = 1,
    Offer = 2,
    Request = 3,
    Decline = 4,
    Ack = 5,
    Nak = 6,
    Release = 7,
}

impl DhcpMessageType {
    /// Convert from byte
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            1 => Some(DhcpMessageType::Discover),
            2 => Some(DhcpMessageType::Offer),
            3 => Some(DhcpMessageType::Request),
            4 => Some(DhcpMessageType::Decline),
            5 => Some(DhcpMessageType::Ack),
            6 => Some(DhcpMessageType::Nak),
            7 => Some(DhcpMessageType::Release),
            _ => None,
        }
    }
}

/// DHCP error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpError {
    /// Packet too short
    PacketTooShort,
    /// Invalid magic cookie
    InvalidMagicCookie,
    /// Timeout waiting for response
    Timeout,
    /// Failed to send DHCP message
    SendFailed,
    /// Failed to parse DHCP response
    ParseError,
    /// Server returned NAK or error
    ServerError,
    /// Failed to bind UDP socket
    BindFailed,
    /// No IP address offered
    NoIpOffered,
}

impl fmt::Display for DhcpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DhcpError::PacketTooShort => write!(f, "DHCP packet too short"),
            DhcpError::InvalidMagicCookie => write!(f, "Invalid DHCP magic cookie"),
            DhcpError::Timeout => write!(f, "DHCP timeout"),
            DhcpError::SendFailed => write!(f, "Failed to send DHCP message"),
            DhcpError::ParseError => write!(f, "Failed to parse DHCP response"),
            DhcpError::ServerError => write!(f, "DHCP server error"),
            DhcpError::BindFailed => write!(f, "Failed to bind UDP socket"),
            DhcpError::NoIpOffered => write!(f, "No IP address offered"),
        }
    }
}

/// DHCP packet structure (simplified)
#[derive(Debug, Clone)]
pub struct DhcpPacket {
    /// Operation: 1=request, 2=reply
    pub op: u8,
    /// Hardware type: 1=Ethernet
    pub htype: u8,
    /// Hardware address length (6 for MAC)
    pub hlen: u8,
    /// Hops (client sets to 0)
    pub hops: u8,
    /// Transaction ID (chosen by client)
    pub xid: u32,
    /// Seconds elapsed since client started trying to acquire lease
    pub secs: u16,
    /// Flags (bit 15: broadcast, 0-14: reserved)
    pub flags: u16,
    /// Client IP address (0.0.0.0 initially)
    pub ciaddr: Ipv4Addr,
    /// Your IP address (assigned by server)
    pub yiaddr: Ipv4Addr,
    /// Server IP address
    pub siaddr: Ipv4Addr,
    /// Gateway IP address
    pub giaddr: Ipv4Addr,
    /// Client hardware address (MAC)
    pub chaddr: [u8; 16],
    /// Server host name (optional)
    pub sname: [u8; 64],
    /// Boot file name (optional)
    pub file: [u8; 128],
    /// DHCP options
    pub options: Vec<u8>,
}

impl DhcpPacket {
    /// Create new DHCP DISCOVER packet
    pub fn new_discover(mac: &[u8; 6]) -> Self {
        use crate::serial_println;
        
        static mut XID: u32 = 0x12345678;
        
        let xid = unsafe {
            let x = XID;
            XID = XID.wrapping_add(1);
            x
        };

        let mut chaddr = [0u8; 16];
        chaddr[0..6].copy_from_slice(mac);

        let mut packet = DhcpPacket {
            op: 1,           // Request
            htype: 1,        // Ethernet
            hlen: 6,         // MAC length
            hops: 0,
            xid,
            secs: 0,
            flags: 0x8000,   // Broadcast
            ciaddr: Ipv4Addr::new(0, 0, 0, 0),
            yiaddr: Ipv4Addr::new(0, 0, 0, 0),
            siaddr: Ipv4Addr::new(0, 0, 0, 0),
            giaddr: Ipv4Addr::new(0, 0, 0, 0),
            chaddr,
            sname: [0; 64],
            file: [0; 128],
            options: Vec::new(),
        };

        // Add DHCP options
        packet.add_option(53, &[DhcpMessageType::Discover as u8]); // Message type
        packet.add_option(55, &[1, 3, 6, 15, 31, 33, 43, 44, 46, 47, 119, 120, 121]); // Requested params
        packet.add_option(255, &[]); // End option

        serial_println!("[DHCP] Created DISCOVER packet (xid=0x{:08x})", xid);

        packet
    }

    /// Create new DHCP REQUEST packet
    pub fn new_request(mac: &[u8; 6], offered_ip: Ipv4Addr, server_ip: Ipv4Addr, xid: u32) -> Self {
        let mut chaddr = [0u8; 16];
        chaddr[0..6].copy_from_slice(mac);

        let mut packet = DhcpPacket {
            op: 1,           // Request
            htype: 1,        // Ethernet
            hlen: 6,         // MAC length
            hops: 0,
            xid,
            secs: 0,
            flags: 0x8000,   // Broadcast
            ciaddr: Ipv4Addr::new(0, 0, 0, 0),
            yiaddr: Ipv4Addr::new(0, 0, 0, 0),
            siaddr: Ipv4Addr::new(0, 0, 0, 0),
            giaddr: Ipv4Addr::new(0, 0, 0, 0),
            chaddr,
            sname: [0; 64],
            file: [0; 128],
            options: Vec::new(),
        };

        // Add DHCP options
        packet.add_option(53, &[DhcpMessageType::Request as u8]); // Message type
        packet.add_option(50, &offered_ip.octets()); // Requested IP
        packet.add_option(54, &server_ip.octets()); // DHCP server
        packet.add_option(255, &[]); // End option

        packet
    }

    /// Add DHCP option
    pub fn add_option(&mut self, code: u8, value: &[u8]) {
        self.options.push(code);
        if code != 255 {
            self.options.push(value.len() as u8);
            self.options.extend_from_slice(value);
        }
    }

    /// Parse DHCP option
    pub fn parse_option(data: &[u8], offset: &mut usize) -> Option<(u8, Vec<u8>)> {
        if *offset >= data.len() {
            return None;
        }

        let code = data[*offset];
        *offset += 1;

        if code == 255 {
            // End option
            return Some((255, Vec::new()));
        }

        if *offset >= data.len() {
            return None;
        }

        let length = data[*offset] as usize;
        *offset += 1;

        if *offset + length > data.len() {
            return None;
        }

        let value = data[*offset..*offset + length].to_vec();
        *offset += length;

        Some((code, value))
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(576); // Minimum DHCP packet size

        bytes.push(self.op);
        bytes.push(self.htype);
        bytes.push(self.hlen);
        bytes.push(self.hops);

        bytes.extend_from_slice(&self.xid.to_be_bytes());
        bytes.extend_from_slice(&self.secs.to_be_bytes());
        bytes.extend_from_slice(&self.flags.to_be_bytes());

        bytes.extend_from_slice(&self.ciaddr.octets());
        bytes.extend_from_slice(&self.yiaddr.octets());
        bytes.extend_from_slice(&self.siaddr.octets());
        bytes.extend_from_slice(&self.giaddr.octets());

        bytes.extend_from_slice(&self.chaddr);
        bytes.extend_from_slice(&self.sname);
        bytes.extend_from_slice(&self.file);

        // Magic cookie
        bytes.extend_from_slice(&DHCP_MAGIC_COOKIE.to_be_bytes());

        // Options
        bytes.extend_from_slice(&self.options);

        // Pad to minimum size
        while bytes.len() < 300 {
            bytes.push(0);
        }

        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, DhcpError> {
        if data.len() < 240 {
            return Err(DhcpError::PacketTooShort);
        }

        let op = data[0];
        let htype = data[1];
        let hlen = data[2];
        let hops = data[3];

        let xid = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let secs = u16::from_be_bytes([data[8], data[9]]);
        let flags = u16::from_be_bytes([data[10], data[11]]);

        let ciaddr = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
        let yiaddr = Ipv4Addr::new(data[16], data[17], data[18], data[19]);
        let siaddr = Ipv4Addr::new(data[20], data[21], data[22], data[23]);
        let giaddr = Ipv4Addr::new(data[24], data[25], data[26], data[27]);

        let mut chaddr = [0u8; 16];
        chaddr.copy_from_slice(&data[28..44]);

        let mut sname = [0u8; 64];
        sname.copy_from_slice(&data[44..108]);

        let mut file = [0u8; 128];
        file.copy_from_slice(&data[108..236]);

        // Check magic cookie
        let magic = u32::from_be_bytes([data[236], data[237], data[238], data[239]]);
        if magic != DHCP_MAGIC_COOKIE {
            return Err(DhcpError::InvalidMagicCookie);
        }

        // Parse options
        let options = if data.len() > 240 {
            data[240..].to_vec()
        } else {
            Vec::new()
        };

        Ok(DhcpPacket {
            op,
            htype,
            hlen,
            hops,
            xid,
            secs,
            flags,
            ciaddr,
            yiaddr,
            siaddr,
            giaddr,
            chaddr,
            sname,
            file,
            options,
        })
    }

    /// Extract message type from options
    pub fn message_type(&self) -> Option<DhcpMessageType> {
        let mut offset = 0;
        while offset < self.options.len() {
            if let Some((code, value)) = Self::parse_option(&self.options, &mut offset) {
                if code == 53 && !value.is_empty() {
                    return DhcpMessageType::from_byte(value[0]);
                }
            }
        }
        None
    }

    /// Extract offered IP from options
    pub fn offered_ip(&self) -> Option<Ipv4Addr> {
        let mut offset = 0;
        while offset < self.options.len() {
            if let Some((code, value)) = Self::parse_option(&self.options, &mut offset) {
                if code == 50 && value.len() == 4 {
                    return Some(Ipv4Addr::new(value[0], value[1], value[2], value[3]));
                }
            }
        }
        None
    }

    /// Extract server IP from options
    pub fn server_ip(&self) -> Option<Ipv4Addr> {
        let mut offset = 0;
        while offset < self.options.len() {
            if let Some((code, value)) = Self::parse_option(&self.options, &mut offset) {
                if code == 54 && value.len() == 4 {
                    return Some(Ipv4Addr::new(value[0], value[1], value[2], value[3]));
                }
            }
        }
        None
    }

    /// Extract subnet mask from options
    pub fn subnet_mask(&self) -> Option<Ipv4Addr> {
        let mut offset = 0;
        while offset < self.options.len() {
            if let Some((code, value)) = Self::parse_option(&self.options, &mut offset) {
                if code == 1 && value.len() == 4 {
                    return Some(Ipv4Addr::new(value[0], value[1], value[2], value[3]));
                }
            }
        }
        None
    }

    /// Extract gateway from options
    pub fn gateway(&self) -> Option<Ipv4Addr> {
        let mut offset = 0;
        while offset < self.options.len() {
            if let Some((code, value)) = Self::parse_option(&self.options, &mut offset) {
                if code == 3 && value.len() >= 4 {
                    return Some(Ipv4Addr::new(value[0], value[1], value[2], value[3]));
                }
            }
        }
        None
    }

    /// Extract DNS servers from options
    pub fn dns_servers(&self) -> Vec<Ipv4Addr> {
        let mut result = Vec::new();
        let mut offset = 0;
        
        while offset < self.options.len() {
            if let Some((code, value)) = Self::parse_option(&self.options, &mut offset) {
                if code == 6 {
                    for chunk in value.chunks(4) {
                        if chunk.len() == 4 {
                            result.push(Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]));
                        }
                    }
                }
            }
        }
        
        result
    }

    /// Extract lease time from options (in seconds)
    pub fn lease_time(&self) -> Option<u32> {
        let mut offset = 0;
        while offset < self.options.len() {
            if let Some((code, value)) = Self::parse_option(&self.options, &mut offset) {
                if code == 51 && value.len() == 4 {
                    return Some(u32::from_be_bytes([value[0], value[1], value[2], value[3]]));
                }
            }
        }
        None
    }
}

/// DHCP configuration result
#[derive(Debug, Clone)]
pub struct DhcpConfig {
    pub ip_addr: Ipv4Addr,
    pub netmask: Ipv4Addr,
    pub gateway: Option<Ipv4Addr>,
    pub dns_servers: Vec<Ipv4Addr>,
    pub lease_time: u32,
}

/// Perform DHCP negotiation
///
/// # Arguments
/// * `mac` - Client MAC address
///
/// # Returns
/// * `Ok(DhcpConfig)` - Leased configuration
/// * `Err(DhcpError)` - Negotiation failed
pub async fn acquire_lease(mac: &[u8; 6]) -> Result<DhcpConfig, DhcpError> {
    const TIMEOUT_ITERATIONS: u32 = 1000;

    crate::serial_println!("[DHCP] Starting lease acquisition...");

    // Create and send DISCOVER
    let socket = UdpSocket::bind(DHCP_CLIENT_PORT).map_err(|_| DhcpError::BindFailed)?;
    
    let discover = DhcpPacket::new_discover(mac);
    let discover_bytes = discover.to_bytes();
    let xid = discover.xid;

    socket
        .send_to(&discover_bytes, DHCP_BROADCAST, DHCP_SERVER_PORT)
        .map_err(|_| DhcpError::SendFailed)?;

    crate::serial_println!("[DHCP] DISCOVER sent (xid=0x{:08x})", xid);

    // Wait for OFFER
    let mut offer_ip: Option<Ipv4Addr> = None;
    let mut server_ip = None;

    for _ in 0..TIMEOUT_ITERATIONS {
        match socket.recv_from() {
            Ok((data, _src_ip, _src_port)) => {
                if let Ok(packet) = DhcpPacket::from_bytes(&data) {
                    if packet.xid == xid && packet.message_type() == Some(DhcpMessageType::Offer) {
                        crate::serial_println!("[DHCP] OFFER received");
                        offer_ip = Some(packet.yiaddr);
                        server_ip = packet.server_ip();
                        break;
                    }
                }
            }
            Err(_) => {
                yield_now().await;
            }
        }
    }

    let offer_ip = offer_ip.ok_or(DhcpError::NoIpOffered)?;
    let server_ip = server_ip.ok_or(DhcpError::ServerError)?;

    crate::serial_println!("[DHCP] Offered IP: {}", offer_ip);

    // Send REQUEST
    let request = DhcpPacket::new_request(mac, offer_ip, server_ip, xid);
    let request_bytes = request.to_bytes();

    socket
        .send_to(&request_bytes, DHCP_BROADCAST, DHCP_SERVER_PORT)
        .map_err(|_| DhcpError::SendFailed)?;

    crate::serial_println!("[DHCP] REQUEST sent");

    // Wait for ACK
    for _ in 0..TIMEOUT_ITERATIONS {
        match socket.recv_from() {
            Ok((data, _src_ip, _src_port)) => {
                if let Ok(packet) = DhcpPacket::from_bytes(&data) {
                    if packet.xid == xid {
                        match packet.message_type() {
                            Some(DhcpMessageType::Ack) => {
                                crate::serial_println!("[DHCP] ACK received");
                                
                                return Ok(DhcpConfig {
                                    ip_addr: packet.yiaddr,
                                    netmask: packet.subnet_mask().unwrap_or(Ipv4Addr::new(255, 255, 255, 0)),
                                    gateway: packet.gateway(),
                                    dns_servers: packet.dns_servers(),
                                    lease_time: packet.lease_time().unwrap_or(3600),
                                });
                            }
                            Some(DhcpMessageType::Nak) => {
                                crate::serial_println!("[DHCP] NAK received");
                                return Err(DhcpError::ServerError);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(_) => {
                yield_now().await;
            }
        }
    }

    Err(DhcpError::Timeout)
}
