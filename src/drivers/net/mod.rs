// Network Device Abstraction Layer

use alloc::vec::Vec;

/// link status of a network interface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkStatus {
    Up,
    Down,
    Unknown,
}

/// Errors that can occur during packet transmission
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransmitError {
    /// Packet too large for the device
    PacketTooLarge,
    /// TX buffer is full, try again later
    BufferFull,
    /// Device is not ready
    NotReady,
    /// Hardware error during transmission
    HardwareError,
    /// Device is not initialized
    NotInitialized,
}

/// Errors that can occur during packet reception
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiveError {
    /// No packet available
    NoPacket,
    /// CRC error in received packet
    CrcError,
    /// Frame alignment error
    AlignmentError,
    /// Packet is too large
    PacketTooLarge,
    /// Hardware error during reception
    HardwareError,
}

/// Network device trait that all network drivers must implement
pub trait NetworkDevice: Send + Sync {
    /// Get the MAC address of this device
    fn mac_address(&self) -> [u8; 6];

    /// Transmit a packet
    /// 
    /// # Arguments
    /// * `packet` - The raw Ethernet frame to transmit (including header)
    /// 
    /// # Returns
    /// * `Ok(())` if the packet was queued for transmission
    /// * `Err(TransmitError)` if transmission failed
    fn transmit(&mut self, packet: &[u8]) -> Result<(), TransmitError>;

    /// Receive a packet if one is available
    /// 
    /// # Returns
    /// * `Some(packet)` if a packet was received
    /// * `None` if no packet is available
    fn receive(&mut self) -> Option<Vec<u8>>;

    /// Get the current link status
    fn link_status(&self) -> LinkStatus;

    /// Get device name/identifier
    fn device_name(&self) -> &str;

    /// Check if the device is initialized and ready
    fn is_ready(&self) -> bool;
}

pub mod rtl8139;
