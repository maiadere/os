/*
 * Load the address of a symbol into a register, PC-relative.
 *
 * The symbol must lie within +/- 4 GiB of the Program Counter.
 *
 * # Resources
 *
 * - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
 */
.macro ADR_REL register, symbol
    adrp \register, \symbol
    add \register, \register, #:lo12:\symbol
.endm

.section .text._start

_start:
    /* All the CPU cores will jump to `_start` after the bootloader finishes */
    mrs x0, MPIDR_EL1
    and x0, x0, {CORE_ID_BITMASK}
    mov x1, {BOOT_CORE_ID}
    cmp x0, x1

    /* All non boot cores will jump into the parking loop */
    b.ne .L_parking_loop

    ADR_REL x0, __bss_start
    ADR_REL x1, __bss_end

    /* Zero the bss segment */
.L_bss_init_loop:
    cmp x0, x1
    b.eq .L_setup_rust
    stp xzr, xzr, [x0], #16
    b .L_bss_init_loop

.L_setup_rust:
    /* Set the stack pointer to the start of the boot stack */
    ADR_REL x0, __boot_stack_end
    mov sp, x0

    /* Jump into rust */
    b _start_rust

    /* Infinite loop to stop non-boot cores from executing */
.L_parking_loop:
    wfe
    b .L_parking_loop

.size _start, . - _start
.type _start, function
.global _start
