// RTL8139 Network Driver Implementation

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;

use crate::memory::dma::{allocate_dma_buffer, DmaBuffer};
use crate::native_ffi::{pci_scan, PciDevice, pci_enable_dma, pci_enable_mmio, pci_get_bar, pci_get_interrupt_line};
use crate::memory::{MAPPER, FRAME_ALLOCATOR, PHYS_MEM_OFFSET};
use super::{NetworkDevice, TransmitError, LinkStatus};

mod consts;
mod registers;

use consts::*;
use registers::*;

/// RTL8139 Network Card Driver
pub struct Rtl8139 {
    /// Virtual address of MMIO base
    mmio_base: VirtAddr,
    /// Physical address of MMIO base
    mmio_phys: PhysAddr,
    /// MAC address of the device
    mac_addr: [u8; 6],
    /// Receive buffer
    rx_buffer: Option<DmaBuffer>,
    /// Transmit buffers (4 descriptors)
    tx_buffers: [Option<DmaBuffer>; 4],
    /// Current TX descriptor index (round-robin)
    current_tx: AtomicU8,
    /// IRQ number
    irq: u8,
    /// Device initialized flag
    initialized: AtomicBool,
    /// PCI device info
    pci_device: PciDevice,
    /// Current RX buffer read offset
    rx_offset: Mutex<u16>,
}

impl Rtl8139 {
    /// Detect and initialize RTL8139 device
    /// 
    /// we would return:
    /// * `Some(Rtl8139)` if device was found and initialized successfully
    /// * `None` if device was not found or initialization failed
    pub fn new() -> Option<Self> {
        serial_println!("[RTL8139] Scanning for RTL8139 device...");
        
        // Scan PCI bus for RTL8139
        let devices = pci_scan();
        let rtl8139_device = devices.iter().find(|dev| {
            dev.vendor_id == VENDOR_ID && dev.device_id == DEVICE_ID
        })?;

        serial_println!("[RTL8139] Found RTL8139 at bus {}, device {}, function {}",
            rtl8139_device.bus, rtl8139_device.device, rtl8139_device.function);

        // Enable PCI bus mastering and MMIO
        pci_enable_dma(rtl8139_device);
        pci_enable_mmio(rtl8139_device);

        // Get BAR0 (MMIO base address)
        let bar = pci_get_bar(rtl8139_device, 0)?;
        if !bar.is_mmio {
            serial_println!("[RTL8139] BAR0 is not MMIO!");
            return None;
        }

        serial_println!("[RTL8139] MMIO base: {:#x}, size: {} bytes", 
            bar.base_addr.as_u64(), bar.size);

        // Map MMIO region to virtual memory
        let mmio_phys = bar.base_addr;
        let mmio_virt = Self::map_mmio(mmio_phys, bar.size)?;

        serial_println!("[RTL8139] Mapped MMIO to virtual address: {:#x}", mmio_virt.as_u64());

        // Get IRQ line
        let irq = pci_get_interrupt_line(rtl8139_device);
        serial_println!("[RTL8139] IRQ line: {}", irq);

        let mut driver = Self {
            mmio_base: mmio_virt,
            mmio_phys,
            mac_addr: [0; 6],
            rx_buffer: None,
            tx_buffers: [None, None, None, None],
            current_tx: AtomicU8::new(0),
            irq,
            initialized: AtomicBool::new(false),
            pci_device: *rtl8139_device,
            rx_offset: Mutex::new(0),
        };

        // Initialize the device
        if driver.initialize().is_err() {
            serial_println!("[RTL8139] Failed to initialize device");
            return None;
        }

        serial_println!("[RTL8139] Device initialized successfully");
        serial_println!("[RTL8139] MAC Address: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            driver.mac_addr[0], driver.mac_addr[1], driver.mac_addr[2],
            driver.mac_addr[3], driver.mac_addr[4], driver.mac_addr[5]);

        Some(driver)
    }

