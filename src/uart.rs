// UART driver for serial communication with ECU
// Optimized for fast initialization and low latency

use crate::mmio::{mmio_write, mmio_read};

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

pub struct Uart {
    initialized: bool,
}

impl Uart {
    pub fn new() -> Self {
        Uart { initialized: false }
    }
    
    /// Initialize UART for fast ECU communication
    /// Standard baud rates: 9600, 19200, 38400, 57600, 115200
    pub fn init(&mut self, baud_rate: u32) {
        // Disable UART
        mmio_write(UART0_CR, 0);
        
        // Setup GPIO pins 14 & 15 for UART
        let mut ra = mmio_read(GPFSEL1);
        ra &= !((7 << 12) | (7 << 15)); // Clear GPIO 14 & 15
        ra |= (4 << 12) | (4 << 15);    // Alt 0 for UART
        mmio_write(GPFSEL1, ra);
        
        // Disable pull up/down for pins 14 & 15
        mmio_write(GPPUD, 0);
        delay(150);
        mmio_write(GPPUDCLK0, (1 << 14) | (1 << 15));
        delay(150);
        mmio_write(GPPUDCLK0, 0);
        
        // Clear interrupts
        mmio_write(UART0_ICR, 0x7FF);
        
        // Set baud rate
        // Baud divisor = UART_CLOCK / (16 * baud_rate)
        // UART_CLOCK = 3MHz (RPi3)
        let divisor = 3000000 / (16 * baud_rate);
        let fractional = ((3000000 * 4 / baud_rate) % 64) & 0x3F;
        
        mmio_write(UART0_IBRD, divisor);
        mmio_write(UART0_FBRD, fractional);
        
        // Enable FIFO, 8N1 (8 bits, no parity, 1 stop bit)
        mmio_write(UART0_LCRH, (1 << 4) | (3 << 5));
        
        // Enable UART, TX, RX
        mmio_write(UART0_CR, (1 << 0) | (1 << 8) | (1 << 9));
        
        self.initialized = true;
    }
    
    /// Send a single byte (non-blocking check, then wait)
    pub fn send_byte(&self, byte: u8) {
        // Wait for UART to be ready
        while (mmio_read(UART0_FR) & (1 << 5)) != 0 {}
        mmio_write(UART0_DR, byte as u32);
    }
    
    /// Send multiple bytes
    pub fn send_bytes(&self, data: &[u8]) {
        for &byte in data {
            self.send_byte(byte);
        }
    }
    
    /// Receive a single byte (blocking with timeout)
    pub fn recv_byte(&self, timeout_cycles: u32) -> Option<u8> {
        let mut cycles = 0;
        
        // Wait for data to be available
        while (mmio_read(UART0_FR) & (1 << 4)) != 0 {
            cycles += 1;
            if cycles > timeout_cycles {
                return None;
            }
        }
        
        Some((mmio_read(UART0_DR) & 0xFF) as u8)
    }
    
    /// Check if data is available to read
    pub fn has_data(&self) -> bool {
        (mmio_read(UART0_FR) & (1 << 4)) == 0
    }
    
    /// Receive multiple bytes into buffer (with timeout)
    pub fn recv_bytes(&self, buffer: &mut [u8], timeout_cycles: u32) -> usize {
        let mut count = 0;
        
        for i in 0..buffer.len() {
            if let Some(byte) = self.recv_byte(timeout_cycles) {
                buffer[i] = byte;
                count += 1;
            } else {
                break;
            }
        }
        
        count
    }
    
    /// Flush receive buffer
    pub fn flush_rx(&self) {
        while self.has_data() {
            let _ = mmio_read(UART0_DR);
        }
    }
}

/// Simple delay function (CPU cycles)
fn delay(cycles: u32) {
    for _ in 0..cycles {
        unsafe {
            core::ptr::read_volatile(&0u32);
        }
    }
}

impl Default for Uart {
    fn default() -> Self {
        Self::new()
    }
}
