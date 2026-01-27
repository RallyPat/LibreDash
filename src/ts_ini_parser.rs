// TunerStudio INI file parser for gauge configurations
// Based on TunerStudio ECU Definition file specification

use core::str;

/// Maximum number of gauge configurations
pub const MAX_GAUGE_CONFIGS: usize = 64;

/// Gauge configuration from TunerStudio INI [GaugeConfigurations] section
/// Format: name = var, "title", "units", lo, hi, loD, loW, hiW, hiD, vd, ld
#[derive(Copy, Clone, Debug)]
pub struct GaugeConfig {
    /// Gauge name (internal identifier)
    pub name: [u8; 64],
    /// Variable name from OutputChannels
    pub var: [u8; 64],
    /// Title displayed on gauge
    pub title: [u8; 64],
    /// Units label (e.g., "RPM", "PSI")
    pub units: [u8; 16],
    /// Lower scale limit
    pub lo: f32,
    /// Upper scale limit
    pub hi: f32,
    /// Lower danger threshold
    pub lo_danger: f32,
    /// Lower warning threshold
    pub lo_warning: f32,
    /// Upper warning threshold
    pub hi_warning: f32,
    /// Upper danger threshold
    pub hi_danger: f32,
    /// Value decimal places
    pub value_decimals: u8,
    /// Label decimal places
    pub label_decimals: u8,
}

impl GaugeConfig {
    pub fn new() -> Self {
        GaugeConfig {
            name: [0; 64],
            var: [0; 64],
            title: [0; 64],
            units: [0; 16],
            lo: 0.0,
            hi: 100.0,
            lo_danger: 0.0,
            lo_warning: 0.0,
            hi_warning: 80.0,
            hi_danger: 90.0,
            value_decimals: 0,
            label_decimals: 0,
        }
    }
    
    /// Get name as string slice
    pub fn name_str(&self) -> &str {
        str_from_bytes(&self.name)
    }
    
    /// Get variable name as string slice
    pub fn var_str(&self) -> &str {
        str_from_bytes(&self.var)
    }
    
    /// Get title as string slice
    pub fn title_str(&self) -> &str {
        str_from_bytes(&self.title)
    }
    
    /// Get units as string slice
    pub fn units_str(&self) -> &str {
        str_from_bytes(&self.units)
    }
}

impl Default for GaugeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to convert byte array to string slice
fn str_from_bytes(bytes: &[u8]) -> &str {
    // Find null terminator
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    core::str::from_utf8(&bytes[..len]).unwrap_or("")
}

/// Copy string to byte array, null-terminated
fn copy_str_to_bytes(dest: &mut [u8], src: &str) {
    let bytes = src.as_bytes();
    let len = bytes.len().min(dest.len() - 1);
    dest[..len].copy_from_slice(&bytes[..len]);
    if len < dest.len() {
        dest[len] = 0;
    }
}

/// Parse a simple float from string (bare metal, no std)
fn parse_f32(s: &str) -> f32 {
    let s = s.trim();
    let mut result: f32 = 0.0;
    let mut is_negative = false;
    let mut is_fraction = false;
    let mut fraction_divisor: f32 = 10.0;
    
    for c in s.chars() {
        match c {
            '-' => is_negative = true,
            '.' => is_fraction = true,
            '0'..='9' => {
                let digit = (c as u8 - b'0') as f32;
                if is_fraction {
                    result += digit / fraction_divisor;
                    fraction_divisor *= 10.0;
                } else {
                    result = result * 10.0 + digit;
                }
            }
            _ => break,
        }
    }
    
    if is_negative {
        -result
    } else {
        result
    }
}

/// Parse a simple u8 from string
fn parse_u8(s: &str) -> u8 {
    let s = s.trim();
    let mut result: u8 = 0;
    
    for c in s.chars() {
        if let Some(digit) = c.to_digit(10) {
            result = result * 10 + digit as u8;
        } else {
            break;
        }
    }
    
    result
}

