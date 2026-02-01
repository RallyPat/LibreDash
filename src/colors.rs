/// Color utilities for gauge rendering
/// Provides color selection based on value thresholds (danger/warning/normal zones)
/// and color conversion utilities for the framebuffer

/// RGB color as 24-bit value (0xRRGGBB)
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    /// Convert to 24-bit RGB integer
    pub fn to_u32(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Create color from 24-bit RGB integer
    pub fn from_u32(color: u32) -> Self {
        Color {
            r: ((color >> 16) & 0xFF) as u8,
            g: ((color >> 8) & 0xFF) as u8,
            b: (color & 0xFF) as u8,
        }
    }
}

pub mod colors {
    use super::Color;

    /// Normal/Safe operating zone
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    
    /// Warning zone - approaching limits
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0 };
    
    /// Danger zone - critical
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    
    /// Neutral backgrounds
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const DARK_GRAY: Color = Color { r: 64, g: 64, b: 64 };
    pub const LIGHT_GRAY: Color = Color { r: 192, g: 192, b: 192 };
    
    /// Display colors
    pub const CYAN: Color = Color { r: 0, g: 255, b: 255 };
    pub const MAGENTA: Color = Color { r: 255, g: 0, b: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const ORANGE: Color = Color { r: 255, g: 165, b: 0 };
}

/// Gauge status based on current value relative to thresholds
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GaugeStatus {
    /// Value in danger/critical zone
    Danger,
    /// Value in warning zone
    Warning,
    /// Value in normal operating range
    Normal,
}

/// Calculate gauge color based on value and thresholds
pub fn get_gauge_color(
    current_value: f32,
    lo_danger: f32,
    lo_warning: f32,
    hi_warning: f32,
    hi_danger: f32,
) -> Color {
    if current_value <= lo_danger || current_value >= hi_danger {
        colors::RED
    } else if current_value <= lo_warning || current_value >= hi_warning {
        colors::YELLOW
    } else {
        colors::GREEN
    }
}

/// Determine gauge status
pub fn get_gauge_status(
    current_value: f32,
    lo_danger: f32,
    lo_warning: f32,
    hi_warning: f32,
    hi_danger: f32,
) -> GaugeStatus {
    if current_value <= lo_danger || current_value >= hi_danger {
        GaugeStatus::Danger
    } else if current_value <= lo_warning || current_value >= hi_warning {
        GaugeStatus::Warning
    } else {
        GaugeStatus::Normal
    }
}

/// Interpolate between two colors based on progress (0.0 to 1.0)
pub fn interpolate_color(color1: Color, color2: Color, progress: f32) -> Color {
    let p = if progress < 0.0 { 0.0 } else if progress > 1.0 { 1.0 } else { progress };
    
    Color {
        r: (color1.r as f32 * (1.0 - p) + color2.r as f32 * p) as u8,
        g: (color1.g as f32 * (1.0 - p) + color2.g as f32 * p) as u8,
        b: (color1.b as f32 * (1.0 - p) + color2.b as f32 * p) as u8,
    }
}
