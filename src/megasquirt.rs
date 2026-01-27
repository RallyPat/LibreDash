// MegaSquirt serial protocol implementation
// Fast, efficient ECU communication for real-time data

use crate::uart::Uart;

/// MegaSquirt command codes
const MS_CMD_SIGNATURE: u8 = b'S';
const MS_CMD_REALTIME: u8 = b'A';
const MS_CMD_TABLE: u8 = b'T';
const MS_CMD_REVISION: u8 = b'Q';

/// Maximum response size
const MAX_RESPONSE_SIZE: usize = 256;

/// Communication timeout (CPU cycles)
const TIMEOUT_CYCLES: u32 = 100000;

/// MegaSquirt ECU interface
pub struct MegaSquirt {
    uart: Uart,
    connected: bool,
    realtime_buffer: [u8; MAX_RESPONSE_SIZE],
    realtime_size: usize,
}

impl MegaSquirt {
    pub fn new() -> Self {
        MegaSquirt {
            uart: Uart::new(),
            connected: false,
            realtime_buffer: [0; MAX_RESPONSE_SIZE],
            realtime_size: 0,
        }
    }
    
    /// Initialize and connect to ECU (fast startup)
    pub fn connect(&mut self, baud_rate: u32) -> bool {
        // Initialize UART with specified baud rate
        // Common MegaSquirt baud rates: 9600, 19200, 38400, 57600, 115200
        self.uart.init(baud_rate);
        
        // Flush any pending data
        self.uart.flush_rx();
        
        // Try to get signature
        if self.get_signature().is_some() {
            self.connected = true;
            true
        } else {
            false
        }
    }
    
    /// Get ECU signature (for verification)
    pub fn get_signature(&mut self) -> Option<[u8; 32]> {
        self.uart.send_byte(MS_CMD_SIGNATURE);
        
        let mut sig = [0u8; 32];
        let received = self.uart.recv_bytes(&mut sig, TIMEOUT_CYCLES);
        
        if received == 32 {
            Some(sig)
        } else {
            None
        }
    }
    
    /// Request real-time data (fast, optimized for frequent calls)
    pub fn get_realtime_data(&mut self) -> bool {
        if !self.connected {
            return false;
        }
        
        // Send real-time data request
        self.uart.send_byte(MS_CMD_REALTIME);
        
        // Receive response
        // MS1/MS2: typically 22-119 bytes depending on version
        // MS3: can be larger
        let received = self.uart.recv_bytes(&mut self.realtime_buffer, TIMEOUT_CYCLES);
        
        if received > 0 {
            self.realtime_size = received;
            true
        } else {
            false
        }
    }
    
    /// Extract value from real-time data buffer
    /// Offset and size depend on ECU firmware version
    pub fn get_value_u8(&self, offset: usize) -> Option<u8> {
        if offset < self.realtime_size {
            Some(self.realtime_buffer[offset])
        } else {
            None
        }
    }
    
    pub fn get_value_u16(&self, offset: usize) -> Option<u16> {
        if offset + 1 < self.realtime_size {
            let high = self.realtime_buffer[offset] as u16;
            let low = self.realtime_buffer[offset + 1] as u16;
            Some((high << 8) | low)
        } else {
            None
        }
    }
    
    pub fn get_value_i16(&self, offset: usize) -> Option<i16> {
        self.get_value_u16(offset).map(|v| v as i16)
    }
    
    /// Common MegaSquirt data extraction helpers (MS2 format)
    /// These offsets may vary by firmware - should be configurable
    
    pub fn get_rpm(&self) -> Option<u16> {
        // RPM is typically at offset 6-7 (MS2)
        self.get_value_u16(6)
    }
    
    pub fn get_map(&self) -> Option<u16> {
        // MAP is typically at offset 4-5 (MS2)
        self.get_value_u16(4)
    }
    
    pub fn get_coolant_temp(&self) -> Option<i16> {
        // Coolant temp typically at offset 8-9 (MS2)
        self.get_value_i16(8)
    }
    
    pub fn get_tps(&self) -> Option<u16> {
        // TPS typically at offset 14-15 (MS2)
        self.get_value_u16(14)
    }
    
    pub fn get_afr(&self) -> Option<u16> {
        // AFR/Lambda typically at offset 16-17 (MS2)
        self.get_value_u16(16)
    }
    
    pub fn get_battery_voltage(&self) -> Option<u16> {
        // Battery voltage typically at offset 18-19 (MS2)
        self.get_value_u16(18)
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// Get raw buffer for advanced parsing
    pub fn get_raw_buffer(&self) -> &[u8] {
        &self.realtime_buffer[..self.realtime_size]
    }
}

impl Default for MegaSquirt {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic ECU data structure for common values
pub struct ECUData {
    pub rpm: f32,
    pub map: f32,
    pub tps: f32,
    pub coolant_temp: f32,
    pub intake_temp: f32,
    pub afr: f32,
    pub battery_voltage: f32,
    pub vehicle_speed: f32,
    pub fuel_pressure: f32,
    pub oil_pressure: f32,
    pub boost: f32,
    pub ignition_advance: f32,
    pub injector_duty: f32,
}

impl ECUData {
    pub fn new() -> Self {
        ECUData {
            rpm: 0.0,
            map: 0.0,
            tps: 0.0,
            coolant_temp: 0.0,
            intake_temp: 0.0,
            afr: 0.0,
            battery_voltage: 0.0,
            vehicle_speed: 0.0,
            fuel_pressure: 0.0,
            oil_pressure: 0.0,
            boost: 0.0,
            ignition_advance: 0.0,
            injector_duty: 0.0,
        }
    }
    
    /// Update from MegaSquirt real-time data
    pub fn update_from_ms(&mut self, ms: &MegaSquirt) {
        // Extract and convert values
        // Note: Scaling factors depend on firmware version
        
        if let Some(rpm) = ms.get_rpm() {
            self.rpm = rpm as f32;
        }
        
        if let Some(map) = ms.get_map() {
            self.map = (map as f32) / 10.0; // Typically in 0.1 kPa units
        }
        
        if let Some(temp) = ms.get_coolant_temp() {
            self.coolant_temp = (temp as f32) / 10.0; // Typically in 0.1°F units
        }
        
        if let Some(tps) = ms.get_tps() {
            self.tps = (tps as f32) / 10.0; // Typically in 0.1% units
        }
        
        if let Some(afr) = ms.get_afr() {
            self.afr = (afr as f32) / 10.0; // Typically in 0.1 AFR units
        }
        
        if let Some(voltage) = ms.get_battery_voltage() {
            self.battery_voltage = (voltage as f32) / 10.0; // Typically in 0.1V units
        }
        
        // Calculate boost from MAP (assuming 1 bar = 14.7 PSI at sea level)
        self.boost = (self.map - 101.325) * 0.145038; // kPa to PSI, subtract atmospheric
    }
}

impl Default for ECUData {
    fn default() -> Self {
        Self::new()
    }
}
