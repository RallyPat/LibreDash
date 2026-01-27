#![no_std]
#![no_main]

mod boot;
mod framebuffer;
mod dashboard;
mod mmio;

use core::panic::PanicInfo;
use framebuffer::Framebuffer;
use dashboard::{Dashboard, DashElement, DashElementType};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // Initialize framebuffer
    let mut fb = Framebuffer::new(1280, 720);
    
    // Initialize dashboard
    let mut dash = Dashboard::new("LibreDash Demo");
    
    // Add dashboard elements
    dash.add_element(DashElement {
        element_type: DashElementType::Gauge,
        x: 50,
        y: 50,
        width: 400,
        height: 60,
        color: 0x00FF00, // Green
        label: *b"Speed                                                           ",
        value: 75.0,
        min_value: 0.0,
        max_value: 100.0,
    });
    
    dash.add_element(DashElement {
        element_type: DashElementType::Gauge,
        x: 50,
        y: 130,
        width: 400,
        height: 60,
        color: 0x0000FF, // Blue
        label: *b"RPM                                                             ",
        value: 45.0,
        min_value: 0.0,
        max_value: 100.0,
    });
    
    dash.add_element(DashElement {
        element_type: DashElementType::Value,
        x: 500,
        y: 50,
        width: 200,
        height: 60,
        color: 0x00FFFF, // Cyan
        label: *b"Temp                                                            ",
        value: 65.0,
        min_value: 0.0,
        max_value: 100.0,
    });
    
    dash.add_element(DashElement {
        element_type: DashElementType::Graph,
        x: 50,
        y: 250,
        width: 650,
        height: 200,
        color: 0xFFFF00, // Yellow
        label: *b"History                                                         ",
        value: 0.0,
        min_value: 0.0,
        max_value: 100.0,
    });
    
    dash.add_element(DashElement {
        element_type: DashElementType::Label,
        x: 50,
        y: 500,
        width: 650,
        height: 50,
        color: 0xFFFFFF, // White
        label: *b"LibreDash - Raspberry Pi Bare Metal Dashboard                   ",
        value: 0.0,
        min_value: 0.0,
        max_value: 0.0,
    });
    
    // Animation loop
    let mut counter: f32 = 0.0;
    loop {
        counter += 0.5;
        if counter > 100.0 {
            counter = 0.0;
        }
        
        // Update values
        dash.update_value(0, counter);
        dash.update_value(1, 100.0 - counter);
        dash.update_value(2, counter * 0.7);
        
        // Render
        dash.render(&mut fb);
        
        // Simple delay
        for _ in 0..1_000_000 {
            unsafe { 
                let dummy = 0u32;
                core::ptr::read_volatile(&dummy);
            }
        }
    }
}