    /// Map MMIO region to virtual memory
    fn map_mmio(phys_addr: PhysAddr, size: usize) -> Option<VirtAddr> {
        let mut mapper = MAPPER.lock();
        let mut frame_allocator = FRAME_ALLOCATOR.lock();

        // Calculate number of pages needed
        let page_count = (size + 0xFFF) / 0x1000;
        
        // Use a high virtual address for MMIO mapping
        let virt_start = VirtAddr::new(0xFFFF_8000_0000_0000 + phys_addr.as_u64());

        for i in 0..page_count {
            let page = Page::<Size4KiB>::containing_address(virt_start + (i * 0x1000) as u64);
            let frame = PhysFrame::containing_address(phys_addr + (i * 0x1000) as u64);
            let flags = PageTableFlags::PRESENT 
                | PageTableFlags::WRITABLE 
                | PageTableFlags::NO_CACHE;

            unsafe {
                if mapper.map_to(page, frame, flags, &mut *frame_allocator).is_err() {
                    serial_println!("[RTL8139] Failed to map MMIO page {}", i);
                    return None;
                }
            }
        }

        Some(virt_start)
    }

    /// Initialize the RTL8139 device
    fn initialize(&mut self) -> Result<(), &'static str> {
        serial_println!("[RTL8139] Starting device initialization...");

        // Step 1: Software reset
        self.reset()?;

        // Step 2: Read MAC address
        self.read_mac_address();

        // Step 3: Allocate and setup RX buffer
        self.setup_rx_buffer()?;

        // Step 4: Allocate and setup TX buffers
        self.setup_tx_buffers()?;

        // Step 5: Configure receiver
        self.configure_receiver();

        // Step 6: Configure transmitter
        self.configure_transmitter();

        // Step 7: Enable interrupts
        self.enable_interrupts();

        // Step 8: Enable transmitter and receiver
        self.enable_tx_rx();

        self.initialized.store(true, Ordering::SeqCst);
        serial_println!("[RTL8139] Initialization complete!");

