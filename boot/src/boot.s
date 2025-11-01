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

/* all the CPU cores will jump to `_start` after the bootloader finishes */
_start:
    /* enable CNTP for EL1 */
    mrs x0, cnthctl_el2
    orr x0, x0, #3
    msr cnthctl_el2, x0
    msr cnthp_ctl_el2, xzr

    /* initialize virtual MPIDR */
    mrs x0, midr_el1
    mrs x2, mpidr_el1
    msr vpidr_el2, x0
    msr vmpidr_el2, x2
    
    /* disable coprocessor traps */
    mov x0, #0x33FF
    msr cptr_el2, x0
    msr hstr_el2, xzr
    mov x0, #(3 << 20)
    msr cpacr_el1, x0

    /* enable AArch64 in EL1 */
    mov x0, #(1 << 31)    /* AArch64 */
    orr x0, x0, #(1 << 1) /* SWIO hardwired on Pi3 */
    msr hcr_el2, x0
    mrs x0, hcr_el2

    /* setup SCTLR access */
    mov x2, #0x0800
    movk x2, #0x30d0, lsl #16
    msr sctlr_el1, x2

    ADR_REL x0, __boot_stack_end
    msr     sp_el1, x0

    /* drop down into el1 */
    ldr x0, =0x30C50830 /* all default values */
    msr SCTLR_EL1, x0

    /* drop to el1 */
    ldr x0, =0x3C5 /* set D A I F bits to mask all interrupts and stack to SP_EL1 */
    msr SPSR_EL2, x0
    adr x0, el1
    msr ELR_EL2, x0
    eret

el1:
    ADR_REL x0, __bss_start
    ADR_REL x1, __bss_end

    /* zero the bss segment */
.L_bss_init_loop:
    cmp x0, x1
    b.eq .L_setup_rust
    stp xzr, xzr, [x0], #16
    b .L_bss_init_loop

.L_setup_rust:
    /* jump into rust */
    b _start_rust


.size _start, . - _start
.type _start, function
.global _start
