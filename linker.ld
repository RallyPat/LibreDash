ENTRY(_start)

SECTIONS
{
    /* Kernel loaded at 0x80000 for RPi */
    . = 0x80000;

    .text : {
        KEEP(*(.text._start))
        *(.text*)
    }

    .rodata : {
        *(.rodata*)
    }

    .data : {
        *(.data*)
    }

    .bss : {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        __bss_end = .;
    }

    . = ALIGN(8);
    . = . + 0x8000; /* 32KB stack */
    __stack_top = .;
}