/// Parse a gauge configuration line from INI file
/// Format: name = var, "title", "units", lo, hi, loD, loW, hiW, hiD, vd, ld
pub fn parse_gauge_line(line: &str) -> Option<GaugeConfig> {
    let line = line.trim();
    
    // Skip comments and empty lines
    if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
        return None;
    }
    
    // Split on '='
    let mut parts = [("", 0usize); 2];
    let mut part_count = 0;
    
    if let Some(eq_pos) = line.find('=') {
        parts[0] = (&line[..eq_pos], eq_pos);
        parts[1] = (&line[eq_pos + 1..], line.len() - eq_pos - 1);
        part_count = 2;
    }
    
    if part_count != 2 {
        return None;
    }
    
    let mut config = GaugeConfig::new();
    copy_str_to_bytes(&mut config.name, parts[0].0.trim());
    
    // Parse right side (comma-separated)
    let values = parts[1].0;
    let mut field_index = 0;
    let mut in_quotes = false;
    let mut current_field = String::<64>::new();
    
    for c in values.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c == ',' && !in_quotes {
            process_field(&mut config, field_index, current_field.as_str());
            field_index += 1;
            current_field.clear();
        } else if !in_quotes || c != '"' {
            let _ = current_field.push(c);
        }
    }
    
    // Process last field
    process_field(&mut config, field_index, current_field.as_str());
    
    Some(config)
}

/// Simple fixed-size string for no_std
struct String<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> String<N> {
    fn new() -> Self {
        String { data: [0; N], len: 0 }
    }
    
    fn push(&mut self, c: char) -> Result<(), ()> {
        if self.len < N {
            self.data[self.len] = c as u8;
            self.len += 1;
            Ok(())
        } else {
            Err(())
        }
    }
    
    fn clear(&mut self) {
        self.len = 0;
    }
    
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }
}

fn process_field(config: &mut GaugeConfig, index: usize, value: &str) {
    let value = value.trim();
    
    match index {
        0 => copy_str_to_bytes(&mut config.var, value),
        1 => copy_str_to_bytes(&mut config.title, value),
        2 => copy_str_to_bytes(&mut config.units, value),
        3 => config.lo = parse_f32(value),
        4 => config.hi = parse_f32(value),
        5 => config.lo_danger = parse_f32(value),
        6 => config.lo_warning = parse_f32(value),
        7 => config.hi_warning = parse_f32(value),
        8 => config.hi_danger = parse_f32(value),
        9 => config.value_decimals = parse_u8(value),
        10 => config.label_decimals = parse_u8(value),
        _ => {}
    }
}

/// Collection of gauge configurations
pub struct GaugeConfigurations {
    configs: [Option<GaugeConfig>; MAX_GAUGE_CONFIGS],
    count: usize,
}

impl GaugeConfigurations {
    pub fn new() -> Self {
        GaugeConfigurations {
            configs: [None; MAX_GAUGE_CONFIGS],
            count: 0,
        }
    }
    
    /// Add a gauge configuration
    pub fn add(&mut self, config: GaugeConfig) -> bool {
        if self.count < MAX_GAUGE_CONFIGS {
            self.configs[self.count] = Some(config);
            self.count += 1;
            true
        } else {
            false
        }
    }
    
    /// Get gauge configuration by name
    pub fn get_by_name(&self, name: &str) -> Option<&GaugeConfig> {
        for i in 0..self.count {
            if let Some(ref config) = self.configs[i] {
                if config.name_str() == name {
                    return Some(config);
                }
            }
        }
        None
    }
    
    /// Get gauge configuration by index
    pub fn get(&self, index: usize) -> Option<&GaugeConfig> {
        if index < self.count {
            self.configs[index].as_ref()
        } else {
            None
        }
    }
    
    /// Get number of configurations
    pub fn len(&self) -> usize {
        self.count
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl Default for GaugeConfigurations {
    fn default() -> Self {
        Self::new()
    }
}
