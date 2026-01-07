//! Network stack implementation
//! Phase 1+ - Networking Roadmap

pub mod buffer;
pub mod ethernet;
pub mod arp;
pub mod ipv4;
pub mod icmp;
pub mod stack;
pub mod loopback;  // Phase 5.2 - Loopback interface
pub mod udp;       // Phase 6.1 - UDP protocol
pub mod dns;       // Phase 6.2 - DNS client
pub mod tcp;       // Phase 7 - TCP protocol
