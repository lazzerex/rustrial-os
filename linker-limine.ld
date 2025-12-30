ENTRY(_start)

PHDRS
{
    text PT_LOAD FLAGS(5);    /* R E (read + execute) */
    rodata PT_LOAD FLAGS(4);  /* R   (read only) */
    data PT_LOAD FLAGS(6);    /* RW  (read + write) */
}

SECTIONS
{
    /* Limine requires the kernel to be in the higher half */
    . = 0xFFFFFFFF80000000;

    /* Read-execute section */
    .text : ALIGN(4K) {
        *(.text .text.*)
    } :text

    /* Force separate segment by adding gap */
    . += 0x1000;
    . = ALIGN(4K);
    
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.eh_frame_hdr)
        /* Include GOT in read-only section to avoid separate PHDR */
        *(.got .got.*)
    } :rodata

    /* Force separate writable segment with larger gap */
    . += 0x1000;
    . = ALIGN(4K);
    
    /* Limine requires requests at a known location - put them FIRST in data segment */
    .data : ALIGN(4K) {
        /* Limine requests MUST be first and writable */
        KEEP(*(.limine_reqs))
        *(.data .data.*)
    } :data

    . = ALIGN(4K);
    .bss (NOLOAD) : ALIGN(4K) {
        *(.bss .bss.*)
        *(COMMON)
    } :data

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
    }
}
