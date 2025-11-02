; RustrialOS Custom Bootloader - Stage 2
; Loaded at 0x7E00 by Stage 1
; Responsibilities:
;   1. Enable A20 line
;   2. Load GDT for protected mode
;   3. Switch to protected mode (32-bit)
;   4. Set up basic paging
;   5. Switch to long mode (64-bit)
;   6. Load kernel from disk
;   7. Jump to kernel entry point

[BITS 16]
[ORG 0x7E00]

stage2_start:
    ; Print Stage 2 message
    mov si, msg_stage2
    call print_string_16

    ; Enable A20 line (required for accessing memory above 1MB)
    call enable_a20

    ; Check A20
    call check_a20
    cmp ax, 1
    je .a20_enabled
    mov si, msg_a20_fail
    call print_string_16
    jmp $
.a20_enabled:
    mov si, msg_a20_ok
    call print_string_16

    ; Check for Long Mode support
    call check_long_mode
    cmp ax, 1
    je .long_mode_ok
    mov si, msg_no_long_mode
    call print_string_16
    jmp $
.long_mode_ok:
    mov si, msg_long_mode_ok
    call print_string_16

    ; Load kernel from disk (starting at sector 66, after bootloader)
    ; Load to 0x100000 (1MB mark - standard kernel location)
    mov si, msg_loading_kernel
    call print_string_16
    call load_kernel

    ; Set up paging for long mode
    call setup_paging

    ; Load GDT
    lgdt [gdt_descriptor]

    ; Switch to protected mode
    mov eax, cr0
    or eax, 0x1             ; Set PE (Protection Enable) bit
    mov cr0, eax

    ; Far jump to clear pipeline and enter protected mode
    jmp CODE_SEG:protected_mode_start

; Function: Enable A20 line using fast method
enable_a20:
    in al, 0x92
    or al, 2
    out 0x92, al
    ret

; Function: Check if A20 line is enabled
check_a20:
    pushf
    push ds
    push es
    push di
    push si
    cli
    xor ax, ax
    mov es, ax
    not ax
    mov ds, ax
    mov di, 0x0500
    mov si, 0x0510
    mov al, byte [es:di]
    push ax
    mov al, byte [ds:si]
    push ax
    mov byte [es:di], 0x00
    mov byte [ds:si], 0xFF
    cmp byte [es:di], 0xFF
    pop ax
    mov byte [ds:si], al
    pop ax
    mov byte [es:di], al
    mov ax, 0
    je .exit
    mov ax, 1
.exit:
    pop si
    pop di
    pop es
    pop ds
    popf
    ret

; Function: Check for Long Mode (64-bit) support
check_long_mode:
    ; Check if CPUID is supported
    pushfd
    pop eax
    mov ecx, eax
    xor eax, 0x00200000
    push eax
    popfd
    pushfd
    pop eax
    cmp eax, ecx
    je .no_long_mode

    ; Check for extended CPUID
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode

    ; Check for long mode bit
    mov eax, 0x80000001
    cpuid
    test edx, 0x20000000    ; LM bit
    jz .no_long_mode
    mov ax, 1
    ret
.no_long_mode:
    xor ax, ax
    ret

