//! Circular ring buffer for RX/TX network packets
//! Phase 1.1 - Networking Roadmap

use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

pub struct PacketRingBuffer<const N: usize, const PACKET_SIZE: usize> {
    buffer: [UnsafeCell<[u8; PACKET_SIZE]>; N],
    head: usize,
    tail: usize,
    count: usize,
}

unsafe impl<const N: usize, const PACKET_SIZE: usize> Sync for PacketRingBuffer<N, PACKET_SIZE> {}

impl<const N: usize, const PACKET_SIZE: usize> PacketRingBuffer<N, PACKET_SIZE> {
    pub const fn new() -> Self {
        const ZERO: UnsafeCell<[u8; PACKET_SIZE]> = UnsafeCell::new([0; PACKET_SIZE]);
        Self {
            buffer: [ZERO; N],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, packet: &[u8]) -> Result<(), ()> {
        if self.count == N {
            return Err(()); // Buffer full
        }
        let slot = unsafe { &mut *self.buffer[self.tail].get() };
        let len = packet.len().min(PACKET_SIZE);
        slot[..len].copy_from_slice(&packet[..len]);
        self.tail = (self.tail + 1) % N;
        self.count += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<&[u8; PACKET_SIZE]> {
        if self.count == 0 {
            return None;
        }
        let slot = unsafe { &*self.buffer[self.head].get() };
        self.head = (self.head + 1) % N;
        self.count -= 1;
        Some(slot)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn is_full(&self) -> bool {
        self.count == N
    }
}

// Example usage:
// let mut rx_ring: PacketRingBuffer<256, 2048> = PacketRingBuffer::new();
