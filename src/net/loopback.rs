//! Loopback Network Interface (127.0.0.1)
//! 
//!
//! implement a virtual loopback interface that echoes all
//! transmitted packets back as received packets. This allows testing
//! the network stack without physical hardware.

use alloc::vec::Vec;
use alloc::collections::VecDeque;
use spin::Mutex;

use crate::drivers::net::{NetworkDevice, TransmitError, LinkStatus};

/// Loopback network device
///
/// This device stores transmitted packets in a queue and returns them
/// when receive() is called, effectively echoing packets back.
pub struct LoopbackDevice {
    /// MAC address (all zeros for loopback)
    mac_addr: [u8; 6],
    /// Queue of packets to be "received" (packets that were transmitted)
    rx_queue: Mutex<VecDeque<Vec<u8>>>,
    /// Maximum queue size to prevent unbounded memory growth
    max_queue_size: usize,
}

impl LoopbackDevice {
    /// Create a new loopback device
    ///
    /// # Arguments
    /// * `max_queue_size` - Maximum number of packets to queue (default: 64)
    ///
    /// # Returns
    /// A new loopback device instance
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            mac_addr: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            rx_queue: Mutex::new(VecDeque::with_capacity(max_queue_size)),
            max_queue_size,
        }
    }

    /// Create a loopback device with default settings
    pub fn default() -> Self {
        Self::new(64)
    }
}

impl NetworkDevice for LoopbackDevice {
    fn mac_address(&self) -> [u8; 6] {
        self.mac_addr
    }

    fn transmit(&mut self, packet: &[u8]) -> Result<(), TransmitError> {
        let mut queue = self.rx_queue.lock();
        
        // Check if queue is full
        if queue.len() >= self.max_queue_size {
            return Err(TransmitError::BufferFull);
        }

        // Echo the packet back by adding it to the RX queue
        queue.push_back(packet.to_vec());
        
        Ok(())
    }

    fn receive(&mut self) -> Option<Vec<u8>> {
        let mut queue = self.rx_queue.lock();
        queue.pop_front()
    }

    fn link_status(&self) -> LinkStatus {
        // Loopback is always "up"
        LinkStatus::Up
    }

    fn device_name(&self) -> &str {
        "lo (loopback)"
    }

    fn is_ready(&self) -> bool {
        // Loopback is always ready
        true
    }
}