        Ok(())
    }

    /// Perform software reset
    fn reset(&mut self) -> Result<(), &'static str> {
        serial_println!("[RTL8139] Performing software reset...");

        // Send reset command
        self.write_reg_u8(CR, CMD_RST);

        // Wait for reset to complete (RST bit clears when done)
        let mut timeout = 1000;
        while (self.read_reg_u8(CR) & CMD_RST) != 0 {
            if timeout == 0 {
                return Err("Reset timeout");
            }
            timeout -= 1;
            // Small delay
            for _ in 0..1000 {
                unsafe { core::arch::asm!("nop"); }
            }
        }

        serial_println!("[RTL8139] Reset complete");
        Ok(())
    }

    /// Read MAC address from device registers
    fn read_mac_address(&mut self) {
        for i in 0..6 {
            self.mac_addr[i] = self.read_reg_u8(IDR0 + i);
        }
    }

    /// Setup receive buffer
    fn setup_rx_buffer(&mut self) -> Result<(), &'static str> {
        serial_println!("[RTL8139] Setting up RX buffer...");

        // Allocate DMA buffer for reception
        let rx_buffer = allocate_dma_buffer(RX_BUFFER_SIZE)
            .map_err(|_| "Failed to allocate RX buffer")?;

        serial_println!("[RTL8139] RX buffer allocated: virt={:#x}, phys={:#x}, size={}",
            rx_buffer.virt_addr.as_u64(), rx_buffer.phys_addr.as_u64(), rx_buffer.size);

        // Write physical address to RBSTART register
        self.write_reg_u32(RBSTART, rx_buffer.phys_addr.as_u64() as u32);

        // Initialize RX offset to 0
        *self.rx_offset.lock() = 0;

        self.rx_buffer = Some(rx_buffer);
        serial_println!("[RTL8139] RX buffer setup complete");

        Ok(())
    }

    /// Setup transmit buffers
    fn setup_tx_buffers(&mut self) -> Result<(), &'static str> {
        serial_println!("[RTL8139] Setting up TX buffers...");

        for i in 0..TX_BUFFER_COUNT {
            let tx_buffer = allocate_dma_buffer(TX_BUFFER_SIZE)
                .map_err(|_| "Failed to allocate TX buffer")?;

            serial_println!("[RTL8139] TX buffer {} allocated: virt={:#x}, phys={:#x}",
                i, tx_buffer.virt_addr.as_u64(), tx_buffer.phys_addr.as_u64());

            self.tx_buffers[i] = Some(tx_buffer);
        }

        serial_println!("[RTL8139] TX buffers setup complete");
        Ok(())
    }

    /// Configure receiver
    fn configure_receiver(&mut self) {
        serial_println!("[RTL8139] Configuring receiver...");

        // Accept broadcast, multicast, and packets matching our MAC
        // Use 8KB buffer, unlimited DMA, 1024-byte FIFO threshold
        let rcr = RCR_AB | RCR_AM | RCR_APM 
            | RCR_RBLEN_8K 
            | RCR_MXDMA_UNLIMITED 
            | RCR_RXFTH_1024
            | RCR_WRAP;

        self.write_reg_u32(RCR, rcr);
        serial_println!("[RTL8139] Receiver configured");
    }

    /// Configure transmitter
    fn configure_transmitter(&mut self) {
        serial_println!("[RTL8139] Configuring transmitter...");

        // Standard configuration: 2048-byte max DMA, standard interframe gap
        let tcr = TCR_MXDMA_2048 | TCR_IFG_STANDARD;

        self.write_reg_u32(TCR, tcr);
        serial_println!("[RTL8139] Transmitter configured");
    }

    /// Enable interrupts
    fn enable_interrupts(&mut self) {
        serial_println!("[RTL8139] Enabling interrupts...");

        // Enable RX/TX interrupts
        let imr = IMR_ROK | IMR_TOK | IMR_RER | IMR_TER | IMR_RXOVW;
        self.write_reg_u16(IMR, imr);

        serial_println!("[RTL8139] Interrupts enabled");
    }

    /// Enable transmitter and receiver
    fn enable_tx_rx(&mut self) {
        serial_println!("[RTL8139] Enabling transmitter and receiver...");

        let cmd = CMD_TE | CMD_RE;
        self.write_reg_u8(CR, cmd);

        serial_println!("[RTL8139] Transmitter and receiver enabled");
    }

    /// Handle interrupt (called by interrupt handler)
    pub fn handle_interrupt(&mut self) {
        let isr = self.read_reg_u16(ISR);
        
        // Clear interrupts by writing back
        self.write_reg_u16(ISR, isr);

        if isr & ISR_ROK != 0 {
            // Packet received
            serial_println!("[RTL8139] RX interrupt");
        }

        if isr & ISR_TOK != 0 {
            // Packet transmitted
            serial_println!("[RTL8139] TX interrupt");
        }

        if isr & ISR_RER != 0 {
            serial_println!("[RTL8139] RX error interrupt");
        }

        if isr & ISR_TER != 0 {
            serial_println!("[RTL8139] TX error interrupt");
        }

        if isr & ISR_RXOVW != 0 {
            serial_println!("[RTL8139] RX buffer overflow interrupt");
        }
    }

    // MMIO register access helpers
    fn read_reg_u8(&self, offset: u8) -> u8 {
        unsafe {
            read_volatile((self.mmio_base.as_u64() + offset as u64) as *const u8)
        }
    }

    fn write_reg_u8(&mut self, offset: u8, value: u8) {
        unsafe {
            write_volatile((self.mmio_base.as_u64() + offset as u64) as *mut u8, value);
        }
    }

    fn read_reg_u16(&self, offset: u8) -> u16 {
        unsafe {
            read_volatile((self.mmio_base.as_u64() + offset as u64) as *const u16)
        }
    }

    fn write_reg_u16(&mut self, offset: u8, value: u16) {
        unsafe {
            write_volatile((self.mmio_base.as_u64() + offset as u64) as *mut u16, value);
        }
    }

    fn read_reg_u32(&self, offset: u8) -> u32 {
        unsafe {
            read_volatile((self.mmio_base.as_u64() + offset as u64) as *const u32)
        }
    }

    fn write_reg_u32(&mut self, offset: u8, value: u32) {
        unsafe {
            write_volatile((self.mmio_base.as_u64() + offset as u64) as *mut u32, value);
        }
    }
}

impl NetworkDevice for Rtl8139 {
    fn mac_address(&self) -> [u8; 6] {
        self.mac_addr
    }

