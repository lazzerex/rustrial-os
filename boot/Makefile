# ============================================================================
# RustrialOS Custom Bootloader Build Script - LINUX/macOS (Make)
# ============================================================================
# This Makefile is designed for Linux and macOS systems.
# For Windows, use build.ps1 instead.
# 
# Requires: nasm, dd

.PHONY: all clean

# Assembly source files
BOOT_SRC = boot.asm
STAGE2_SRC = stage2.asm

# Output binary files
BOOT_BIN = boot.bin
STAGE2_BIN = stage2.bin
BOOTLOADER_IMG = bootloader.img

# Assembler
ASM = nasm
ASMFLAGS = -f bin

all: $(BOOTLOADER_IMG)

# Assemble boot sector (stage 1)
$(BOOT_BIN): $(BOOT_SRC)
	@echo "Assembling Stage 1 bootloader..."
	$(ASM) $(ASMFLAGS) $(BOOT_SRC) -o $(BOOT_BIN)
	@echo "Stage 1 size: $$(stat -c%s $(BOOT_BIN) 2>/dev/null || stat -f%z $(BOOT_BIN)) bytes (must be 512)"

# Assemble stage 2
$(STAGE2_BIN): $(STAGE2_SRC)
	@echo "Assembling Stage 2 bootloader..."
	$(ASM) $(ASMFLAGS) $(STAGE2_SRC) -o $(STAGE2_BIN)
	@echo "Stage 2 size: $$(stat -c%s $(STAGE2_BIN) 2>/dev/null || stat -f%z $(STAGE2_BIN)) bytes"

# Create bootloader image (boot sector + stage 2)
$(BOOTLOADER_IMG): $(BOOT_BIN) $(STAGE2_BIN)
	@echo "Creating bootloader image..."
	cat $(BOOT_BIN) $(STAGE2_BIN) > $(BOOTLOADER_IMG)
	@echo "Bootloader image created: $(BOOTLOADER_IMG)"
	@echo "Total size: $$(stat -c%s $(BOOTLOADER_IMG) 2>/dev/null || stat -f%z $(BOOTLOADER_IMG)) bytes"

clean:
	@echo "Cleaning bootloader build files..."
	rm -f $(BOOT_BIN) $(STAGE2_BIN) $(BOOTLOADER_IMG)

help:
	@echo "RustrialOS Custom Bootloader Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all (default) - Build complete bootloader"
	@echo "  clean         - Remove build artifacts"
	@echo "  help          - Show this help message"
	@echo ""
	@echo "Requirements:"
	@echo "  - nasm (Netwide Assembler)"
	@echo "  - dd, cat (standard Unix tools)"
