/**
 * PCI Device Enumeration Header
 */

#ifndef PCI_H
#define PCI_H

#include <stdint.h>
#include <stdbool.h>

/**
 * PCI Device Information Structure
 */
typedef struct {
    uint8_t bus;
    uint8_t device;
    uint8_t function;
    uint16_t vendor_id;
    uint16_t device_id;
    uint8_t class_code;
    uint8_t subclass;
    uint8_t prog_if;
    uint8_t revision;
    uint8_t header_type;
    uint8_t interrupt_line;
    uint8_t interrupt_pin;
    uint32_t bar[6];  // Base Address Registers
} pci_device_t;

// PCI Configuration Space Offsets
#define PCI_COMMAND         0x04
#define PCI_STATUS          0x06
#define PCI_BAR0            0x10
#define PCI_INTERRUPT_LINE  0x3C
#define PCI_INTERRUPT_PIN   0x3D

// PCI Command Register Bits
#define PCI_COMMAND_IO          0x0001  // Enable I/O space
#define PCI_COMMAND_MEMORY      0x0002  // Enable memory space
#define PCI_COMMAND_BUS_MASTER  0x0004  // Enable bus mastering (DMA)
#define PCI_COMMAND_INTERRUPT   0x0400  // Interrupt disable

// Function prototypes
uint32_t pci_read_config32(uint8_t bus, uint8_t device, uint8_t function, uint8_t offset);
uint16_t pci_read_config16(uint8_t bus, uint8_t device, uint8_t function, uint8_t offset);
uint8_t pci_read_config8(uint8_t bus, uint8_t device, uint8_t function, uint8_t offset);
void pci_write_config32(uint8_t bus, uint8_t device, uint8_t function, uint8_t offset, uint32_t value);
void pci_write_config16(uint8_t bus, uint8_t device, uint8_t function, uint8_t offset, uint16_t value);
bool pci_device_exists(uint8_t bus, uint8_t device, uint8_t function);
void pci_read_device_info(uint8_t bus, uint8_t device, uint8_t function, pci_device_t* info);
int pci_enumerate_devices(pci_device_t* devices, int max_devices);
const char* pci_get_class_name(uint8_t class_code);
const char* pci_get_vendor_name(uint16_t vendor_id);

// Stage 1.2: Enhanced PCI functions
void pci_enable_bus_mastering(uint8_t bus, uint8_t device, uint8_t function);
void pci_enable_memory_space(uint8_t bus, uint8_t device, uint8_t function);
void pci_enable_io_space(uint8_t bus, uint8_t device, uint8_t function);
uint32_t pci_get_bar_size(uint8_t bus, uint8_t device, uint8_t function, uint8_t bar_index);

#endif // PCI_H
