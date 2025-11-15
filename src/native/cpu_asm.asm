; ==============================================================================
; CPU Feature Detection Assembly Module
; ==============================================================================
; This module provides low-level CPUID instruction access for CPU detection
; Assembles with NASM for x86-64

global cpu_get_vendor
global cpu_get_features
global cpu_has_sse2
global cpu_has_avx
global cpu_get_brand

section .text
bits 64

; ------------------------------------------------------------------------------
; Get CPU vendor string (12 bytes: EBX, EDX, ECX from CPUID leaf 0)
; void cpu_get_vendor(char* buffer)
; ------------------------------------------------------------------------------
cpu_get_vendor:
    push rbx                ; Save RBX (callee-saved)
    mov r8, rdi             ; Save buffer pointer (rdi is first arg)
    
    xor eax, eax            ; CPUID leaf 0 (vendor string)
    cpuid
    
    ; Store vendor string: EBX, EDX, ECX
    mov [r8], ebx           ; First 4 bytes
    mov [r8 + 4], edx       ; Next 4 bytes
    mov [r8 + 8], ecx       ; Last 4 bytes
    
    pop rbx
    ret

; ------------------------------------------------------------------------------
; Get CPU feature flags from CPUID leaf 1
; uint64_t cpu_get_features(void) - Returns EDX:ECX
; ------------------------------------------------------------------------------
cpu_get_features:
    push rbx
    
    mov eax, 1              ; CPUID leaf 1 (features)
    cpuid
    
    ; Return EDX in upper 32 bits, ECX in lower 32 bits
    mov rax, rdx
    shl rax, 32
    or rax, rcx
    
    pop rbx
    ret

; ------------------------------------------------------------------------------
; Check if SSE2 is supported
; bool cpu_has_sse2(void) - Returns 1 if supported, 0 otherwise
; ------------------------------------------------------------------------------
cpu_has_sse2:
    push rbx
    
    mov eax, 1
    cpuid
    
    ; SSE2 is bit 26 of EDX
    bt edx, 26              ; Test bit 26
    setc al                 ; Set AL to 1 if carry (bit was set)
    movzx rax, al           ; Zero-extend to 64-bit
    
    pop rbx
    ret

; ------------------------------------------------------------------------------
; Check if AVX is supported
; bool cpu_has_avx(void) - Returns 1 if supported, 0 otherwise
; ------------------------------------------------------------------------------
cpu_has_avx:
    push rbx
    
    mov eax, 1
    cpuid
    
    ; AVX is bit 28 of ECX
    bt ecx, 28              ; Test bit 28
    setc al                 ; Set AL to 1 if carry
    movzx rax, al
    
    pop rbx
    ret

; ------------------------------------------------------------------------------
; Get CPU brand string (48 bytes from CPUID 0x80000002-0x80000004)
; void cpu_get_brand(char* buffer)
; ------------------------------------------------------------------------------
cpu_get_brand:
    push rbx
    mov r8, rdi             ; Save buffer pointer
    
    ; Check if extended CPUID is available
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000004
    jb .not_available
    
    ; Get brand string part 1
    mov eax, 0x80000002
    cpuid
    mov [r8], eax
    mov [r8 + 4], ebx
    mov [r8 + 8], ecx
    mov [r8 + 12], edx
    
    ; Get brand string part 2
    mov eax, 0x80000003
    cpuid
    mov [r8 + 16], eax
    mov [r8 + 20], ebx
    mov [r8 + 24], ecx
    mov [r8 + 28], edx
    
    ; Get brand string part 3
    mov eax, 0x80000004
    cpuid
    mov [r8 + 32], eax
    mov [r8 + 36], ebx
    mov [r8 + 40], ecx
    mov [r8 + 44], edx
    
    pop rbx
    ret

.not_available:
    ; Clear buffer if not available
    xor eax, eax
    mov rcx, 12             ; 48 bytes / 4
.clear_loop:
    mov [r8], eax
    add r8, 4
    loop .clear_loop
    
    pop rbx
    ret
