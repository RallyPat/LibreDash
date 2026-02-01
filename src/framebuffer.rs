use core::ptr;

pub const COLOR_WHITE: u32 = 0xFFFFFF;
pub const COLOR_RED: u32 = 0xFF0000;
pub const COLOR_GREEN: u32 = 0x00FF00;
pub const COLOR_BLUE: u32 = 0x0000FF;
pub const COLOR_YELLOW: u32 = 0xFFFF00;
pub const COLOR_BLACK: u32 = 0x000000;
pub const COLOR_CYAN: u32 = 0x00FFFF;
pub const COLOR_MAGENTA: u32 = 0xFF00FF;
pub const COLOR_GRAY: u32 = 0x808080;

pub struct Framebuffer {
    buffer: *mut u32,
    width: u32,
    height: u32,
    pitch: u32,
}

impl Framebuffer {
    /// Create framebuffer with explicit address (allows QEMU and hardware to work)
    pub fn new(address: u32, width: u32, height: u32) -> Self {
        Framebuffer {
            buffer: address as *mut u32,
            width,
            height,
            pitch: width * 4,  // 32-bit pixels, 4 bytes per pixel
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
    
    pub fn width(&self) -> u32 {
        self.width
    }
    
    pub fn height(&self) -> u32 {
        self.height
    }
    
    pub fn buffer_ptr(&self) -> *const u8 {
        self.buffer as *const u8
    }
}
