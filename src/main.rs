#![no_std]
#![no_main]

mod boot;
mod framebuffer_config;
mod framebuffer;
mod dashboard;
mod mmio;
mod ts_ini_parser;
mod ts_gauge;
mod uart;
// mod megasquirt;
mod math;
mod fatfs;
mod mock_ecu;
mod xml_parser;
mod config_loader;
mod colors;
mod digit_renderer;

use core::panic::PanicInfo;
use framebuffer::Framebuffer;
use framebuffer_config::FramebufferConfig;
use mock_ecu::MockECU;
use config_loader::DashboardConfig;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // Initialize UART for debug output
    uart::uart_init();
    uart::uart_puts("\n=== LibreDash Boot v0.1 ===\n");

    // Detect environment and get framebuffer configuration
    let fb_config = FramebufferConfig::detect();
    uart::uart_puts("Framebuffer mode: ");
    uart::uart_puts(fb_config.mode_name());
    uart::uart_puts("\n");

    // Initialize framebuffer with detected address
    uart::uart_puts("Initializing framebuffer...\n");
    let mut fb = Framebuffer::new(fb_config.address, fb_config.width, fb_config.height);
    uart::uart_puts("Framebuffer: ");
    uart::uart_puts("1280x720 @ 0x");
    uart::uart_puts(&format_hex_str(fb_config.address));
    uart::uart_puts("\n");

    // Draw test pattern
    uart::uart_puts("Drawing test pattern...\n");
    test_display_pattern(&mut fb);
    uart::uart_puts("Display pattern rendered\n");

    // Hex dump first 64 pixels
    uart::uart_puts("First 64 pixels:\n");
    uart::uart_hex_dump(fb.buffer_ptr(), 256);

    // Main loop with heartbeat
    let mut counter: u32 = 0;
    loop {
        counter += 1;

        // Print heartbeat every ~5 seconds
        if counter % 500_000 == 0 {
            uart::uart_puts("Running...\n");
        }

        // Spin delay
        for _ in 0..10_000 {
            unsafe {
                let dummy = 0u32;
                core::ptr::read_volatile(&dummy);
            }
        }
    }
}

/// Generate resolution-scaled test pattern
fn test_display_pattern(fb: &mut Framebuffer) {
    let w = fb.width();
    let h = fb.height();

    // Fill with gradient background (top: white, bottom: gray)
    for y in 0..h {
        let intensity = 255 - ((y * 128) / h) as u8;
        let gray_color = ((intensity as u32) << 16) | ((intensity as u32) << 8) | (intensity as u32);
        for x in 0..w {
            fb.draw_pixel(x, y, gray_color);
        }
    }

    // Draw 4 colored corner rectangles (100x100)
    fb.draw_filled_rect(0, 0, 100, 100, framebuffer::COLOR_RED);
    fb.draw_filled_rect(w - 100, 0, 100, 100, framebuffer::COLOR_GREEN);
    fb.draw_filled_rect(0, h - 100, 100, 100, framebuffer::COLOR_BLUE);
    fb.draw_filled_rect(w - 100, h - 100, 100, 100, framebuffer::COLOR_YELLOW);

    // Draw white grid (8 columns x 4 rows)
    for i in 0..=8 {
        let x = i * (w / 8);
        fb.draw_filled_rect(x, 0, 2, h, framebuffer::COLOR_WHITE);
    }
    for i in 0..=4 {
        let y = i * (h / 4);
        fb.draw_filled_rect(0, y, w, 2, framebuffer::COLOR_WHITE);
    }
}

fn format_hex_str(val: u32) -> &'static str {
    // Convert value to hex string (simple 8-char hex)
    // For now return a static string representation
    match val {
        0x04000000 => "04000000",
        0xC0000000 => "c0000000",
        _ => "????????",
    }
}

fn get_var_name(_config: &ts_ini_parser::GaugeConfig) -> Option<&str> {
    None
}
