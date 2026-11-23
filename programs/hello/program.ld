__program_entry = 0x80000;

ENTRY(_start)

SECTIONS {
    . = __program_entry;

    /* the kernel enters a program at the first byte of its image, so _start has to come first */
    .text : {
        KEEP(*(.text._start))
        *(.text*)
    }

    .rodata : ALIGN(8) {
        *(.rodata*)
    }

    .data : ALIGN(8) {
        *(.data*)
    }

    .bss (NOLOAD) : ALIGN(16) {
        __bss_start = .;
        *(.bss*)
        . = ALIGN(16);
        __bss_end = .;
    }

    .got : { *(.got*) }

    /* the loader copies the image verbatim and does not zero bss, so there must not be any */
    ASSERT(__bss_end == __bss_start, "programs must not have a .bss section")
    ASSERT(SIZEOF(.got) == 0, "Relocation support not expected")

    /DISCARD/ : { *(.comment*) }
}
