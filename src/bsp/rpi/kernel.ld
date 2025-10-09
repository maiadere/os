__rpi_kernel_entry = 0x80000;

ENTRY(_start)

SECTIONS
{
    . = __rpi_kernel_entry;

    .text :
    {
        KEEP(*(.text._start))
    }
}
