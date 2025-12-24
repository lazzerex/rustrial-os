//! Circular ring buffer for RX/TX network packets
//! Phase 1.1 - Networking Roadmap
//!
//! Provides fixed-size ring buffers for efficient packet queueing
//! Typical usage: 256 buffers × 2KB = 512KB total

use core::cell::UnsafeCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferError {
    Full,
    Empty,
    PacketTooLarge,
}

/// A circular ring buffer for network packets
/// Generic over buffer count (N) and packet size (PACKET_SIZE)
pub struct PacketRingBuffer<const N: usize, const PACKET_SIZE: usize> {
    buffer: [UnsafeCell<PacketSlot>; N],
    head: usize,
    tail: usize,
    count: usize,
}

struct PacketSlot {
    data: [u8; 2048], // Max typical packet size
    len: usize,
}

impl Default for PacketSlot {
    fn default() -> Self {
        Self {
            data: [0; 2048],
            len: 0,
        }
    }
}

unsafe impl<const N: usize, const PACKET_SIZE: usize> Sync for PacketRingBuffer<N, PACKET_SIZE> {}
unsafe impl<const N: usize, const PACKET_SIZE: usize> Send for PacketRingBuffer<N, PACKET_SIZE> {}

impl<const N: usize, const PACKET_SIZE: usize> PacketRingBuffer<N, PACKET_SIZE> {
    pub const fn new() -> Self {
        const INIT: UnsafeCell<PacketSlot> = UnsafeCell::new(PacketSlot {
            data: [0; 2048],
            len: 0,
        });
        Self {
            buffer: [INIT; N],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    /// Push a packet into the ring buffer
    /// Returns an error if the buffer is full or packet is too large
    pub fn push(&mut self, packet: &[u8]) -> Result<(), BufferError> {
        if self.count == N {
            return Err(BufferError::Full);
        }
        if packet.len() > PACKET_SIZE {
            return Err(BufferError::PacketTooLarge);
        }
        
        let slot = unsafe { &mut *self.buffer[self.tail].get() };
        slot.data[..packet.len()].copy_from_slice(packet);
        slot.len = packet.len();
        
        self.tail = (self.tail + 1) % N;
        self.count += 1;
        Ok(())
    }

    /// Pop a packet from the ring buffer
    /// Returns the packet data and its length
    pub fn pop(&mut self) -> Result<([u8; PACKET_SIZE], usize), BufferError> {
        if self.count == 0 {
            return Err(BufferError::Empty);
        }
        
        let slot = unsafe { &*self.buffer[self.head].get() };
        let mut data = [0u8; PACKET_SIZE];
        data[..slot.len].copy_from_slice(&slot.data[..slot.len]);
        let len = slot.len;
        
        self.head = (self.head + 1) % N;
        self.count -= 1;
        
        Ok((data, len))
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    
    pub fn is_full(&self) -> bool {
        self.count == N
    }
    
    pub fn len(&self) -> usize {
        self.count
    }
    
    pub fn capacity(&self) -> usize {
        N
    }
}

/// Standard network packet ring buffer: 256 buffers × 2KB each
pub type StandardRxBuffer = PacketRingBuffer<256, 2048>;
pub type StandardTxBuffer = PacketRingBuffer<256, 2048>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ring_buffer_basic() {
        let mut ring: PacketRingBuffer<4, 2048> = PacketRingBuffer::new();
        
        assert!(ring.is_empty());
        assert_eq!(ring.len(), 0);
        
        let packet = [0x01, 0x02, 0x03, 0x04];
        ring.push(&packet).unwrap();
        
        assert!(!ring.is_empty());
        assert_eq!(ring.len(), 1);
        
        let (data, len) = ring.pop().unwrap();
        assert_eq!(len, 4);
        assert_eq!(&data[..len], &packet);
        
        assert!(ring.is_empty());
    }
}
