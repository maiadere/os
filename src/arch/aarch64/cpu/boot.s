.section .text._start

// all the rpi cpu cores will jump to _start after the bootloader finishes
_start:
    wfe
    b _start
    mrs x0, MPIDR_EL1
    and x0, {CORE_ID_BITMASK}

// infinite loop to stop non-boot cores from executing
.L_parking_loop:

.size _start, . - _start
.type _start, function
.global _start
