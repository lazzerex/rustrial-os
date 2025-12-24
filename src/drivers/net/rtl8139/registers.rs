// RTL8139 Register Offsets

// MAC Address Registers (6 bytes)
pub const IDR0: u8 = 0x00; // ID Register 0 (MAC byte 0)
pub const IDR1: u8 = 0x01; // ID Register 1 (MAC byte 1)
pub const IDR2: u8 = 0x02; // ID Register 2 (MAC byte 2)
pub const IDR3: u8 = 0x03; // ID Register 3 (MAC byte 3)
pub const IDR4: u8 = 0x04; // ID Register 4 (MAC byte 4)
pub const IDR5: u8 = 0x05; // ID Register 5 (MAC byte 5)

// Multicast Registers
pub const MAR0: u8 = 0x08; // Multicast Address Register 0
pub const MAR4: u8 = 0x0C; // Multicast Address Register 4

// Transmit Status Registers (4 registers, one per descriptor)
pub const TSD0: u8 = 0x10; // Transmit Status of Descriptor 0
pub const TSD1: u8 = 0x14; // Transmit Status of Descriptor 1
pub const TSD2: u8 = 0x18; // Transmit Status of Descriptor 2
pub const TSD3: u8 = 0x1C; // Transmit Status of Descriptor 3

// Transmit Start Address Registers (4 registers)
pub const TSAD0: u8 = 0x20; // Transmit Start Address of Descriptor 0
pub const TSAD1: u8 = 0x24; // Transmit Start Address of Descriptor 1
pub const TSAD2: u8 = 0x28; // Transmit Start Address of Descriptor 2
pub const TSAD3: u8 = 0x2C; // Transmit Start Address of Descriptor 3

// Receive Buffer Start Address
pub const RBSTART: u8 = 0x30; // Receive Buffer Start Address (32-bit)

// Early Receive Byte Count
pub const ERBCR: u8 = 0x34; // Early RX Byte Count Register

// Early Receive Status Register
pub const ERSR: u8 = 0x36; // Early RX Status Register

// Command Register
pub const CR: u8 = 0x37; // Command Register (8-bit)

// Current Address of Packet Read (CAPR)
pub const CAPR: u8 = 0x38; // Current Address of Packet Read (16-bit)

// Current Buffer Address (CBA)
pub const CBR: u8 = 0x3A; // Current Buffer Address (16-bit)

// Interrupt Mask Register
pub const IMR: u8 = 0x3C; // Interrupt Mask Register (16-bit)

// Interrupt Status Register
pub const ISR: u8 = 0x3E; // Interrupt Status Register (16-bit)

// Transmit Configuration Register
pub const TCR: u8 = 0x40; // Transmit Configuration Register (32-bit)

// Receive Configuration Register
pub const RCR: u8 = 0x44; // Receive Configuration Register (32-bit)

// Timer Count Register
pub const TCTR: u8 = 0x48; // Timer Count Register (32-bit)

// Missed Packet Counter
pub const MPC: u8 = 0x4C; // Missed Packet Counter (32-bit)

// 93C46 Command Register
pub const CR9346: u8 = 0x50; // 93C46 Command Register (8-bit)

// Configuration Registers
pub const CONFIG0: u8 = 0x51; // Configuration Register 0
pub const CONFIG1: u8 = 0x52; // Configuration Register 1
pub const CONFIG2: u8 = 0x53; // Configuration Register 2
pub const CONFIG3: u8 = 0x54; // Configuration Register 3
pub const CONFIG4: u8 = 0x55; // Configuration Register 4
pub const CONFIG5: u8 = 0x56; // Configuration Register 5

// Media Status Register
pub const MSR: u8 = 0x58; // Media Status Register (8-bit)

// Basic Mode Control Register (MII)
pub const BMCR: u8 = 0x62; // Basic Mode Control Register (16-bit)

// Basic Mode Status Register (MII)
pub const BMSR: u8 = 0x64; // Basic Mode Status Register (16-bit)

// Helper function to get TSD register offset by index
pub const fn tsd(index: usize) -> u8 {
    match index {
        0 => TSD0,
        1 => TSD1,
        2 => TSD2,
        3 => TSD3,
        _ => TSD0, // Default to 0 if invalid
    }
}

// Helper function to get TSAD register offset by index
pub const fn tsad(index: usize) -> u8 {
    match index {
        0 => TSAD0,
        1 => TSAD1,
        2 => TSAD2,
        3 => TSAD3,
        _ => TSAD0, // Default to 0 if invalid
    }
}
