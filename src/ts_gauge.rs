// TunerStudio-compatible gauge rendering
// Matches the visual style of TunerStudio and Shadow Dash

use crate::framebuffer::Framebuffer;
use crate::ts_ini_parser::GaugeConfig;
use crate::math::{sin, cos};

// TunerStudio color scheme
pub const TS_COLOR_BACKGROUND: u32 = 0x000000;  // Black
pub const TS_COLOR_FOREGROUND: u32 = 0xFFFFFF;  // White
pub const TS_COLOR_NORMAL: u32 = 0x00FF00;      // Green
pub const TS_COLOR_WARNING: u32 = 0xFFFF00;     // Yellow
pub const TS_COLOR_DANGER: u32 = 0xFF0000;      // Red
pub const TS_COLOR_GRID: u32 = 0x404040;        // Dark gray
pub const TS_COLOR_TEXT: u32 = 0xFFFFFF;        // White

/// Gauge styles matching TunerStudio
#[derive(Copy, Clone, PartialEq)]
pub enum TSGaugeStyle {
    /// Horizontal bar gauge
    HorizontalBar,
    /// Vertical bar gauge
    VerticalBar,
    /// Circular needle gauge (analog)
    Circular,
    /// Digital numeric display
    Digital,
}

/// Gauge instance with current value
pub struct TSGauge {
    pub config: GaugeConfig,
    pub style: TSGaugeStyle,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub current_value: f32,
}

impl TSGauge {
    pub fn new(config: GaugeConfig, style: TSGaugeStyle, x: u32, y: u32, width: u32, height: u32) -> Self {
        TSGauge {
            config,
            style,
            x,
            y,
            width,
            height,
            current_value: 0.0,
        }
    }
    
    /// Update the current value
    pub fn set_value(&mut self, value: f32) {
        self.current_value = value;
    }
    
    /// Get the color based on current value and thresholds
    fn get_value_color(&self) -> u32 {
        let value = self.current_value;
        
        if value <= self.config.lo_danger || value >= self.config.hi_danger {
            TS_COLOR_DANGER
        } else if value <= self.config.lo_warning || value >= self.config.hi_warning {
            TS_COLOR_WARNING
        } else {
            TS_COLOR_NORMAL
        }
    }
    
    /// Render the gauge to framebuffer
    pub fn render(&self, fb: &mut Framebuffer) {
        match self.style {
            TSGaugeStyle::HorizontalBar => self.render_horizontal_bar(fb),
            TSGaugeStyle::VerticalBar => self.render_vertical_bar(fb),
            TSGaugeStyle::Circular => self.render_circular(fb),
            TSGaugeStyle::Digital => self.render_digital(fb),
        }
    }
    
    fn render_horizontal_bar(&self, fb: &mut Framebuffer) {
        // Draw background
        fb.draw_filled_rect(self.x, self.y, self.width, self.height, TS_COLOR_BACKGROUND);
        
        // Draw border
        fb.draw_rect(self.x, self.y, self.width, self.height, TS_COLOR_FOREGROUND);
        
        // Calculate fill percentage
        let range = self.config.hi - self.config.lo;
        let percentage = if range > 0.0 {
            ((self.current_value - self.config.lo) / range).max(0.0).min(1.0)
        } else {
            0.0
        };
        
        // Draw warning/danger zones as background
        self.draw_threshold_zones_horizontal(fb);
        
        // Draw fill bar
        let bar_height = self.height.saturating_sub(8);
        let bar_width = (self.width.saturating_sub(8) as f32 * percentage) as u32;
        let bar_color = self.get_value_color();
        
        if bar_width > 0 && bar_height > 0 {
            fb.draw_filled_rect(self.x + 4, self.y + 4, bar_width, bar_height, bar_color);
        }
        
        // Draw scale marks
        self.draw_scale_marks_horizontal(fb);
    }
    
    fn render_vertical_bar(&self, fb: &mut Framebuffer) {
        // Draw background
        fb.draw_filled_rect(self.x, self.y, self.width, self.height, TS_COLOR_BACKGROUND);
        
        // Draw border
        fb.draw_rect(self.x, self.y, self.width, self.height, TS_COLOR_FOREGROUND);
        
        // Calculate fill percentage
        let range = self.config.hi - self.config.lo;
        let percentage = if range > 0.0 {
            ((self.current_value - self.config.lo) / range).max(0.0).min(1.0)
        } else {
            0.0
        };
        
        // Draw fill bar (bottom to top)
        let bar_width = self.width.saturating_sub(8);
        let bar_height = (self.height.saturating_sub(8) as f32 * percentage) as u32;
        let bar_color = self.get_value_color();
        let bar_y = self.y + self.height - 4 - bar_height;
        
        if bar_width > 0 && bar_height > 0 {
            fb.draw_filled_rect(self.x + 4, bar_y, bar_width, bar_height, bar_color);
        }
    }
    
