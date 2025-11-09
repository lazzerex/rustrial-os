; ============================================================================
; RustrialOS Custom Bootloader - Stage 1 (Boot Sector)
; ============================================================================
; Platform: x86-64 (all operating systems)
; Build with: NASM assembler
;   - Windows: nasm -f bin boot.asm -o boot.bin
;   - Linux/macOS: nasm -f bin boot.asm -o boot.bin
;
; This is loaded at 0x7C00 by BIOS and must be exactly 512 bytes.
; Its job is to load Stage 2 from disk into memory and jump to it.
; ============================================================================

[BITS 16]           ; Real mode (16-bit)
[ORG 0x7C00]        ; BIOS loads us here

; Entry point
start:
    ; Set up segments
    cli                     ; Disable interrupts
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00          ; Stack grows downward from bootloader
    sti                     ; Enable interrupts

    ; Save boot drive number (provided by BIOS in DL)
    mov [boot_drive], dl

    ; Print loading message
    mov si, msg_loading
    call print_string

    ; Load Stage 2 from disk
    ; Stage 2 is stored immediately after boot sector (sector 2 onwards)
    ; We'll load 64 sectors (32KB) to be safe
    mov bx, 0x7E00          ; Load Stage 2 at 0x7E00 (right after boot sector)
    mov dh, 64              ; Number of sectors to load
    mov dl, [boot_drive]    ; Drive number
    call load_sectors

    ; Check if load was successful
    jc disk_error

    ; Print success message
    mov si, msg_loaded
    call print_string

    ; Jump to Stage 2
    jmp 0x0000:0x7E00

; Disk error handler
disk_error:
    mov si, msg_disk_error
    call print_string
    jmp $                   ; Hang

; Function: Load sectors from disk
; Input: BX = memory offset, DH = number of sectors, DL = drive number
load_sectors:
    pusha
    mov ah, 0x02            ; BIOS read sector function
    mov al, dh              ; Number of sectors to read
    mov ch, 0x00            ; Cylinder 0
    mov cl, 0x02            ; Start from sector 2 (sector 1 is boot sector)
    mov dh, 0x00            ; Head 0
    ; DL already contains drive number
    int 0x13                ; BIOS disk interrupt
    jc .error               ; Jump if carry flag set (error)
    popa
    clc                     ; Clear carry flag (success)
    ret
.error:
    popa
    stc                     ; Set carry flag (error)
    ret

; Function: Print string
; Input: SI = pointer to null-terminated string
print_string:
    pusha
.loop:
    lodsb                   ; Load byte from SI into AL
    cmp al, 0               ; Check for null terminator
    je .done
    mov ah, 0x0E            ; BIOS teletype function
    mov bh, 0x00            ; Page number
    mov bl, 0x07            ; Text attribute (light gray)
    int 0x10                ; BIOS video interrupt
    jmp .loop
.done:
    popa
    ret

; Data
boot_drive:     db 0
msg_loading:    db 'RustrialOS Bootloader Stage 1...', 0x0D, 0x0A, 0
msg_loaded:     db 'Stage 2 loaded successfully!', 0x0D, 0x0A, 0
msg_disk_error: db 'Disk read error!', 0x0D, 0x0A, 0

; Padding and boot signature
times 510-($-$$) db 0   ; Pad to 510 bytes
dw 0xAA55               ; Boot signature (must be at bytes 510-511)
