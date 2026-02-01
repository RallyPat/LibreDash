/// Framebuffer configuration and environment detection
/// Supports both QEMU emulation and real hardware (Raspberry Pi)

use core::ptr;

pub enum FramebufferMode {
    QEMU,
    RealHardware,
}

pub struct FramebufferConfig {
    pub mode: FramebufferMode,
    pub address: u32,
    pub width: u32,
    pub height: u32,
}

impl FramebufferConfig {
    /// Detect runtime environment and return appropriate framebuffer configuration
    pub fn detect() -> Self {
        #[cfg(feature = "qemu")]
        {
            // QEMU mode: use fixed DRAM buffer that SDL will display
            return FramebufferConfig {
                mode: FramebufferMode::QEMU,
                address: 0x04000000,
                width: 1280,
                height: 720,
            };
        }

        #[cfg(feature = "hardware")]
        {
            // Real hardware mode: query GPU via mailbox for framebuffer allocation
            let address = query_gpu_framebuffer();
            return FramebufferConfig {
                mode: FramebufferMode::RealHardware,
                address,
                width: 1280,
                height: 720,
            };
        }

        // Fallback to QEMU mode if no feature specified
        #[cfg(not(any(feature = "qemu", feature = "hardware")))]
        {
            FramebufferConfig {
                mode: FramebufferMode::QEMU,
                address: 0x04000000,
                width: 1280,
                height: 720,
            }
        }
    }

    pub fn mode_name(&self) -> &'static str {
        match self.mode {
            FramebufferMode::QEMU => "QEMU",
            FramebufferMode::RealHardware => "Real Hardware",
        }
    }
}

/// BCM2835 Mailbox interface for querying GPU framebuffer
#[cfg(feature = "hardware")]
mod mailbox {
    use core::ptr;

    const MAILBOX_BASE: u32 = 0x3F00B880;
    const MAILBOX_READ: u32 = MAILBOX_BASE + 0x00;
    const MAILBOX_STATUS: u32 = MAILBOX_BASE + 0x18;
    const MAILBOX_WRITE: u32 = MAILBOX_BASE + 0x20;
    const MAILBOX_FULL: u32 = 0x80000000;
    const MAILBOX_EMPTY: u32 = 0x40000000;

    #[repr(C, align(16))]
    pub struct PropertyBuffer {
        size: u32,           // Buffer size in bytes
        code: u32,           // Request code (0 for request)
        // Property tags follow
        tag_allocate: u32,   // Tag ID: 0x00040001 (allocate framebuffer)
        tag_size: u32,       // Tag data size (6 words = 24 bytes)
        tag_status: u32,     // Tag status/response code
        width: u32,          // Framebuffer width
        height: u32,         // Framebuffer height
        depth: u32,          // Color depth (32-bit)
        pointer: u32,        // Response: GPU address
        size_response: u32,  // Response: Framebuffer size
        pitch: u32,          // Response: Bytes per line
        tag_end: u32,        // Tag end marker (0)
    }

    impl PropertyBuffer {
        pub fn new() -> Self {
            PropertyBuffer {
                size: 32,                    // Buffer size
                code: 0,                     // Request code
                tag_allocate: 0x00040001,    // Allocate framebuffer tag
                tag_size: 24,                // Data size (6 u32s)
                tag_status: 0,               // Status field
                width: 1280,
                height: 720,
                depth: 32,
                pointer: 0,
                size_response: 0,
                pitch: 0,
                tag_end: 0,                  // End tag
            }
        }
    }

    pub fn mailbox_write(channel: u32, data: u32) {
        let value = (data & !0xF) | (channel & 0xF);
        unsafe {
            while (ptr::read_volatile(MAILBOX_STATUS as *const u32) & MAILBOX_FULL) != 0 {}
            ptr::write_volatile(MAILBOX_WRITE as *mut u32, value);
        }
    }

    pub fn mailbox_read(channel: u32) -> u32 {
        unsafe {
            loop {
                while (ptr::read_volatile(MAILBOX_STATUS as *const u32) & MAILBOX_EMPTY) != 0 {}
                let data = ptr::read_volatile(MAILBOX_READ as *const u32);
                if (data & 0xF) == channel {
                    return data & !0xF;
                }
            }
        }
    }

    pub fn query_framebuffer() -> u32 {
        let mut buffer = PropertyBuffer::new();
        
        let buffer_addr = (&mut buffer as *mut PropertyBuffer) as u32;
        
        // Request framebuffer from GPU
        mailbox_write(8, buffer_addr);  // Channel 8 = property tags
        let _response = mailbox_read(8);
        
        // GPU returns framebuffer address (with status bits in lower bits)
        // Mask off the status bits to get the actual address
        buffer.pointer & 0x3FFFFFFF
    }
}

#[cfg(feature = "hardware")]
fn query_gpu_framebuffer() -> u32 {
    mailbox::query_framebuffer()
}

#[cfg(not(feature = "hardware"))]
fn query_gpu_framebuffer() -> u32 {
    // Fallback - should not reach here if features configured correctly
    0x04000000
}
