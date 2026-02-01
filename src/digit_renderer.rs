/// Simple 7-segment style digit renderer for digital displays
/// Renders 0-9, colon, and minus sign using rectangular segments

use crate::framebuffer::Framebuffer;
use crate::colors::Color;

/// Render a single digit (0-9) at the given position
/// Size controls the pixel size of each segment
pub fn draw_digit(fb: &mut Framebuffer, digit: u8, x: u32, y: u32, size: u32, color: Color) {
    if digit > 9 {
        return; // Only support 0-9
    }

    let digit_val = digit as usize;
    
    // 7-segment layout (a-g):
    //   aaa
    //  f   b
    //   ggg
    //  e   c
    //   ddd
    
    let digits = [
        [true,  true,  true,  false, true,  true,  true],  // 0
        [false, true,  true,  false, false, false, false], // 1
        [true,  true,  false, true,  true,  false, true],  // 2
        [true,  true,  true,  true,  false, false, true],  // 3
        [false, true,  true,  true,  false, true,  false], // 4
        [true,  false, true,  true,  false, true,  true],  // 5
        [true,  false, true,  true,  true,  true,  true],  // 6
        [true,  true,  true,  false, false, false, false], // 7
        [true,  true,  true,  true,  true,  true,  true],  // 8
        [true,  true,  true,  true,  false, true,  true],  // 9
    ];

    let segs = digits[digit_val];
    
    // Calculate positions for each segment
    let thickness = size / 2;
    
    // Horizontal segments (a, g, d)
    if segs[0] {
        // Segment a (top)
        fb.draw_filled_rect(x + thickness, y, size, thickness, color.to_u32());
    }
    if segs[6] {
        // Segment g (middle)
        fb.draw_filled_rect(x + thickness, y + size, size, thickness, color.to_u32());
    }
    if segs[3] {
        // Segment d (bottom)
        fb.draw_filled_rect(x + thickness, y + size * 2, size, thickness, color.to_u32());
    }
    
    // Vertical segments (b, c, e, f)
    if segs[1] {
        // Segment b (top-right)
        fb.draw_filled_rect(x + size + thickness, y + thickness, thickness, size, color.to_u32());
    }
    if segs[2] {
        // Segment c (bottom-right)
        fb.draw_filled_rect(x + size + thickness, y + size + thickness, thickness, size, color.to_u32());
    }
    if segs[4] {
        // Segment e (bottom-left)
        fb.draw_filled_rect(x, y + size + thickness, thickness, size, color.to_u32());
    }
    if segs[5] {
        // Segment f (top-left)
        fb.draw_filled_rect(x, y + thickness, thickness, size, color.to_u32());
    }
}

/// Render a decimal number with specified number of digits
/// Handles negative numbers with minus sign
pub fn draw_number(
    fb: &mut Framebuffer,
    mut value: i32,
    max_digits: u32,
    x: u32,
    y: u32,
    digit_size: u32,
    color: Color,
) {
    let is_negative = value < 0;
    if is_negative {
        value = -value;
    }
    
    // Convert to digits
    let mut digits = [0u8; 8];
    let mut digit_count = 0;
    
    if value == 0 {
        digits[0] = 0;
        digit_count = 1;
    } else {
        let mut temp = value as u32;
        while temp > 0 && digit_count < 8 {
            digits[digit_count] = (temp % 10) as u8;
            temp /= 10;
            digit_count += 1;
        }
        // Reverse to get correct order
        for i in 0..digit_count / 2 {
            let j = digit_count - 1 - i;
            let tmp = digits[i];
            digits[i] = digits[j];
            digits[j] = tmp;
        }
    }
    
    // Add leading spaces if needed
    let digit_width = digit_size + digit_size / 2; // Size + gap between digits
    let mut current_x = x;
    
    if is_negative {
        // Draw minus sign
        let thickness = digit_size / 3;
        fb.draw_filled_rect(current_x + digit_size / 4, y + digit_size, digit_size / 2, thickness, color.to_u32());
        current_x += digit_width;
    }
    
    // Draw digits
    for i in 0..digit_count.min(max_digits as usize) {
        draw_digit(fb, digits[i], current_x, y, digit_size, color);
        current_x += digit_width;
    }
}

/// Render a floating point number as digital display
pub fn draw_float(
    fb: &mut Framebuffer,
    value: f32,
    integer_digits: u32,
    decimal_digits: u32,
    x: u32,
    y: u32,
    digit_size: u32,
    color: Color,
) {
    let is_negative = value < 0.0;
    let abs_val = if is_negative { -value } else { value };
    
    // Split into integer and decimal parts
    let integer_part = abs_val as i32;
    
    // Calculate 10^decimal_digits manually without powi
    let mut multiplier = 1.0_f32;
    for _ in 0..decimal_digits {
        multiplier *= 10.0;
    }
    let decimal_part = ((abs_val - integer_part as f32) * multiplier) as i32;
    
    let digit_width = digit_size + digit_size / 2;
    let mut current_x = x;
    
    // Draw minus sign if needed
    if is_negative {
        let thickness = digit_size / 3;
        fb.draw_filled_rect(current_x + digit_size / 4, current_x + digit_size, digit_size / 2, thickness, color.to_u32());
        current_x += digit_width;
    }
    
    // Draw integer part
    draw_number(fb, integer_part, integer_digits, current_x, y, digit_size, color);
    current_x += integer_digits * digit_width;
    
    // Draw decimal point
    if decimal_digits > 0 {
        let dot_size = digit_size / 4;
        fb.draw_filled_rect(current_x, y + digit_size * 2 - dot_size, dot_size, dot_size, color.to_u32());
        current_x += digit_width / 2;
        
        // Draw decimal digits
        draw_number(fb, decimal_part, decimal_digits, current_x, y, digit_size, color);
    }
}