    fn transmit(&mut self, packet: &[u8]) -> Result<(), TransmitError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TransmitError::NotInitialized);
        }

        if packet.len() > MAX_ETH_FRAME_SIZE {
            return Err(TransmitError::PacketTooLarge);
        }

        // Get current TX descriptor (round-robin)
        let tx_index = self.current_tx.fetch_add(1, Ordering::SeqCst) as usize % TX_BUFFER_COUNT;

        // Check if descriptor is available
        let tsd = self.read_reg_u32(tsd(tx_index));
        if (tsd & TSD_OWN) == 0 && (tsd & TSD_TOK) == 0 {
            // Descriptor is busy, buffer full
            return Err(TransmitError::BufferFull);
        }

        // Copy packet to TX buffer
        let tx_buffer = self.tx_buffers[tx_index].as_ref()
            .ok_or(TransmitError::NotInitialized)?;

        unsafe {
            core::ptr::copy_nonoverlapping(
                packet.as_ptr(),
                tx_buffer.virt_addr.as_u64() as *mut u8,
                packet.len()
            );
        }

        // Write physical address to TSAD
        self.write_reg_u32(tsad(tx_index), tx_buffer.phys_addr.as_u64() as u32);

        // Write packet length to TSD (this triggers transmission)
        self.write_reg_u32(tsd(tx_index), packet.len() as u32);

        serial_println!("[RTL8139] Transmitted packet: {} bytes (descriptor {})", packet.len(), tx_index);

        Ok(())
    }

    fn receive(&mut self) -> Option<Vec<u8>> {
        if !self.initialized.load(Ordering::SeqCst) {
            return None;
        }

        let rx_buffer = self.rx_buffer.as_ref()?;
        let mut rx_offset = self.rx_offset.lock();

        // Read command register to check if buffer is empty
        let cmd = self.read_reg_u8(CR);
        if (cmd & CMD_BUFE) != 0 {
            // Buffer is empty
            return None;
        }

        // Read packet header
        let header_addr = rx_buffer.virt_addr.as_u64() + *rx_offset as u64;
        let header = unsafe {
            core::ptr::read_volatile(header_addr as *const u32)
        };

        let status = (header & 0xFFFF) as u16;
        let length = ((header >> 16) & 0xFFFF) as u16;

        // Check for errors
        if (status & RX_ROK) == 0 {
            serial_println!("[RTL8139] RX error: status={:#x}", status);
            // Skip this packet
            *rx_offset = (*rx_offset + length + 4 + 3) & !3; // Align to 4 bytes
            self.write_reg_u16(CAPR, *rx_offset - 16);
            return None;
        }

        // Allocate buffer for packet (excluding CRC)
        let packet_len = (length - 4) as usize; // Remove 4-byte CRC
        let mut packet = Vec::with_capacity(packet_len);

        // Copy packet data
        let data_addr = header_addr + 4;
        unsafe {
            core::ptr::copy_nonoverlapping(
                data_addr as *const u8,
                packet.as_mut_ptr(),
                packet_len
            );
            packet.set_len(packet_len);
        }

        // Update read offset (align to 4 bytes)
        *rx_offset = (*rx_offset + length + 4 + 3) & !3;
        
        // Update CAPR register (need to subtract 16 as per RTL8139 quirk)
        self.write_reg_u16(CAPR, *rx_offset - 16);

        serial_println!("[RTL8139] Received packet: {} bytes", packet_len);

        Some(packet)
    }

    fn link_status(&self) -> LinkStatus {
        // Read media status register
        let msr = self.read_reg_u8(MSR);
        
        // Bit 2 is LINKB (Link Status, 0 = link OK, 1 = no link)
        if (msr & 0x04) == 0 {
            LinkStatus::Up
        } else {
            LinkStatus::Down
        }
    }

    fn device_name(&self) -> &str {
        "RTL8139"
    }

    fn is_ready(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }
}

// Global network device instance
use lazy_static::lazy_static;

lazy_static! {
    pub static ref NETWORK_DEVICE: Mutex<Option<Box<dyn NetworkDevice>>> = Mutex::new(None);
}

/// Initialize network device
pub fn init_network() -> Result<(), &'static str> {
    serial_println!("[Network] Initializing network device...");

    match Rtl8139::new() {
        Some(device) => {
            let boxed: Box<dyn NetworkDevice> = Box::new(device);
            *NETWORK_DEVICE.lock() = Some(boxed);
            serial_println!("[Network] Network device initialized successfully");
            Ok(())
        }
        None => {
            serial_println!("[Network] No supported network device found");
            Err("No network device found")
        }
    }
}

/// Get reference to network device
pub fn get_network_device() -> Option<&'static Mutex<Option<Box<dyn NetworkDevice>>>> {
    Some(&NETWORK_DEVICE)
}
