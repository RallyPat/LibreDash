use crate::mmio;

const MAILBOX_TAG_SETPHYWH: u32 = 0x48003;
const MAILBOX_TAG_SETVIRTWH: u32 = 0x48004;
const MAILBOX_TAG_SETVIRTOFF: u32 = 0x48009;
const MAILBOX_TAG_SETDEPTH: u32 = 0x48005;
const MAILBOX_TAG_SETPXLORDR: u32 = 0x48006;
const MAILBOX_TAG_GETFB: u32 = 0x40001;
const MAILBOX_TAG_GETPITCH: u32 = 0x40008;
const MAILBOX_TAG_LAST: u32 = 0;

const MAILBOX_CH_PROP: u32 = 8;
const FB_ADDRESS_MASK: u32 = 0x3FFFFFFF;

// Color constants
pub const COLOR_BLACK: u32 = 0x000000;
pub const COLOR_WHITE: u32 = 0xFFFFFF;
pub const COLOR_RED: u32 = 0xFF0000;
pub const COLOR_GREEN: u32 = 0x00FF00;
pub const COLOR_BLUE: u32 = 0x0000FF;
pub const COLOR_YELLOW: u32 = 0xFFFF00;
pub const COLOR_CYAN: u32 = 0x00FFFF;
pub const COLOR_MAGENTA: u32 = 0xFF00FF;
pub const COLOR_GRAY: u32 = 0x808080;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    buffer: *mut u32,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let mut mailbox: [u32; 35] = [0; 35];
        
        mailbox[0] = 35 * 4;
        mailbox[1] = 0;
        
        // Set physical width/height
        mailbox[2] = MAILBOX_TAG_SETPHYWH;
        mailbox[3] = 8;
        mailbox[4] = 8;
        mailbox[5] = width;
        mailbox[6] = height;
        
        // Set virtual width/height
        mailbox[7] = MAILBOX_TAG_SETVIRTWH;
        mailbox[8] = 8;
        mailbox[9] = 8;
        mailbox[10] = width;
        mailbox[11] = height;
        
        // Set virtual offset
        mailbox[12] = MAILBOX_TAG_SETVIRTOFF;
        mailbox[13] = 8;
        mailbox[14] = 8;
        mailbox[15] = 0;
        mailbox[16] = 0;
        
        // Set depth (32-bit)
        mailbox[17] = MAILBOX_TAG_SETDEPTH;
        mailbox[18] = 4;
        mailbox[19] = 4;
        mailbox[20] = 32;
        
        // Set pixel order (RGB)
        mailbox[21] = MAILBOX_TAG_SETPXLORDR;
        mailbox[22] = 4;
        mailbox[23] = 4;
        mailbox[24] = 1;
        
        // Get framebuffer
        mailbox[25] = MAILBOX_TAG_GETFB;
        mailbox[26] = 8;
        mailbox[27] = 8;
        mailbox[28] = 4096;
        mailbox[29] = 0;
        
        // Get pitch
        mailbox[30] = MAILBOX_TAG_GETPITCH;
        mailbox[31] = 4;
        mailbox[32] = 4;
        mailbox[33] = 0;
        
        // End tag
        mailbox[34] = MAILBOX_TAG_LAST;
        
        mmio::mailbox_call(&mut mailbox, MAILBOX_CH_PROP);
        
        let pitch = mailbox[33];
        let buffer_addr = mailbox[28] & FB_ADDRESS_MASK;
        
        Framebuffer {
            width,
            height,
            pitch,
            buffer: buffer_addr as *mut u32,
        }
    }
    
    pub fn clear(&mut self, color: u32) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.draw_pixel(x, y, color);
            }
        }
    }
    
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        unsafe {
            let offset = y * (self.pitch / 4) + x;
            *self.buffer.add(offset as usize) = color;
        }
    }
    
    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        // Top and bottom
        for i in 0..w {
            self.draw_pixel(x + i, y, color);
            if h > 0 {
                self.draw_pixel(x + i, y + h - 1, color);
            }
        }
        // Left and right
        for i in 0..h {
            self.draw_pixel(x, y + i, color);
            if w > 0 {
                self.draw_pixel(x + w - 1, y + i, color);
            }
        }
    }
    
    pub fn draw_filled_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        for j in 0..h {
            for i in 0..w {
                self.draw_pixel(x + i, y + j, color);
            }
        }
    }
}
