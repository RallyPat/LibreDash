/* Linker script for Raspberry Pi bare metal */

ENTRY(_start)

SECTIONS
{
    /* Kernel loaded at 0x80000 for RPi */
    . = 0x80000;

    .text : {
        KEEP(*(.text.boot))
        *(.text)
    }

    .rodata : {
        *(.rodata)
    }

    .data : {
        *(.data)
    }

    .bss : {
        __bss_start = .;
        *(.bss)
        *(COMMON)
        __bss_end = .;
    }

    . = ALIGN(8);
    . = . + 0x1000; /* 4KB stack */
    stack_top = .;
}
