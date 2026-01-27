// Simple math functions for no_std
// Bare minimum for gauge rendering

/// Fast sine approximation using Taylor series
pub fn sin(mut x: f32) -> f32 {
    // Normalize to -PI to PI
    const PI: f32 = 3.14159265359;
    const TWO_PI: f32 = 6.28318530718;
    
    while x > PI {
        x -= TWO_PI;
    }
    while x < -PI {
        x += TWO_PI;
    }
    
    // Taylor series approximation
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    
    x - (x3 / 6.0) + (x5 / 120.0) - (x7 / 5040.0)
}

/// Cosine using sin(x + PI/2)
pub fn cos(x: f32) -> f32 {
    sin(x + 1.5707963268) // PI/2
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