    fn render_circular(&self, fb: &mut Framebuffer) {
        // Draw background circle
        fb.draw_filled_rect(self.x, self.y, self.width, self.height, TS_COLOR_BACKGROUND);
        
        // Draw outer circle
        let center_x = self.x + self.width / 2;
        let center_y = self.y + self.height / 2;
        let radius = self.width.min(self.height) / 2 - 4;
        
        self.draw_circle(fb, center_x, center_y, radius, TS_COLOR_FOREGROUND);
        
        // Draw scale marks in circle
        for i in 0..=10 {
            let angle = -135.0 + (270.0 * i as f32 / 10.0);
            let angle_rad = angle * 3.14159 / 180.0;
            let x1 = center_x as i32 + ((radius as f32 * cos(angle_rad)) as i32);
            let y1 = center_y as i32 + ((radius as f32 * sin(angle_rad)) as i32);
            let x2 = center_x as i32 + (((radius - 5) as f32 * cos(angle_rad)) as i32);
            let y2 = center_y as i32 + (((radius - 5) as f32 * sin(angle_rad)) as i32);
            
            if x1 >= 0 && y1 >= 0 && x2 >= 0 && y2 >= 0 {
                self.draw_line(fb, x1 as u32, y1 as u32, x2 as u32, y2 as u32, TS_COLOR_GRID);
            }
        }
        
        // Draw needle
        let range = self.config.hi - self.config.lo;
        let percentage = if range > 0.0 {
            ((self.current_value - self.config.lo) / range).max(0.0).min(1.0)
        } else {
            0.0
        };
        
        let needle_angle = -135.0 + (270.0 * percentage);
        let needle_angle_rad = needle_angle * 3.14159 / 180.0;
        let needle_x = center_x as i32 + (((radius - 10) as f32 * cos(needle_angle_rad)) as i32);
        let needle_y = center_y as i32 + (((radius - 10) as f32 * sin(needle_angle_rad)) as i32);
        
        let needle_color = self.get_value_color();
        if needle_x >= 0 && needle_y >= 0 {
            self.draw_line(fb, center_x, center_y, needle_x as u32, needle_y as u32, needle_color);
        }
    }
    
    fn render_digital(&self, fb: &mut Framebuffer) {
        // Draw background
        fb.draw_filled_rect(self.x, self.y, self.width, self.height, TS_COLOR_BACKGROUND);
        
        // Draw border with color based on value
        let border_color = self.get_value_color();
        fb.draw_rect(self.x, self.y, self.width, self.height, border_color);
        fb.draw_rect(self.x + 1, self.y + 1, self.width - 2, self.height - 2, border_color);
        
        // Value would be drawn here with text rendering
        // For now, just show a colored indicator
        let indicator_width = self.width / 3;
        let indicator_height = self.height / 3;
        let indicator_x = self.x + (self.width - indicator_width) / 2;
        let indicator_y = self.y + (self.height - indicator_height) / 2;
        
        fb.draw_filled_rect(indicator_x, indicator_y, indicator_width, indicator_height, border_color);
    }
    
    fn draw_threshold_zones_horizontal(&self, fb: &mut Framebuffer) {
        let range = self.config.hi - self.config.lo;
        if range <= 0.0 {
            return;
        }
        
        let bar_height = self.height.saturating_sub(8);
        let total_width = self.width.saturating_sub(8);
        
        // Draw danger zones
        if self.config.lo_danger > self.config.lo {
            let zone_width = ((self.config.lo_danger - self.config.lo) / range * total_width as f32) as u32;
            fb.draw_filled_rect(self.x + 4, self.y + 4, zone_width, bar_height, 0x400000); // Dark red
        }
        
        if self.config.hi_danger < self.config.hi {
            let zone_start = ((self.config.hi_danger - self.config.lo) / range * total_width as f32) as u32;
            let zone_width = total_width - zone_start;
            fb.draw_filled_rect(self.x + 4 + zone_start, self.y + 4, zone_width, bar_height, 0x400000);
        }
    }
    
    fn draw_scale_marks_horizontal(&self, fb: &mut Framebuffer) {
        let marks = 10;
        let mark_spacing = (self.width.saturating_sub(8)) / marks;
        
        for i in 0..=marks {
            let x = self.x + 4 + (i * mark_spacing);
            let y = self.y + self.height - 4;
            fb.draw_filled_rect(x, y - 3, 1, 3, TS_COLOR_GRID);
        }
    }
    
    fn draw_circle(&self, fb: &mut Framebuffer, cx: u32, cy: u32, radius: u32, color: u32) {
        let r = radius as i32;
        let mut x = 0;
        let mut y = r;
        let mut d = 3 - 2 * r;
        
        while x <= y {
            self.draw_circle_points(fb, cx, cy, x, y, color);
            x += 1;
            if d < 0 {
                d = d + 4 * x + 6;
            } else {
                y -= 1;
                d = d + 4 * (x - y) + 10;
            }
        }
    }
    
    fn draw_circle_points(&self, fb: &mut Framebuffer, cx: u32, cy: u32, x: i32, y: i32, color: u32) {
        let points = [
            (cx as i32 + x, cy as i32 + y),
            (cx as i32 - x, cy as i32 + y),
            (cx as i32 + x, cy as i32 - y),
            (cx as i32 - x, cy as i32 - y),
            (cx as i32 + y, cy as i32 + x),
            (cx as i32 - y, cy as i32 + x),
            (cx as i32 + y, cy as i32 - x),
            (cx as i32 - y, cy as i32 - x),
        ];
        
        for (px, py) in points.iter() {
            if *px >= 0 && *py >= 0 {
                fb.draw_pixel(*px as u32, *py as u32, color);
            }
        }
    }
    
    fn draw_line(&self, fb: &mut Framebuffer, x0: u32, y0: u32, x1: u32, y1: u32, color: u32) {
        let mut x0 = x0 as i32;
        let mut y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;
        
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        
        loop {
            if x0 >= 0 && y0 >= 0 {
                fb.draw_pixel(x0 as u32, y0 as u32, color);
            }
            
            if x0 == x1 && y0 == y1 {
                break;
            }
            
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
    }
}
