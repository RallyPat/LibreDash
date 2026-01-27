#ifndef MMIO_H
#define MMIO_H

#include <stdint.h>

// Memory-mapped I/O for Raspberry Pi

// Base address for Raspberry Pi 3
#define MMIO_BASE 0x3F000000

// GPIO registers
#define GPIO_BASE (MMIO_BASE + 0x200000)

// UART0 registers
#define UART0_BASE (GPIO_BASE + 0x1000)
#define UART0_DR   (UART0_BASE + 0x00)
#define UART0_FR   (UART0_BASE + 0x18)
#define UART0_IBRD (UART0_BASE + 0x24)
#define UART0_FBRD (UART0_BASE + 0x28)
#define UART0_LCRH (UART0_BASE + 0x2C)
#define UART0_CR   (UART0_BASE + 0x30)

// Mailbox registers
#define MAILBOX_BASE (MMIO_BASE + 0xB880)
#define MAILBOX_READ (MAILBOX_BASE + 0x00)
#define MAILBOX_STATUS (MAILBOX_BASE + 0x18)
#define MAILBOX_WRITE (MAILBOX_BASE + 0x20)

#define MAILBOX_FULL 0x80000000
#define MAILBOX_EMPTY 0x40000000

// MMIO functions
static inline void mmio_write(uint32_t reg, uint32_t data) {
    *(volatile uint32_t*)reg = data;
}

static inline uint32_t mmio_read(uint32_t reg) {
    return *(volatile uint32_t*)reg;
}

#endif // MMIO_H
