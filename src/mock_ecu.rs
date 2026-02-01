/// Mock ECU data generator for testing without real MegaSquirt hardware
/// Provides realistic simulated engine parameters using mathematical patterns
/// Can be easily replaced with rusEFI simulator or real ECU data later

use core::f32::consts::PI;
use crate::math::sin;

pub struct MockECU {
    pub time_ms: u32,
    pub frame_count: u32,
}

#[derive(Clone, Copy)]
pub struct MockECUData {
    pub rpm: f32,
    pub map_pressure: f32,
    pub coolant_temp: f32,
    pub intake_temp: f32,
    pub air_fuel_ratio: f32,
    pub oil_pressure: f32,
    pub fuel_pressure: f32,
    pub battery_voltage: f32,
    pub throttle_position: f32,
    pub boost_pressure: f32,
    pub ignition_advance: f32,
    pub injector_duty: f32,
    pub vehicle_speed: f32,
}

impl MockECUData {
    pub fn new() -> Self {
        MockECUData {
            rpm: 0.0,
            map_pressure: 0.0,
            coolant_temp: 70.0,
            intake_temp: 65.0,
            air_fuel_ratio: 14.7,
            oil_pressure: 30.0,
            fuel_pressure: 45.0,
            battery_voltage: 13.5,
            throttle_position: 0.0,
            boost_pressure: 0.0,
            ignition_advance: 15.0,
            injector_duty: 0.0,
            vehicle_speed: 0.0,
        }
    }
}

impl MockECU {
    pub fn new() -> Self {
        MockECU {
            time_ms: 0,
            frame_count: 0,
        }
    }

    /// Update mock data with realistic patterns
    /// Simulates engine startup, idle, acceleration, and load scenarios
    pub fn update(&mut self, delta_ms: u32) -> MockECUData {
        self.time_ms += delta_ms;
        self.frame_count += 1;

        let t = (self.time_ms as f32) / 1000.0; // Convert to seconds
        let mut data = MockECUData::new();

        // Scenario: Simulate engine startup -> idle -> rev cycle
        // Phase 1 (0-3s): Engine starts, RPM rises
        // Phase 2 (3-8s): Idle at ~1000 RPM
        // Phase 3 (8-13s): Acceleration to 5000 RPM
        // Phase 4 (13-18s): High load at 6000 RPM
        // Then repeat
        let cycle_time = t % 18.0;

        // RPM pattern: startup -> idle -> accel -> cruise -> repeat
        data.rpm = if cycle_time < 3.0 {
            // Startup phase: 0 -> 1200 RPM
            (cycle_time / 3.0) * 1200.0 + sin(cycle_time * 3.0) * 100.0
        } else if cycle_time < 8.0 {
            // Idle phase: ~1000 RPM with small oscillations
            1000.0 + sin((cycle_time - 3.0) * 2.0) * 50.0
        } else if cycle_time < 13.0 {
            // Acceleration: 1000 -> 5000 RPM
            let accel_phase = (cycle_time - 8.0) / 5.0;
            1000.0 + accel_phase * 4000.0 + sin(accel_phase * 4.0 * PI) * 200.0
        } else {
            // Cruise at 6000 RPM
            6000.0 + sin((cycle_time - 13.0) * 1.5) * 150.0
        };

        // MAP pressure (manifold absolute pressure) varies with load
        let load_factor = (data.rpm / 6500.0).min(1.0);
        data.map_pressure = 20.0 + load_factor * 80.0 + sin(t * 0.5) * 5.0;

        // Coolant temperature: gradual warmup from 70°F to 190°F
        let warmup = if t < 20.0 { t / 20.0 } else { 1.0 };
        data.coolant_temp = 70.0 + warmup * 120.0 + sin(t * 0.3) * 3.0;

        // Intake temperature: follows coolant with offset
        data.intake_temp = data.coolant_temp - 5.0 + sin(t * 0.7) * 2.0;

        // Air/fuel ratio: lean at cruise, rich under load
        let afr_base = 14.7 + (1.0 - load_factor) * 1.0; // Leaner at higher load
        data.air_fuel_ratio = afr_base + sin(t * 1.2) * 0.3;

        // Oil pressure: increases with RPM
        let oil_rpm_factor = (data.rpm / 7000.0).min(1.0);
        data.oil_pressure = 20.0 + oil_rpm_factor * 50.0 + sin(t * 0.4) * 2.0;

        // Fuel pressure: varies slightly with load
        data.fuel_pressure = 40.0 + load_factor * 10.0 + sin(t * 1.5) * 1.0;

        // Battery voltage: drops under heavy load, recovers at idle
        data.battery_voltage = 13.5 - load_factor * 0.8 + sin(t * 0.2) * 0.1;

        // Throttle position: cycles through 0-100%
        data.throttle_position = if cycle_time < 8.0 {
            5.0 + sin((cycle_time - 3.0) * 0.5) * 2.0 // Small oscillations at idle
        } else if cycle_time < 13.0 {
            let accel = (cycle_time - 8.0) / 5.0;
            accel * 100.0
        } else {
            70.0 + sin((cycle_time - 13.0) * 2.0) * 10.0
        };

        // Boost pressure (for turbocharged): only at high load
        if load_factor > 0.6 {
            data.boost_pressure = (load_factor - 0.6) * 25.0 + sin(t * 1.0) * 1.0;
        } else {
            data.boost_pressure = sin(t * 0.5) * 0.5; // Slight vacuum at cruise
        }

        // Ignition advance: varies with load
        data.ignition_advance = 15.0 + (1.0 - load_factor) * 10.0 + sin(t * 0.8) * 1.0;

        // Injector duty cycle: proportional to load
        data.injector_duty = load_factor * 95.0 + sin(t * 2.0) * 3.0;

        // Vehicle speed: proportional to RPM and throttle
        let speed_factor = (data.rpm / 7000.0) * (data.throttle_position / 100.0);
        data.vehicle_speed = speed_factor * 150.0 + sin(t * 0.3) * 2.0;

        // Clamp values to realistic ranges
        data.rpm = data.rpm.max(0.0).min(8500.0);
        data.map_pressure = data.map_pressure.max(0.0).min(250.0);
        data.coolant_temp = data.coolant_temp.max(60.0).min(250.0);
        data.intake_temp = data.intake_temp.max(50.0).min(200.0);
        data.air_fuel_ratio = data.air_fuel_ratio.max(10.0).min(20.0);
        data.oil_pressure = data.oil_pressure.max(0.0).min(100.0);
        data.fuel_pressure = data.fuel_pressure.max(0.0).min(100.0);
        data.battery_voltage = data.battery_voltage.max(10.0).min(16.0);
        data.throttle_position = data.throttle_position.max(0.0).min(100.0);
        data.boost_pressure = data.boost_pressure.max(-15.0).min(30.0);
        data.ignition_advance = data.ignition_advance.max(-10.0).min(50.0);
        data.injector_duty = data.injector_duty.max(0.0).min(100.0);
        data.vehicle_speed = data.vehicle_speed.max(0.0).min(200.0);

        data
    }

