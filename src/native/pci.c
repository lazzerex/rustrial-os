/**
 * PCI Device Enumeration - C Implementation
 * 
 * This module provides PCI configuration space access and device enumeration
 * using I/O port operations. Works on bare metal x86-64.
 */

#include "pci.h"
#include <stdint.h>
#include <stdbool.h>

// PCI configuration space access ports
#define PCI_CONFIG_ADDRESS  0xCF8
#define PCI_CONFIG_DATA     0xCFC

// I/O port access functions (will be provided by inline assembly or Rust)
static inline void outl(uint16_t port, uint32_t value) {
    __asm__ volatile("outl %0, %1" : : "a"(value), "Nd"(port));
}

static inline uint32_t inl(uint16_t port) {
    uint32_t value;
    __asm__ volatile("inl %1, %0" : "=a"(value) : "Nd"(port));
    return value;
}

/**
 * Build PCI configuration address
 */
static uint32_t pci_config_address(uint8_t bus, uint8_t device, 
                                    uint8_t function, uint8_t offset) {
    return (1U << 31) |                     // Enable bit
           ((uint32_t)bus << 16) |          // Bus number
           ((uint32_t)(device & 0x1F) << 11) | // Device number
           ((uint32_t)(function & 0x07) << 8) | // Function number
           ((uint32_t)(offset & 0xFC));     // Register offset (aligned)
}

/**
 * Read 32-bit value from PCI configuration space
 */
uint32_t pci_read_config32(uint8_t bus, uint8_t device, 
                            uint8_t function, uint8_t offset) {
    uint32_t address = pci_config_address(bus, device, function, offset);
    outl(PCI_CONFIG_ADDRESS, address);
    return inl(PCI_CONFIG_DATA);
}

/**
 * Read 16-bit value from PCI configuration space
 */
uint16_t pci_read_config16(uint8_t bus, uint8_t device, 
                            uint8_t function, uint8_t offset) {
    uint32_t value = pci_read_config32(bus, device, function, offset & 0xFC);
    return (uint16_t)((value >> ((offset & 2) * 8)) & 0xFFFF);
}

/**
 * Read 8-bit value from PCI configuration space
 */
uint8_t pci_read_config8(uint8_t bus, uint8_t device, 
                          uint8_t function, uint8_t offset) {
    uint32_t value = pci_read_config32(bus, device, function, offset & 0xFC);
    return (uint8_t)((value >> ((offset & 3) * 8)) & 0xFF);
}

/**
 * Write 32-bit value to PCI configuration space
 */
void pci_write_config32(uint8_t bus, uint8_t device, uint8_t function,
                        uint8_t offset, uint32_t value) {
    uint32_t address = pci_config_address(bus, device, function, offset);
    outl(PCI_CONFIG_ADDRESS, address);
    outl(PCI_CONFIG_DATA, value);
}

/**
 * Check if a PCI device exists
 */
bool pci_device_exists(uint8_t bus, uint8_t device, uint8_t function) {
    uint16_t vendor_id = pci_read_config16(bus, device, function, 0x00);
    return vendor_id != 0xFFFF;
}

/**
 * Read complete device information
 */
void pci_read_device_info(uint8_t bus, uint8_t device, uint8_t function,
                           pci_device_t* info) {
    if (!info) return;
    
    info->bus = bus;
    info->device = device;
    info->function = function;
    
    info->vendor_id = pci_read_config16(bus, device, function, 0x00);
    info->device_id = pci_read_config16(bus, device, function, 0x02);
    info->class_code = pci_read_config8(bus, device, function, 0x0B);
    info->subclass = pci_read_config8(bus, device, function, 0x0A);
    info->prog_if = pci_read_config8(bus, device, function, 0x09);
    info->revision = pci_read_config8(bus, device, function, 0x08);
    info->header_type = pci_read_config8(bus, device, function, 0x0E);
    info->interrupt_line = pci_read_config8(bus, device, function, 0x3C);
    info->interrupt_pin = pci_read_config8(bus, device, function, 0x3D);
    
    // Read BARs (Base Address Registers)
    for (int i = 0; i < 6; i++) {
        info->bar[i] = pci_read_config32(bus, device, function, 0x10 + (i * 4));
    }
}

/**
 * Enumerate all PCI devices
 * Returns number of devices found
 */
int pci_enumerate_devices(pci_device_t* devices, int max_devices) {
    int count = 0;
    
    // Scan all buses
    for (int bus = 0; bus < 256; bus++) {
        // Scan all devices on this bus
        for (int device = 0; device < 32; device++) {
            // Check function 0 first
            if (!pci_device_exists(bus, device, 0)) {
                continue;
            }
            
            if (count < max_devices && devices) {
                pci_read_device_info(bus, device, 0, &devices[count]);
            }
            count++;
            
            // Check if multi-function device
            uint8_t header_type = pci_read_config8(bus, device, 0, 0x0E);
            if (header_type & 0x80) {
                // Scan other functions
                for (int function = 1; function < 8; function++) {
                    if (pci_device_exists(bus, device, function)) {
                        if (count < max_devices && devices) {
                            pci_read_device_info(bus, device, function, &devices[count]);
                        }
                        count++;
                    }
                }
            }
        }
    }
    
    return count;
}

/**
 * Get human-readable class name
 */
const char* pci_get_class_name(uint8_t class_code) {
    switch (class_code) {
        case 0x00: return "Unclassified";
        case 0x01: return "Mass Storage Controller";
        case 0x02: return "Network Controller";
        case 0x03: return "Display Controller";
        case 0x04: return "Multimedia Controller";
        case 0x05: return "Memory Controller";
        case 0x06: return "Bridge Device";
        case 0x07: return "Simple Communication Controller";
        case 0x08: return "Base System Peripheral";
        case 0x09: return "Input Device Controller";
        case 0x0A: return "Docking Station";
        case 0x0B: return "Processor";
        case 0x0C: return "Serial Bus Controller";
        case 0x0D: return "Wireless Controller";
        case 0x0E: return "Intelligent Controller";
        case 0x0F: return "Satellite Communication Controller";
        case 0x10: return "Encryption Controller";
        case 0x11: return "Signal Processing Controller";
        default: return "Unknown";
    }
}

/**
 * Get vendor name for common vendors
 */
const char* pci_get_vendor_name(uint16_t vendor_id) {
    switch (vendor_id) {
        case 0x8086: return "Intel";
        case 0x1022: return "AMD";
        case 0x10DE: return "NVIDIA";
        case 0x1002: return "ATI/AMD";
        case 0x1234: return "QEMU";
        case 0x15AD: return "VMware";
        case 0x80EE: return "VirtualBox";
        case 0x1AF4: return "VirtIO";
        case 0x10EC: return "Realtek";
        default: return "Unknown";
    }
}
