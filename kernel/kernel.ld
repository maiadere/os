__phys_dram_start = 0xfffffffe00000000;
__rpi_kernel_entry = 0x80000;
__boot_code_max_size = 0x1000;

ENTRY(_start_rust)

SECTIONS {
    . = __phys_dram_start;
    
    .boot_stack (NOLOAD) : {
        . += __rpi_kernel_entry;
        __boot_stack_end = .;
    }

    . += __boot_code_max_size;
    
    .text : {
        KEEP(*(.text._start_rust))
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
