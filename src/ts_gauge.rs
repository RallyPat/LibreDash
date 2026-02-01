/// TunerStudio-compatible gauge rendering engine
/// Supports multiple gauge styles with color-coded thresholds
/// Handles value updates and smooth rendering

use crate::framebuffer::Framebuffer;
use crate::ts_ini_parser::GaugeConfig;
use crate::colors::{Color, get_gauge_color, colors};
use crate::math::sin;
use core::f32::consts::PI;

#[derive(Clone, Copy, Debug)]
pub enum TSGaugeStyle {
    Circular,       // Analog needle gauge
    HorizontalBar,  // Left-to-right bar
    VerticalBar,    // Bottom-to-top bar
    Digital,        // Large numeric display
}

pub struct TSGauge {
    pub config: GaugeConfig,
    pub style: TSGaugeStyle,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub current_value: f32,
    pub last_rendered_value: f32,
    pub animation_progress: f32,
    pub dirty: bool,
}

impl TSGauge {
    pub fn new(
        config: GaugeConfig,
        style: TSGaugeStyle,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Self {
        TSGauge {
            config,
            style,
            x,
            y,
            width,
            height,
            current_value: 0.0,
            last_rendered_value: 0.0,
            animation_progress: 0.0,
            dirty: true,
        }
    }

    /// Set gauge value and mark as dirty if changed
    pub fn set_value(&mut self, value: f32) {
        // Clamp to min/max range
        let clamped = if value < self.config.lo {
            self.config.lo
        } else if value > self.config.hi {
            self.config.hi
        } else {
            value
        };

        // Mark dirty if value changed significantly (>1% of range)
        let range = self.config.hi - self.config.lo;
        let change_threshold = range * 0.01;
        
        if (clamped - self.current_value).abs() > change_threshold {
            self.dirty = true;
            self.animation_progress = 0.0;
        }

        self.current_value = clamped;
    }

    /// Get interpolated value for animation (0.0 to 1.0 progress)
    pub fn get_animated_value(&self) -> f32 {
        // Linear interpolation from last rendered to current
        let progress = self.animation_progress.min(1.0);
        self.last_rendered_value + (self.current_value - self.last_rendered_value) * progress
    }

    /// Get gauge color based on current value
    pub fn get_color(&self) -> Color {
        get_gauge_color(
            self.current_value,
            self.config.lo_danger,
            self.config.lo_warning,
            self.config.hi_warning,
            self.config.hi_danger,
        )
    }

    /// Normalize value to 0.0-1.0 range based on min/max
    pub fn get_normalized_value(&self, value: f32) -> f32 {
        let range = self.config.hi - self.config.lo;
        if range <= 0.0 {
            return 0.0;
        }
        ((value - self.config.lo) / range).max(0.0).min(1.0)
    }

    /// Render gauge to framebuffer
    pub fn render(&mut self, fb: &mut Framebuffer) {
        if !self.dirty && self.animation_progress >= 1.0 {
            return; // Nothing to render
        }

        match self.style {
            TSGaugeStyle::Circular => self.render_circular(fb),
            TSGaugeStyle::HorizontalBar => self.render_horizontal_bar(fb),
            TSGaugeStyle::VerticalBar => self.render_vertical_bar(fb),
            TSGaugeStyle::Digital => self.render_digital(fb),
        }

        self.animation_progress += 0.5; // Advance animation
        if self.animation_progress >= 1.0 {
            self.last_rendered_value = self.current_value;
            self.dirty = false;
        }
    }

    /// Render circular needle gauge
    fn render_circular(&mut self, fb: &mut Framebuffer) {
        let center_x = self.x + self.width / 2;
        let center_y = self.y + self.height / 2;
        let radius = (self.width.min(self.height) / 2) as f32 * 0.85;

        let color = self.get_color();

        // Draw outer circle (border)
        self.draw_circle(fb, center_x, center_y, radius as u32, color.to_u32());

        // Draw background circle
        if radius > 5.0 {
            self.draw_circle(fb, center_x, center_y, (radius - 5.0) as u32, colors::DARK_GRAY.to_u32());
        }

        // Calculate needle angle: -180° to 0° for typical gauge
        let normalized = self.get_normalized_value(self.get_animated_value());
        let angle_degrees = -180.0 + (normalized * 180.0);
        let angle_rad = angle_degrees * PI / 180.0;

        // Draw needle using sine/cosine
        let needle_length = (radius * 0.75) as i32;
        let cos_angle = sin(angle_rad + PI / 2.0); // cos = sin(x + π/2)
        let sin_angle = sin(angle_rad);
        
        let needle_end_x = center_x as i32 + (cos_angle * needle_length as f32) as i32;
        let needle_end_y = center_y as i32 + (sin_angle * needle_length as f32) as i32;

        self.draw_line(
            fb,
            center_x as i32,
            center_y as i32,
            needle_end_x,
            needle_end_y,
            color.to_u32(),
        );

        // Draw center dot
        fb.draw_filled_rect(center_x - 3, center_y - 3, 6, 6, color.to_u32());

        // Draw title below gauge
        self.draw_title(fb, color);
    }

    /// Render horizontal bar gauge
    fn render_horizontal_bar(&mut self, fb: &mut Framebuffer) {
        let color = self.get_color();
        let normalized = self.get_normalized_value(self.get_animated_value());
        let fill_width = (self.width as f32 * normalized) as u32;

        // Draw border
        fb.draw_rect(self.x, self.y, self.width, self.height, color.to_u32());

        // Draw background
        fb.draw_filled_rect(self.x + 2, self.y + 2, self.width - 4, self.height - 4, colors::DARK_GRAY.to_u32());

        // Draw fill
        if fill_width > 0 {
            fb.draw_filled_rect(
                self.x + 2,
                self.y + 2,
                fill_width.saturating_sub(4),
                self.height - 4,
                color.to_u32(),
            );
        }

        // Draw title
        self.draw_title(fb, color);
    }

    /// Render vertical bar gauge
    fn render_vertical_bar(&mut self, fb: &mut Framebuffer) {
        let color = self.get_color();
        let normalized = self.get_normalized_value(self.get_animated_value());
        let fill_height = (self.height as f32 * normalized) as u32;

        // Draw border
        fb.draw_rect(self.x, self.y, self.width, self.height, color.to_u32());

        // Draw background
        fb.draw_filled_rect(self.x + 2, self.y + 2, self.width - 4, self.height - 4, colors::DARK_GRAY.to_u32());

        // Draw fill from bottom up
        if fill_height > 0 {
            let fill_y = self.y + self.height.saturating_sub(2).saturating_sub(fill_height);
            fb.draw_filled_rect(
                self.x + 2,
                fill_y,
                self.width - 4,
                fill_height.saturating_sub(4),
                color.to_u32(),
            );
        }

        // Draw title
        self.draw_title(fb, color);
    }

    /// Render digital numeric display with colored border
    fn render_digital(&mut self, fb: &mut Framebuffer) {
        let color = self.get_color();
        let value = self.get_animated_value();

        // Draw colored border frame
        fb.draw_rect(self.x, self.y, self.width, self.height, color.to_u32());
        fb.draw_rect(
            self.x + 1,
            self.y + 1,
            self.width.saturating_sub(2),
            self.height.saturating_sub(2),
            color.to_u32(),
        );

        // Draw dark background
        fb.draw_filled_rect(
            self.x + 4,
            self.y + 4,
            self.width.saturating_sub(8),
            self.height.saturating_sub(8),
            colors::DARK_GRAY.to_u32(),
        );

        // Draw numeric value using digit renderer
        let digit_size = (self.height / 3).min(20);
        let text_x = self.x + 10;
        let text_y = self.y + (self.height.saturating_sub(digit_size * 2)) / 2;

        crate::digit_renderer::draw_float(fb, value, 4, 1, text_x, text_y, digit_size, color);

        // Draw title
        self.draw_title(fb, color);
    }

    /// Draw gauge title text (simplified - using rectangles as placeholder)
    fn draw_title(&self, fb: &mut Framebuffer, color: Color) {
        // Placeholder: Draw a small rectangle below gauge for title area
        let title_y = self.y + self.height + 2;
        fb.draw_filled_rect(self.x, title_y, self.width, 10, colors::BLACK.to_u32());
    }

    /// Draw circle using Bresenham-style algorithm
    fn draw_circle(&self, fb: &mut Framebuffer, cx: u32, cy: u32, radius: u32, color: u32) {
        if radius == 0 {
            return;
        }

        let r = radius as i32;
        let mut x = r;
        let mut y = 0;
        let mut decision_parameter = 3 - 2 * r;

        while x >= y {
            // Draw 8 symmetric points
            self.draw_circle_point(fb, cx, cy, x as u32, y as u32, color);
            self.draw_circle_point(fb, cx, cy, y as u32, x as u32, color);

            if decision_parameter <= 0 {
                decision_parameter = decision_parameter + 4 * y + 6;
            } else {
                decision_parameter = decision_parameter + 4 * (y - x) + 10;
                x -= 1;
            }
            y += 1;
        }
    }

    /// Draw circle points in 8 symmetric positions
    fn draw_circle_point(&self, fb: &mut Framebuffer, cx: u32, cy: u32, x: u32, y: u32, color: u32) {
        let points = [
            (cx + x, cy + y),
            (cx - x, cy + y),
            (cx + x, cy - y),
            (cx - x, cy - y),
            (cx + y, cy + x),
            (cx - y, cy + x),
            (cx + y, cy - x),
            (cx - y, cy - x),
        ];

        for (px, py) in &points {
            if *px < 1280 && *py < 720 {
                fb.draw_filled_rect(*px - 1, *py - 1, 2, 2, color);
            }
        }
    }

    /// Draw line using Bresenham algorithm
    fn draw_line(&self, fb: &mut Framebuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx as i32 - dy as i32;
        let mut x = x0;
        let mut y = y0;

        loop {
            if x >= 0 && x < 1280 && y >= 0 && y < 720 {
                fb.draw_filled_rect(x as u32, y as u32, 2, 2, color);
            }

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -(dy as i32) {
                err -= dy as i32;
                x += sx;
            }
            if e2 < dx as i32 {
                err += dx as i32;
                y += sy;
            }
        }
    }
}