; Function: Load kernel from disk
load_kernel:
    pusha
    ; Load 128 sectors (64KB) to 0x10000 temporarily
    ; (We'll move it to 1MB later in protected mode)
    mov bx, 0x1000          ; Segment
    mov es, bx
    mov bx, 0x0000          ; Offset
    mov ah, 0x02            ; Read sectors
    mov al, 128             ; Number of sectors
    mov ch, 0x00            ; Cylinder
    mov cl, 66              ; Start sector (after bootloader)
    mov dh, 0x00            ; Head
    mov dl, [boot_drive]
    int 0x13
    jc .error
    popa
    mov si, msg_kernel_loaded
    call print_string_16
    ret
.error:
    popa
    mov si, msg_kernel_error
    call print_string_16
    jmp $

; Function: Set up identity paging for long mode
setup_paging:
    ; Point PML4 to PDPT
    mov edi, 0x1000         ; PML4 at 0x1000
    mov cr3, edi            ; Set page table address
    xor eax, eax
    mov ecx, 4096           ; Clear 4KB
    rep stosd
    mov edi, cr3

    ; Set up PML4[0] -> PDPT
    mov dword [edi], 0x2003 ; Present, writable, PDPT at 0x2000

    ; Set up PDPT[0] -> PD
    mov dword [0x2000], 0x3003 ; Present, writable, PD at 0x3000

    ; Set up PD entries (identity map first 2MB)
    mov dword [0x3000], 0x0083 ; Present, writable, huge page, address 0
    mov dword [0x3008], 0x200083 ; Second 2MB

    ret

; 16-bit helper functions
print_string_16:
    pusha
.loop:
    lodsb
    cmp al, 0
    je .done
    mov ah, 0x0E
    mov bh, 0x00
    mov bl, 0x07
    int 0x10
    jmp .loop
.done:
    popa
    ret

; Protected Mode (32-bit) code
[BITS 32]
protected_mode_start:
    ; Set up segments
    mov ax, DATA_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000        ; Set stack

    ; Enable PAE (Physical Address Extension)
    mov eax, cr4
    or eax, 0x20            ; Set PAE bit
    mov cr4, eax

    ; Load PML4 address to CR3 (already done, but let's be explicit)
    mov eax, 0x1000
    mov cr3, eax

    ; Enable long mode in EFER MSR
    mov ecx, 0xC0000080     ; EFER MSR
    rdmsr
    or eax, 0x00000100      ; Set LM bit
    wrmsr

    ; Enable paging (this activates long mode)
    mov eax, cr0
    or eax, 0x80000000      ; Set PG bit
    mov cr0, eax

    ; Load 64-bit GDT
    lgdt [gdt64_descriptor]

    ; Far jump to 64-bit code
    jmp CODE64_SEG:long_mode_start

; Long Mode (64-bit) code
[BITS 64]
long_mode_start:
    ; Set up segments
    mov ax, DATA64_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; Set up stack
    mov rsp, 0x200000       ; 2MB stack

    ; Move kernel from 0x10000 to 0x100000 (1MB)
    mov rsi, 0x10000
    mov rdi, 0x100000
    mov rcx, 0x8000         ; Copy 64KB (128 sectors * 512 bytes / 4)
    rep movsd

    ; Jump to kernel entry point
    ; The Rust kernel expects to be at 0x100000
    mov rax, 0x100000
    jmp rax

; Hang if we somehow return
    jmp $

; GDT for 32-bit protected mode
gdt_start:
    ; Null descriptor
    dq 0x0000000000000000
    ; Code segment
    dq 0x00CF9A000000FFFF
    ; Data segment
    dq 0x00CF92000000FFFF
gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dd gdt_start

CODE_SEG equ 0x08
DATA_SEG equ 0x10

; GDT for 64-bit long mode
gdt64_start:
    ; Null descriptor
    dq 0x0000000000000000
    ; Code segment (64-bit)
    dq 0x00AF9A000000FFFF
    ; Data segment
    dq 0x00AF92000000FFFF
gdt64_end:

gdt64_descriptor:
    dw gdt64_end - gdt64_start - 1
    dq gdt64_start

CODE64_SEG equ 0x08
DATA64_SEG equ 0x10

; Data section
boot_drive:             db 0x80
msg_stage2:             db 'Stage 2 initializing...', 0x0D, 0x0A, 0
msg_a20_ok:             db 'A20 line enabled', 0x0D, 0x0A, 0
msg_a20_fail:           db 'A20 line failed!', 0x0D, 0x0A, 0
msg_long_mode_ok:       db 'Long mode supported', 0x0D, 0x0A, 0
msg_no_long_mode:       db 'No Long mode support!', 0x0D, 0x0A, 0
msg_loading_kernel:     db 'Loading kernel...', 0x0D, 0x0A, 0
msg_kernel_loaded:      db 'Kernel loaded', 0x0D, 0x0A, 0
msg_kernel_error:       db 'Kernel load error!', 0x0D, 0x0A, 0

; Pad to make sure stage 2 is at least a few sectors
times 8192-($-$$) db 0  ; Pad to 8KB (16 sectors)
