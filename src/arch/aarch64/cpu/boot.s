.section .text._start

_start:
    wfe
    b _start

.size _start, . - _start
.type _start, function
.global _start