    /// Get current mock data without advancing time
    pub fn get_current(&self) -> MockECUData {
        let t = (self.time_ms as f32) / 1000.0;
        let cycle_time = t % 18.0;

        let mut data = MockECUData::new();

        // RPM calculation (same as update)
        data.rpm = if cycle_time < 3.0 {
            (cycle_time / 3.0) * 1200.0 + sin(cycle_time * 3.0) * 100.0
        } else if cycle_time < 8.0 {
            1000.0 + sin((cycle_time - 3.0) * 2.0) * 50.0
        } else if cycle_time < 13.0 {
            let accel_phase = (cycle_time - 8.0) / 5.0;
            1000.0 + accel_phase * 4000.0 + sin(accel_phase * 4.0 * PI) * 200.0
        } else {
            6000.0 + sin((cycle_time - 13.0) * 1.5) * 150.0
        };

        let load_factor = (data.rpm / 6500.0).min(1.0);
        data.map_pressure = 20.0 + load_factor * 80.0 + sin(t * 0.5) * 5.0;
        data.coolant_temp = 70.0 + if t < 20.0 { t / 20.0 } else { 1.0 } * 120.0 + sin(t * 0.3) * 3.0;
        data.intake_temp = data.coolant_temp - 5.0 + sin(t * 0.7) * 2.0;
        data.air_fuel_ratio = 14.7 + (1.0 - load_factor) * 1.0 + sin(t * 1.2) * 0.3;
        data.oil_pressure = 20.0 + load_factor * 50.0 + sin(t * 0.4) * 2.0;
        data.fuel_pressure = 40.0 + load_factor * 10.0 + sin(t * 1.5) * 1.0;
        data.battery_voltage = 13.5 - load_factor * 0.8 + sin(t * 0.2) * 0.1;
        data.throttle_position = if cycle_time < 8.0 {
            5.0 + sin((cycle_time - 3.0) * 0.5) * 2.0
        } else if cycle_time < 13.0 {
            ((cycle_time - 8.0) / 5.0) * 100.0
        } else {
            70.0 + sin((cycle_time - 13.0) * 2.0) * 10.0
        };
        
        if load_factor > 0.6 {
            data.boost_pressure = (load_factor - 0.6) * 25.0 + sin(t * 1.0) * 1.0;
        }

        data.ignition_advance = 15.0 + (1.0 - load_factor) * 10.0 + sin(t * 0.8) * 1.0;
        data.injector_duty = load_factor * 95.0 + sin(t * 2.0) * 3.0;
        let speed_factor = (data.rpm / 7000.0) * (data.throttle_position / 100.0);
        data.vehicle_speed = speed_factor * 150.0 + sin(t * 0.3) * 2.0;

        // Clamp values
        data.rpm = data.rpm.max(0.0).min(8500.0);
        data.map_pressure = data.map_pressure.max(0.0).min(250.0);
        data.coolant_temp = data.coolant_temp.max(60.0).min(250.0);
        data.intake_temp = data.intake_temp.max(50.0).min(200.0);
        data.air_fuel_ratio = data.air_fuel_ratio.max(10.0).min(20.0);
        data.oil_pressure = data.oil_pressure.max(0.0).min(100.0);
        data.fuel_pressure = data.fuel_pressure.max(0.0).min(100.0);
        data.battery_voltage = data.battery_voltage.max(10.0).min(16.0);
        data.throttle_position = data.throttle_position.max(0.0).min(100.0);
        data.boost_pressure = data.boost_pressure.max(-15.0).min(30.0);
        data.ignition_advance = data.ignition_advance.max(-10.0).min(50.0);
        data.injector_duty = data.injector_duty.max(0.0).min(100.0);
        data.vehicle_speed = data.vehicle_speed.max(0.0).min(200.0);

        data
    }
}
