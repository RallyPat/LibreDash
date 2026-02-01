use core::ptr;

const UART0_BASE: u32 = 0x3F201000;
const UART0_DR: u32 = UART0_BASE + 0x00;
const UART0_FR: u32 = UART0_BASE + 0x18;
const UART0_IBRD: u32 = UART0_BASE + 0x24;
const UART0_FBRD: u32 = UART0_BASE + 0x28;
const UART0_LCRH: u32 = UART0_BASE + 0x2C;
const UART0_CR: u32 = UART0_BASE + 0x30;
const UART0_ICR: u32 = UART0_BASE + 0x44;

const GPIO_BASE: u32 = 0x3F200000;
const GPFSEL1: u32 = GPIO_BASE + 0x04;
const GPPUD: u32 = GPIO_BASE + 0x94;
const GPPUDCLK0: u32 = GPIO_BASE + 0x98;

const UART_FR_TXFF: u32 = 1 << 5;

fn delay(count: u32) {
    for _ in 0..count {
        unsafe { 
            let dummy = 0u32;
            core::ptr::read_volatile(&dummy); 
        }
    }
}

pub fn uart_init() {
    unsafe {
        ptr::write_volatile(UART0_CR as *mut u32, 0);
        
        let mut ra = ptr::read_volatile(GPFSEL1 as *const u32);
        ra &= !(7 << 12);
        ra |= 4 << 12;
        ra &= !(7 << 15);
        ra |= 4 << 15;
        ptr::write_volatile(GPFSEL1 as *mut u32, ra);
        
        ptr::write_volatile(GPPUD as *mut u32, 0);
        delay(150);
        ptr::write_volatile(GPPUDCLK0 as *mut u32, (1 << 14) | (1 << 15));
        delay(150);
        ptr::write_volatile(GPPUDCLK0 as *mut u32, 0);
        
        ptr::write_volatile(UART0_ICR as *mut u32, 0x7FF);
        ptr::write_volatile(UART0_IBRD as *mut u32, 1);
        ptr::write_volatile(UART0_FBRD as *mut u32, 40);
        ptr::write_volatile(UART0_LCRH as *mut u32, 0x70);
        ptr::write_volatile(UART0_CR as *mut u32, 0x301);
    }
}

pub fn uart_putc(byte: u8) {
    unsafe {
        while (ptr::read_volatile(UART0_FR as *const u32) & UART_FR_TXFF) != 0 {}
        ptr::write_volatile(UART0_DR as *mut u32, byte as u32);
    }
}

pub fn uart_puts(s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' {
            uart_putc(b'\r');
        }
        uart_putc(byte);
    }
}

fn hex_digit(val: u8) -> u8 {
    if val < 10 { b'0' + val } else { b'a' + (val - 10) }
}

pub fn uart_hex_dump(addr: *const u8, len: usize) {
    for i in (0..len).step_by(16) {
        // Print address
        for shift in (0..8).rev() {
            uart_putc(hex_digit(((i >> (shift * 4)) & 0xF) as u8));
        }
        uart_puts(": ");
        
        // Print hex bytes
        for j in 0..16 {
            if i + j < len {
                unsafe {
                    let byte = *addr.add(i + j);
                    uart_putc(hex_digit((byte >> 4) & 0xF));
                    uart_putc(hex_digit(byte & 0xF));
                    uart_putc(b' ');
                }
            }
        }
        uart_putc(b'\n');
    }
}
