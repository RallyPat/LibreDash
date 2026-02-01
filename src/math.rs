/// Math utilities for bare-metal graphics and calculations
/// Includes trigonometric functions and basic arithmetic

/// Sine approximation using Taylor series
/// Input: radians
pub fn sin(x: f32) -> f32 {
    // Normalize angle to -PI to PI
    const PI: f32 = 3.141592653589793;
    const TWO_PI: f32 = 2.0 * PI;
    
    let mut x = x;
    while x > PI {
        x -= TWO_PI;
    }
    while x < -PI {
        x += TWO_PI;
    }

    // Taylor series approximation
    let x2 = x * x;
    let x3 = x * x2;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    let x9 = x7 * x2;

    x - x3 / 6.0 + x5 / 120.0 - x7 / 5040.0 + x9 / 362880.0
}

/// Cosine approximation using sin
pub fn cos(x: f32) -> f32 {
    const PI: f32 = 3.141592653589793;
    sin(x + PI / 2.0)
}

/// Absolute value
pub fn abs(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// Maximum of two values
pub fn max(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}

/// Minimum of two values
pub fn min(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}

/// Square root approximation using Newton's method
pub fn sqrt(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    
    let mut z = x;
    let mut prev_z = 0.0;
    
    // Newton's method iterations
    for _ in 0..10 {
        z = (z + x / z) / 2.0;
        if abs(z - prev_z) < 0.0001 {
            break;
        }
        prev_z = z;
    }
    
    z
}

/// Clamp value between min and max
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min { min } else if value > max { max } else { value }
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
