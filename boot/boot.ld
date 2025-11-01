__phys_dram_start = 0x0;
__rpi_kernel_entry = 0x80000;

ENTRY(_start)

SECTIONS {
    . = __phys_dram_start;
    
    .boot_stack (NOLOAD) : {
        . += __rpi_kernel_entry;
        __boot_stack_end = .;
    }

    .text : {
        KEEP(*(.text._start))
        *(.text._start_rust)
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
        *(.bss*);
        . = ALIGN(16);
        __bss_end = .;
    } :segment_data

    .got : { *(.got*) }
    ASSERT(SIZEOF(.got) == 0, "Relocation support not expected")

    /DISCARD/ : { *(.comment*) }
}
