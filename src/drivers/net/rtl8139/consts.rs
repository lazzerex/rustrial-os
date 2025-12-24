// RTL8139 Network Driver Constants Here

// PCI Identification
pub const VENDOR_ID: u16 = 0x10EC;
pub const DEVICE_ID: u16 = 0x8139;

// Buffer Sizes
pub const RX_BUFFER_SIZE: usize = 8192 + 16 + 1536; // 8KB + 16 bytes + 1.5KB for wrap
pub const TX_BUFFER_SIZE: usize = 2048; // 2KB per TX buffer
pub const TX_BUFFER_COUNT: usize = 4; // RTL8139 has 4 TX descriptors

// Packet Size Limits
pub const MAX_ETH_FRAME_SIZE: usize = 1518;
pub const MIN_ETH_FRAME_SIZE: usize = 60;

// Interrupt Mask Values
pub const IMR_ROK: u16 = 0x0001;  // Receive OK
pub const IMR_RER: u16 = 0x0002;  // Receive Error
pub const IMR_TOK: u16 = 0x0004;  // Transmit OK
pub const IMR_TER: u16 = 0x0008;  // Transmit Error
pub const IMR_RXOVW: u16 = 0x0010; // RX Buffer Overflow
pub const IMR_PUN: u16 = 0x0020;  // Packet Underrun
pub const IMR_FOVW: u16 = 0x0040; // RX FIFO Overflow
pub const IMR_LENCHG: u16 = 0x2000; // Cable Length Change
pub const IMR_TIMEOUT: u16 = 0x4000; // Timeout
pub const IMR_SERR: u16 = 0x8000; // System Error

// Interrupt Status Register (same bits as IMR)
pub const ISR_ROK: u16 = 0x0001;
pub const ISR_RER: u16 = 0x0002;
pub const ISR_TOK: u16 = 0x0004;
pub const ISR_TER: u16 = 0x0008;
pub const ISR_RXOVW: u16 = 0x0010;

// Command Register
pub const CMD_BUFE: u8 = 0x01;    // Buffer Empty
pub const CMD_TE: u8 = 0x04;      // Transmitter Enable
pub const CMD_RE: u8 = 0x08;      // Receiver Enable
pub const CMD_RST: u8 = 0x10;     // Reset

// Receiver Configuration
pub const RCR_AAP: u32 = 0x00000001;  // Accept All Packets
pub const RCR_APM: u32 = 0x00000002;  // Accept Physical Match
pub const RCR_AM: u32 = 0x00000004;   // Accept Multicast
pub const RCR_AB: u32 = 0x00000008;   // Accept Broadcast
pub const RCR_AR: u32 = 0x00000010;   // Accept Runt
pub const RCR_AER: u32 = 0x00000020;  // Accept Error
pub const RCR_WRAP: u32 = 0x00000080; // RX Buffer Wrap

// FIFO threshold: 1024 bytes (bits 13-15 = 110)
pub const RCR_RXFTH_1024: u32 = 0x0000C000;

// Max DMA Burst Size: unlimited (bits 8-10 = 111)
pub const RCR_RBLEN_8K: u32 = 0x00000000; // 8K + 16 bytes
pub const RCR_RBLEN_16K: u32 = 0x00000800;
pub const RCR_RBLEN_32K: u32 = 0x00001000;
pub const RCR_RBLEN_64K: u32 = 0x00001800;

pub const RCR_MXDMA_UNLIMITED: u32 = 0x00000700;

// Transmitter Configuration
pub const TCR_CLRABT: u32 = 0x00000001; // Clear Abort
pub const TCR_TXRR_ZERO: u32 = 0x00000000; // No retransmission
pub const TCR_MXDMA_2048: u32 = 0x00000700; // Max DMA burst = 2048 bytes
pub const TCR_IFG_STANDARD: u32 = 0x03000000; // Standard interframe gap

// Transmit Status Register bits
pub const TSD_OWN: u32 = 0x00002000;  // Ownership (0 = driver, 1 = chip)
pub const TSD_TUN: u32 = 0x00004000;  // Transmit FIFO Underrun
pub const TSD_TOK: u32 = 0x00008000;  // Transmit OK
pub const TSD_SIZE_MASK: u32 = 0x00001FFF; // Size mask (13 bits)

// Receive Packet Header
pub const RX_ROK: u16 = 0x0001;  // Receive OK
pub const RX_FAE: u16 = 0x0002;  // Frame Alignment Error
pub const RX_CRC: u16 = 0x0004;  // CRC Error
pub const RX_LONG: u16 = 0x0008; // Long Packet
pub const RX_RUNT: u16 = 0x0010; // Runt Packet
pub const RX_ISE: u16 = 0x0020;  // Invalid Symbol Error
pub const RX_BAR: u16 = 0x2000;  // Broadcast Address Received
pub const RX_PAM: u16 = 0x4000;  // Physical Address Matched
pub const RX_MAR: u16 = 0x8000;  // Multicast Address Received
