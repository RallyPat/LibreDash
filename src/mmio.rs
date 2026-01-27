use core::ptr;

// Memory-mapped I/O base addresses for Raspberry Pi 3
const MMIO_BASE: u32 = 0x3F000000;
const GPIO_BASE: u32 = MMIO_BASE + 0x200000;

// Mailbox registers
const MAILBOX_BASE: u32 = MMIO_BASE + 0xB880;
const MAILBOX_READ: u32 = MAILBOX_BASE + 0x00;
const MAILBOX_STATUS: u32 = MAILBOX_BASE + 0x18;
const MAILBOX_WRITE: u32 = MAILBOX_BASE + 0x20;

const MAILBOX_FULL: u32 = 0x80000000;
const MAILBOX_EMPTY: u32 = 0x40000000;

pub fn mmio_write(reg: u32, data: u32) {
    unsafe {
        ptr::write_volatile(reg as *mut u32, data);
    }
}

pub fn mmio_read(reg: u32) -> u32 {
    unsafe {
        ptr::read_volatile(reg as *const u32)
    }
}

pub fn mailbox_call(buffer: &mut [u32], channel: u32) -> bool {
    let addr = buffer.as_ptr() as u32;
    let r = (addr & !0xF) | (channel & 0xF);
    
    // Wait for mailbox to be ready
    while (mmio_read(MAILBOX_STATUS) & MAILBOX_FULL) != 0 {}
    
    // Write address to mailbox
    mmio_write(MAILBOX_WRITE, r);
    
    // Wait for response
    loop {
        while (mmio_read(MAILBOX_STATUS) & MAILBOX_EMPTY) != 0 {}
        if r == mmio_read(MAILBOX_READ) {
            return buffer[1] == 0x80000000;
        }
    }
}
