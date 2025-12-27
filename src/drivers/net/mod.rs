// Network Device Abstraction Layer
pub mod rtl8139;
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


// Global Network Device Registry Here


use alloc::boxed::Box;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    /// Global loopback device (127.0.0.1)
    /// 
    /// The loopback device is always available and handles 127.0.0.1 traffic.
    pub static ref LOOPBACK_DEVICE: Mutex<Option<Box<dyn NetworkDevice>>> = Mutex::new(None);
    
    /// Global physical network device (primary NIC)
    /// 
    /// Stores the currently active physical network interface card (NIC).
    /// The protocol stack will use this for non-loopback traffic.
    /// 
    /// Thread Safety
    /// Protected by a Mutex for safe concurrent access.
    pub static ref NETWORK_DEVICE: Mutex<Option<Box<dyn NetworkDevice>>> = Mutex::new(None);
}

/// Register the loopback device
/// 
/// # Arguments
/// * `device` - The loopback device to register
pub fn register_loopback_device(device: Box<dyn NetworkDevice>) {
    *LOOPBACK_DEVICE.lock() = Some(device);
}

/// Register a network device as the active physical NIC
/// 
/// Arguments
/// * `device` - The network device to register (must implement NetworkDevice trait)
/// 
/// Example
/// ```
/// let rtl8139 = Rtl8139::new(...);
/// register_network_device(Box::new(rtl8139));
/// ```
pub fn register_network_device(device: Box<dyn NetworkDevice>) {
    *NETWORK_DEVICE.lock() = Some(device);
}

/// Get a reference to the global network device
/// 
/// Returns
/// * `Some(&Mutex<Option<Box<dyn NetworkDevice>>>)` if a device is registered
/// * `None` if no device is registered
pub fn get_network_device() -> &'static Mutex<Option<Box<dyn NetworkDevice>>> {
    &NETWORK_DEVICE
}

/// Check if a network device is registered
pub fn has_network_device() -> bool {
    NETWORK_DEVICE.lock().is_some()
}

/// Transmit a packet using the registered network device
/// 
///  Arguments
/// * `packet` - The raw Ethernet frame to transmit
/// 
/// Returns
/// * `Ok(())` if transmission was successful
/// * `Err(TransmitError)` if no device is registered or transmission failed
pub fn transmit_packet(packet: &[u8]) -> Result<(), TransmitError> {
    let mut device_guard = NETWORK_DEVICE.lock();
    match device_guard.as_mut() {
        Some(device) => device.transmit(packet),
        None => Err(TransmitError::NotInitialized),
    }
}

/// Receive a packet from the registered network device
/// 
/// Returns
/// * `Some(packet)` if a packet was received
/// * `None` if no device is registered or no packet is available
pub fn receive_packet() -> Option<Vec<u8>> {
    let mut device_guard = NETWORK_DEVICE.lock();
    device_guard.as_mut().and_then(|device| device.receive())
}

/// Get the MAC address of the registered network device
/// 
/// Returns
/// * `Some([u8; 6])` if a device is registered
/// * `None` if no device is registered
pub fn get_mac_address() -> Option<[u8; 6]> {
    let device_guard = NETWORK_DEVICE.lock();
    device_guard.as_ref().map(|device| device.mac_address())
}

/// Get the link status of the registered network device
/// 
/// Returns
/// * Link status if a device is registered
/// * `LinkStatus::Unknown` if no device is registered
pub fn get_link_status() -> LinkStatus {
    let device_guard = NETWORK_DEVICE.lock();
    device_guard.as_ref()
        .map(|device| device.link_status())
        .unwrap_or(LinkStatus::Unknown)
}


