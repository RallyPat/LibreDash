#![no_std]
#![no_main]

mod boot;
mod framebuffer;
mod dashboard;
mod mmio;
mod ts_ini_parser;
mod ts_gauge;
mod uart;
mod megasquirt;
mod math;

use core::panic::PanicInfo;
use framebuffer::Framebuffer;
use ts_ini_parser::GaugeConfig;
use ts_gauge::{TSGauge, TSGaugeStyle};
use megasquirt::{MegaSquirt, ECUData};
use math::sin;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Copy string to byte array
fn copy_str(dest: &mut [u8], src: &str) {
    let bytes = src.as_bytes();
    let len = bytes.len().min(dest.len() - 1);
    dest[..len].copy_from_slice(&bytes[..len]);
    if len < dest.len() {
        dest[len] = 0;
    }
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // ========== FAST BOOT PRIORITY ==========
    // 1. Initialize framebuffer FIRST for visual feedback
    let mut fb = Framebuffer::new(1280, 720);
    fb.clear(0x000000);
    
    // Draw boot message (simple colored rectangles for "BOOTING...")
    fb.draw_filled_rect(100, 300, 1080, 120, 0x003300);
    fb.draw_rect(100, 300, 1080, 120, 0x00FF00);
    
    // 2. Initialize ECU communication immediately
    let mut ms = MegaSquirt::new();
    let mut ecu_data = ECUData::new();
    
    // Try to connect at 115200 baud (fastest standard rate)
    // Fall back to lower rates if needed
    let baud_rates = [115200, 57600, 38400, 19200];
    let mut connected = false;
    
    for &baud in baud_rates.iter() {
        if ms.connect(baud) {
            connected = true;
            // Show connection success
            fb.draw_filled_rect(100, 300, 1080, 120, 0x003300);
            fb.draw_filled_rect(100, 300, 540, 120, 0x00FF00); // Half green = connected
            break;
        }
    }
    
    // 3. Setup gauges while ECU warms up
    let mut rpm_config = GaugeConfig::new();
    copy_str(&mut rpm_config.name, "tachometer");
    copy_str(&mut rpm_config.var, "rpm");
    copy_str(&mut rpm_config.title, "RPM");
    copy_str(&mut rpm_config.units, "");
    rpm_config.lo = 0.0;
    rpm_config.hi = 8000.0;
    rpm_config.lo_danger = 0.0;
    rpm_config.lo_warning = 300.0;
    rpm_config.hi_warning = 6500.0;
    rpm_config.hi_danger = 7000.0;
    
    let mut map_config = GaugeConfig::new();
    copy_str(&mut map_config.name, "map");
    copy_str(&mut map_config.var, "map");
    copy_str(&mut map_config.title, "MAP");
    copy_str(&mut map_config.units, "kPa");
    map_config.lo = 0.0;
    map_config.hi = 250.0;
    map_config.lo_danger = 0.0;
    map_config.lo_warning = 20.0;
    map_config.hi_warning = 220.0;
    map_config.hi_danger = 240.0;
    
    let mut coolant_config = GaugeConfig::new();
    copy_str(&mut coolant_config.name, "coolant");
    copy_str(&mut coolant_config.var, "coolantTemp");
    copy_str(&mut coolant_config.title, "Coolant");
    copy_str(&mut coolant_config.units, "°F");
    coolant_config.lo = 50.0;
    coolant_config.hi = 250.0;
    coolant_config.lo_danger = 50.0;
    coolant_config.lo_warning = 140.0;
    coolant_config.hi_warning = 220.0;
    coolant_config.hi_danger = 235.0;
    
    let mut tps_config = GaugeConfig::new();
    copy_str(&mut tps_config.name, "tps");
    copy_str(&mut tps_config.var, "tps");
    copy_str(&mut tps_config.title, "TPS");
    copy_str(&mut tps_config.units, "%");
    tps_config.lo = 0.0;
    tps_config.hi = 100.0;
    tps_config.lo_danger = 0.0;
    tps_config.lo_warning = 0.0;
    tps_config.hi_warning = 95.0;
    tps_config.hi_danger = 98.0;
    
    let mut afr_config = GaugeConfig::new();
    copy_str(&mut afr_config.name, "afr");
    copy_str(&mut afr_config.var, "afr");
    copy_str(&mut afr_config.title, "AFR");
    copy_str(&mut afr_config.units, "");
    afr_config.lo = 10.0;
    afr_config.hi = 20.0;
    afr_config.lo_danger = 10.5;
    afr_config.lo_warning = 12.5;
    afr_config.hi_warning = 16.0;
    afr_config.hi_danger = 18.0;
    
    let mut boost_config = GaugeConfig::new();
    copy_str(&mut boost_config.name, "boost");
    copy_str(&mut boost_config.var, "boost");
    copy_str(&mut boost_config.title, "Boost");
    copy_str(&mut boost_config.units, "PSI");
    boost_config.lo = -15.0;
    boost_config.hi = 30.0;
    boost_config.lo_danger = -10.0;
    boost_config.lo_warning = -5.0;
    boost_config.hi_warning = 25.0;
    boost_config.hi_danger = 28.0;
    
    // Create gauge instances optimized for quick viewing
    let mut gauges: [Option<TSGauge>; 6] = [None, None, None, None, None, None];
    
    // Large RPM gauge at top (most important)
    gauges[0] = Some(TSGauge::new(rpm_config, TSGaugeStyle::HorizontalBar, 50, 30, 1180, 100));
    
    // Circular gauges for key metrics
    gauges[1] = Some(TSGauge::new(coolant_config, TSGaugeStyle::Circular, 50, 160, 280, 280));
    gauges[2] = Some(TSGauge::new(boost_config, TSGaugeStyle::Circular, 360, 160, 280, 280));
    gauges[3] = Some(TSGauge::new(afr_config, TSGaugeStyle::Circular, 670, 160, 280, 280));
    
    // Bar gauges at bottom
    gauges[4] = Some(TSGauge::new(tps_config, TSGaugeStyle::HorizontalBar, 50, 480, 580, 60));
    gauges[5] = Some(TSGauge::new(map_config, TSGaugeStyle::HorizontalBar, 650, 480, 580, 60));
    
    // Clear boot screen
    fb.clear(0x000000);
    
    // ========== MAIN LOOP - OPTIMIZED FOR SPEED ==========
    let mut frame_counter: u32 = 0;
    
    loop {
        // Get ECU data every frame if connected
        if connected {
            if ms.get_realtime_data() {
                ecu_data.update_from_ms(&ms);
            }
        } else {
            // Not connected - show simulated data for testing
            frame_counter += 1;
            let t = frame_counter as f32 * 0.05;
            ecu_data.rpm = 1000.0 + (sin(t) * 3000.0 + 3000.0);
            ecu_data.coolant_temp = 180.0 + (sin(frame_counter as f32 * 0.01) * 20.0);
            ecu_data.boost = sin(frame_counter as f32 * 0.02) * 15.0;
            ecu_data.afr = 14.7 + (sin(frame_counter as f32 * 0.03) * 1.5);
            ecu_data.tps = (sin(frame_counter as f32 * 0.04) * 50.0 + 50.0).max(0.0);
            ecu_data.map = 100.0 + (sin(frame_counter as f32 * 0.02) * 50.0);
        }
        
        // Update gauge values from ECU data
        if let Some(ref mut gauge) = gauges[0] { gauge.set_value(ecu_data.rpm); }
        if let Some(ref mut gauge) = gauges[1] { gauge.set_value(ecu_data.coolant_temp); }
        if let Some(ref mut gauge) = gauges[2] { gauge.set_value(ecu_data.boost); }
        if let Some(ref mut gauge) = gauges[3] { gauge.set_value(ecu_data.afr); }
        if let Some(ref mut gauge) = gauges[4] { gauge.set_value(ecu_data.tps); }
        if let Some(ref mut gauge) = gauges[5] { gauge.set_value(ecu_data.map); }
        
        // Clear screen (fast)
        fb.clear(0x000000);
        
        // Render all gauges (optimized)
        for gauge_opt in gauges.iter() {
            if let Some(ref gauge) = gauge_opt {
                gauge.render(&mut fb);
            }
        }
        
        // Connection status indicator
        let status_color = if connected { 0x00FF00 } else { 0xFF0000 };
        fb.draw_filled_rect(10, 10, 30, 10, status_color);
        
        // Minimal delay - prioritize responsiveness
        // Only delay enough to avoid overwhelming the ECU
        for _ in 0..50_000 {
            unsafe { 
                let dummy = 0u32;
                core::ptr::read_volatile(&dummy);
            }
        }
    }
}
