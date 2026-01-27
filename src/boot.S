// Boot assembly code for Raspberry Pi
.section ".text.boot"

.global _start

_start:
    // Get CPU ID
    mrs     x1, mpidr_el1
    and     x1, x1, #3
    cbz     x1, 2f
    
    // CPU ID > 0, halt
1:  wfe
    b       1b
    
2:  // CPU ID 0, continue boot
    
    // Set stack pointer
    ldr     x1, =stack_top
    mov     sp, x1
    
    // Clear BSS
    ldr     x1, =__bss_start
    ldr     x2, =__bss_end
3:  cmp     x1, x2
    b.ge    4f
    str     xzr, [x1], #8
    b       3b
    
4:  // Jump to C code
    bl      kernel_main
    
    // Halt if kernel_main returns
5:  wfe
    b       5b
