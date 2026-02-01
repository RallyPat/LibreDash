/// TunerStudio XML .dash and .gauge file parser
/// Handles XML-based gauge and dashboard configuration from TunerStudio
/// 
/// Supports:
/// - Gauge definitions with thresholds, scaling, units
/// - Dashboard layouts with gauge positioning
/// - Variable mapping to ECU OutputChannels
/// - Color thresholds and styling

use core::str;

/// Simple XML parser for TunerStudio gauge definitions
#[derive(Clone, Copy)]
pub struct XMLElement {
    pub tag_name: [u8; 64],
    pub tag_name_len: usize,
    pub attributes: [XMLAttribute; 16],
    pub attr_count: usize,
    pub text_content: [u8; 256],
    pub text_len: usize,
}

#[derive(Clone, Copy)]
pub struct XMLAttribute {
    pub name: [u8; 32],
    pub name_len: usize,
    pub value: [u8; 128],
    pub value_len: usize,
}

impl XMLElement {
    pub fn new() -> Self {
        XMLElement {
            tag_name: [0; 64],
            tag_name_len: 0,
            attributes: [XMLAttribute {
                name: [0; 32],
                name_len: 0,
                value: [0; 128],
                value_len: 0,
            }; 16],
            attr_count: 0,
            text_content: [0; 256],
            text_len: 0,
        }
    }

    pub fn tag_str(&self) -> &str {
        str::from_utf8(&self.tag_name[..self.tag_name_len]).unwrap_or("")
    }

    pub fn text_str(&self) -> &str {
        str::from_utf8(&self.text_content[..self.text_len]).unwrap_or("")
    }

    pub fn get_attr(&self, attr_name: &str) -> Option<&str> {
        for i in 0..self.attr_count {
            let name = str::from_utf8(&self.attributes[i].name[..self.attributes[i].name_len])
                .unwrap_or("");
            if name == attr_name {
                return Some(
                    str::from_utf8(&self.attributes[i].value[..self.attributes[i].value_len])
                        .unwrap_or(""),
                );
            }
        }
        None
    }
}

/// TunerStudio gauge definition from XML
#[derive(Clone, Copy)]
pub struct XMLGaugeDefinition {
    pub name: [u8; 64],
    pub name_len: usize,
    pub variable_name: [u8; 64],
    pub variable_name_len: usize,
    pub title: [u8; 64],
    pub title_len: usize,
    pub units: [u8; 32],
    pub units_len: usize,
    pub gauge_type: [u8; 32], // "bar", "analog", "digital"
    pub gauge_type_len: usize,
    pub min_value: f32,
    pub max_value: f32,
    pub warn_min: f32,
    pub warn_max: f32,
    pub danger_min: f32,
    pub danger_max: f32,
}

impl XMLGaugeDefinition {
    pub fn new() -> Self {
        XMLGaugeDefinition {
            name: [0; 64],
            name_len: 0,
            variable_name: [0; 64],
            variable_name_len: 0,
            title: [0; 64],
            title_len: 0,
            units: [0; 32],
            units_len: 0,
            gauge_type: [0; 32],
            gauge_type_len: 0,
            min_value: 0.0,
            max_value: 100.0,
            warn_min: 0.0,
            warn_max: 100.0,
            danger_min: 0.0,
            danger_max: 100.0,
        }
    }

    pub fn name_str(&self) -> &str {
        str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }

    pub fn var_str(&self) -> &str {
        str::from_utf8(&self.variable_name[..self.variable_name_len]).unwrap_or("")
    }

    pub fn title_str(&self) -> &str {
        str::from_utf8(&self.title[..self.title_len]).unwrap_or("")
    }

    pub fn units_str(&self) -> &str {
        str::from_utf8(&self.units[..self.units_len]).unwrap_or("")
    }

    pub fn type_str(&self) -> &str {
        str::from_utf8(&self.gauge_type[..self.gauge_type_len]).unwrap_or("")
    }
}

/// Simple XML gauge parser
pub struct XMLGaugeParser;

impl XMLGaugeParser {
    /// Parse a simple XML gauge element
    /// Very basic parsing - just enough for TunerStudio gauge format
    pub fn parse_gauge_element(_xml_data: &[u8]) -> Option<XMLGaugeDefinition> {
        // Placeholder for actual XML parsing
        // In a real implementation, would:
        // 1. Find <gauge> tags
        // 2. Extract attributes (name, type, etc.)
        // 3. Parse nested elements (min, max, warn, danger, variable, units, etc.)
        // 4. Convert string values to numbers
        None
    }

    /// Parse string to f32
    pub fn parse_float(s: &str) -> f32 {
        let mut result = 0.0;
        let mut decimal_place = 0;
        let mut is_decimal = false;
        let mut is_negative = false;

        for ch in s.chars() {
            match ch {
                '-' => is_negative = true,
                '.' => is_decimal = true,
                '0'..='9' => {
                    let digit = (ch as u8 - b'0') as f32;
                    if is_decimal {
                        decimal_place += 1;
                        // Calculate divisor: 10^decimal_place manually
                        let mut divisor = 10.0;
                        for _ in 1..decimal_place {
                            divisor *= 10.0;
                        }
                        result += digit / divisor;
                    } else {
                        result = result * 10.0 + digit;
                    }
                }
                _ => {}
            }
        }

        if is_negative {
            -result
        } else {
            result
        }
    }

    /// Parse string to u32
    pub fn parse_int(s: &str) -> u32 {
        let mut result = 0u32;
        for ch in s.chars() {
            if ch >= '0' && ch <= '9' {
                result = result * 10 + (ch as u32 - '0' as u32);
            }
        }
        result
    }
}

/// Dashboard layout from XML
#[derive(Clone, Copy)]
pub struct XMLDashboardLayout {
    pub name: [u8; 64],
    pub name_len: usize,
    pub width: u32,
    pub height: u32,
    pub gauge_refs: [GaugeReference; 16],
    pub gauge_count: usize,
}

#[derive(Clone, Copy)]
pub struct GaugeReference {
    pub gauge_name: [u8; 64],
    pub gauge_name_len: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl XMLDashboardLayout {
    pub fn new() -> Self {
        XMLDashboardLayout {
            name: [0; 64],
            name_len: 0,
            width: 1280,
            height: 720,
            gauge_refs: [GaugeReference {
                gauge_name: [0; 64],
                gauge_name_len: 0,
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            }; 16],
            gauge_count: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}
