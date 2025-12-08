; switch_context.asm - context switch routine for x86_64
; Save/restore registers and switch stack pointer

section .text
    global switch_context_asm

switch_context_asm:
    ; Arguments:
    ; rcx = pointer to current PCB (mut)
    ; rdx = pointer to next PCB (const)

    ; Save all callee-saved registers to current PCB
    mov [rcx + 0x00], r15
    mov [rcx + 0x08], r14
    mov [rcx + 0x10], r13
    mov [rcx + 0x18], r12
    mov [rcx + 0x20], r11
    mov [rcx + 0x28], r10
    mov [rcx + 0x30], r9
    mov [rcx + 0x38], r8
    mov [rcx + 0x40], rsi
    mov [rcx + 0x48], rdi
    mov [rcx + 0x50], rbp
    mov [rcx + 0x58], rdx
    mov [rcx + 0x60], rcx
    mov [rcx + 0x68], rbx
    mov [rcx + 0x70], rax
    mov [rcx + 0x78], rsp
    ; Save rip/rflags if needed (requires more logic)

    ; Load all registers from next PCB
    mov r15, [rdx + 0x00]
    mov r14, [rdx + 0x08]
    mov r13, [rdx + 0x10]
    mov r12, [rdx + 0x18]
    mov r11, [rdx + 0x20]
    mov r10, [rdx + 0x28]
    mov r9,  [rdx + 0x30]
    mov r8,  [rdx + 0x38]
    mov rsi, [rdx + 0x40]
    mov rdi, [rdx + 0x48]
    mov rbp, [rdx + 0x50]
    mov rdx, [rdx + 0x58]
    mov rcx, [rdx + 0x60]
    mov rbx, [rdx + 0x68]
    mov rax, [rdx + 0x70]
    mov rsp, [rdx + 0x78]
    ; Load rip/rflags if needed (requires more logic)

    ; Switch to next task's stack
    ; Use ret to jump to next rip (if saved on stack)
    ret
