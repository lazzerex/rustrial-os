/// PS/2 Mouse driver for Rustrial OS
/// 
/// This module handles PS/2 mouse input including packet decoding,
/// cursor position tracking, and button state management.

use conquer_once::spin::OnceCell;
use core::sync::atomic::{AtomicI16, AtomicU8, Ordering};
use crossbeam_queue::ArrayQueue;

/// Global mouse packet queue
static MOUSE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

/// Mouse cursor position (in screen coordinates)
pub static MOUSE_X: AtomicI16 = AtomicI16::new(40); // Center of 80-column screen
pub static MOUSE_Y: AtomicI16 = AtomicI16::new(12); // Center of 25-row screen

/// Mouse button states
pub static MOUSE_LEFT_BUTTON: AtomicU8 = AtomicU8::new(0);
pub static MOUSE_RIGHT_BUTTON: AtomicU8 = AtomicU8::new(0);

/// Screen bounds
const SCREEN_WIDTH: i16 = 80;
const SCREEN_HEIGHT: i16 = 25;

/// Mouse packet structure
#[derive(Debug, Clone, Copy)]
pub struct MousePacket {
    pub buttons: u8,
    pub x_movement: i16,
    pub y_movement: i16,
}

impl MousePacket {
    fn from_bytes(bytes: [u8; 3]) -> Option<Self> {
        // Check if the packet is valid (bit 3 of first byte should be 1)
        if bytes[0] & 0x08 == 0 {
            return None;
        }

        let buttons = bytes[0] & 0x07; // Lower 3 bits are button states
        
        // Sign-extend the movement values
        let x_movement = if bytes[0] & 0x10 != 0 {
            // Negative X
            (bytes[1] as i16) | (0xFF00u16 as i16)
        } else {
            bytes[1] as i16
        };
        
        let y_movement = if bytes[0] & 0x20 != 0 {
            // Negative Y
            (bytes[2] as i16) | (0xFF00u16 as i16)
        } else {
            bytes[2] as i16
        };

        Some(MousePacket {
            buttons,
            x_movement,
            y_movement,
        })
    }

    pub fn left_button(&self) -> bool {
        self.buttons & 0x01 != 0
    }

    pub fn right_button(&self) -> bool {
        self.buttons & 0x02 != 0
    }

    pub fn middle_button(&self) -> bool {
        self.buttons & 0x04 != 0
    }
}

/// Initialize the mouse packet queue
pub fn init() {
    MOUSE_QUEUE.try_init_once(|| ArrayQueue::new(100))
        .expect("MouseQueue::init should only be called once");
}

/// Add a byte from the mouse interrupt
pub fn add_byte(byte: u8) {
    if let Ok(queue) = MOUSE_QUEUE.try_get() {
        if let Err(_) = queue.push(byte) {
            // Queue is full, ignore this byte
        }
    }
}

/// Mouse packet stream for async processing
pub struct MouseStream {
    packet_buffer: [u8; 3],
    buffer_index: usize,
}

impl MouseStream {
    pub fn new() -> Self {
        MouseStream {
            packet_buffer: [0; 3],
            buffer_index: 0,
        }
    }

    /// Try to get the next complete mouse packet
    pub fn try_next(&mut self) -> Option<MousePacket> {
        if let Ok(queue) = MOUSE_QUEUE.try_get() {
            while let Some(byte) = queue.pop() {
                self.packet_buffer[self.buffer_index] = byte;
                self.buffer_index += 1;

                if self.buffer_index == 3 {
                    self.buffer_index = 0;
                    if let Some(packet) = MousePacket::from_bytes(self.packet_buffer) {
                        return Some(packet);
                    }
                }
            }
        }
        None
    }
}

/// Update mouse position based on movement
pub fn update_position(dx: i16, dy: i16) {
    let mut x = MOUSE_X.load(Ordering::Relaxed);
    let mut y = MOUSE_Y.load(Ordering::Relaxed);

    // Update X coordinate
    x += dx / 2; // Scale down movement for better control
    if x < 0 {
        x = 0;
    } else if x >= SCREEN_WIDTH {
        x = SCREEN_WIDTH - 1;
    }

    // Update Y coordinate (invert Y because PS/2 Y is opposite)
    y -= dy / 2; // Scale down and invert
    if y < 0 {
        y = 0;
    } else if y >= SCREEN_HEIGHT {
        y = SCREEN_HEIGHT - 1;
    }

    MOUSE_X.store(x, Ordering::Relaxed);
    MOUSE_Y.store(y, Ordering::Relaxed);
}

/// Get current mouse position
pub fn get_position() -> (i16, i16) {
    (
        MOUSE_X.load(Ordering::Relaxed),
        MOUSE_Y.load(Ordering::Relaxed),
    )
}

/// Update button states
pub fn update_buttons(buttons: u8) {
    MOUSE_LEFT_BUTTON.store(buttons & 0x01, Ordering::Relaxed);
    MOUSE_RIGHT_BUTTON.store((buttons & 0x02) >> 1, Ordering::Relaxed);
}

/// Check if left button is pressed
pub fn is_left_button_pressed() -> bool {
    MOUSE_LEFT_BUTTON.load(Ordering::Relaxed) != 0
}

/// Check if right button is pressed
pub fn is_right_button_pressed() -> bool {
    MOUSE_RIGHT_BUTTON.load(Ordering::Relaxed) != 0
}

/// PS/2 Mouse controller ports
const MOUSE_DATA_PORT: u16 = 0x60;
const MOUSE_COMMAND_PORT: u16 = 0x64;

/// Initialize PS/2 mouse hardware
pub fn init_hardware() {
    use x86_64::instructions::port::Port;
    
    unsafe {
        let mut cmd_port = Port::new(MOUSE_COMMAND_PORT);
        let mut data_port = Port::new(MOUSE_DATA_PORT);
        
        // Enable auxiliary device (mouse)
        cmd_port.write(0xA8u8);
        
        // Tell the controller we want to send a command to the mouse
        cmd_port.write(0xD4u8);
        
        // Enable data reporting
        data_port.write(0xF4u8);
        
        // Read acknowledgment
        let _ack: u8 = data_port.read();
    }
}
