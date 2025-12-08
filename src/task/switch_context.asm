; switch_context.asm - context switch routine for x86_64
; Save/restore registers and switch stack pointer

section .text
    global switch_context_asm

switch_context_asm:
    ; Save caller-saved registers
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    ; TODO: Save stack pointer, instruction pointer, and switch to next task's PCB
    ; This is a stub. Actual implementation will depend on PCB layout and scheduler.

    ; Restore registers
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax

    ret
