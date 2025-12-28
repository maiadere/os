.text
.global el1_vector_table
.section .text._el1_vector_table

.balign 2048
el1_vector_table:

/* exceptions from current EL while using the SP_EL0 stack (exceptions from kernel)*/
  /* Synchronous */ 
  .balign 0x80
  b .
  /* IRQ */
  .balign 0x80
  b .
  /* FIQ */
  .balign 0x80
  b .
  /* SError */
  .balign 0x80
  b .

/* exceptions from current EL while using the SP_ELx stack (exceptions from kernel)*/
  /* Synchronous */ 
  .balign 0x80
  b .
  /* IRQ */
  .balign 0x80
  b .
  /* FIQ */
  .balign 0x80
  b .
  /* SError */
  .balign 0x80
  b .

/* exceptions from lower EL and at least one lower el is running 64 bit (exceptions from userspace)*/
  /* Synchronous */ 
  .balign 0x80
  b .
  /* IRQ */
  .balign 0x80
  b .
  /* FIQ */
  .balign 0x80
  b .
  /* SError */
  .balign 0x80
  b .

/* exceptions from lower EL and all lower el are running 32 bit (should never happen)*/
  /* Synchronous */ 
  .balign 0x80
  b .
  /* IRQ */
  .balign 0x80
  b .
  /* FIQ */
  .balign 0x80
  b .
  /* SError */
  .balign 0x80
  b .

