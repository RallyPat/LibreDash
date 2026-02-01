/// Configuration loader for LibreDash
/// Handles loading gauge definitions, dashboards, and variable mappings
/// from SD card with fallback to embedded defaults
///
/// Priority:
/// 1. Load /boot/config.ini from SD card
/// 2. Load referenced .dash and .gauge files from SD card  
/// 3. Fall back to embedded default dashboard if SD fails
/// 4. Load mock ECU data or connect to real MegaSquirt

use crate::ts_ini_parser::GaugeConfig;
use crate::ts_gauge::TSGaugeStyle;

pub struct DashboardConfig {
    pub gauges: [GaugeConfig; 16],
    pub gauge_count: usize,
    pub use_mock_ecu: bool,
    pub mock_enabled: bool,
}

impl DashboardConfig {
    pub fn new() -> Self {
        DashboardConfig {
            gauges: [GaugeConfig::new(); 16],
            gauge_count: 0,
            use_mock_ecu: true,
            mock_enabled: true,
        }
    }

    /// Load default embedded dashboard (3-gauge layout)
    pub fn load_default_dashboard(&mut self) {
        self.gauge_count = 3;

        // Gauge 1: RPM (Tachometer)
        let mut rpm = GaugeConfig::new();
        rpm.name[0..10].copy_from_slice(b"tachometer");
        rpm.var[0..3].copy_from_slice(b"rpm");
        rpm.title[0..3].copy_from_slice(b"RPM");
        rpm.lo = 0.0;
        rpm.hi = 8000.0;
        rpm.lo_warning = 300.0;
        rpm.hi_warning = 6500.0;
        rpm.hi_danger = 7000.0;
        self.gauges[0] = rpm;

        // Gauge 2: MAP Pressure
        let mut map = GaugeConfig::new();
        map.name[0..3].copy_from_slice(b"map");
        map.var[0..3].copy_from_slice(b"map");
        map.title[0..3].copy_from_slice(b"MAP");
        map.units[0..3].copy_from_slice(b"kPa");
        map.lo = 0.0;
        map.hi = 250.0;
        map.lo_warning = 20.0;
        map.hi_warning = 220.0;
        map.hi_danger = 240.0;
        self.gauges[1] = map;

        // Gauge 3: Coolant Temperature
        let mut coolant = GaugeConfig::new();
        coolant.name[0..7].copy_from_slice(b"coolant");
        coolant.var[0..11].copy_from_slice(b"coolantTemp");
        coolant.title[0..7].copy_from_slice(b"Coolant");
        coolant.units[0..3].copy_from_slice(b"degF");  // Using "degF" instead of degree symbol
        coolant.lo = 50.0;
        coolant.hi = 250.0;
        coolant.lo_warning = 80.0;
        coolant.hi_warning = 220.0;
        coolant.hi_danger = 240.0;
        self.gauges[2] = coolant;
    }

    /// Load 6-gauge extended dashboard
    pub fn load_extended_dashboard(&mut self) {
        self.load_default_dashboard();
        
        if self.gauge_count < 16 {
            // Add more gauges...
            // Gauge 4: Oil Pressure
            let mut oil = GaugeConfig::new();
            oil.name[0..11].copy_from_slice(b"oil_pressure");
            oil.var[0..11].copy_from_slice(b"oilPressure");
            oil.title[0..3].copy_from_slice(b"Oil");
            oil.units[0..3].copy_from_slice(b"PSI");
            oil.lo = 0.0;
            oil.hi = 100.0;
            oil.lo_danger = 10.0;
            oil.lo_warning = 20.0;
            oil.hi_warning = 80.0;
            oil.hi_danger = 90.0;
            self.gauges[3] = oil;
            self.gauge_count = 4;
        }
    }

    /// Try to load configuration from SD card
    /// Returns true if SD config loaded successfully, false if using defaults
    pub fn load_from_sd_card(&mut self) -> bool {
        // Placeholder for actual SD card loading
        // Would:
        // 1. Read /boot/config.ini from SD
        // 2. Parse INI format for gauge definitions
        // 3. Load referenced .dash files if available
        // 4. Set up variable mappings from mainController.ini
        //
        // For now, always falls back to default
        false
    }

    /// Get gauge by name
    pub fn get_gauge(&self, name: &str) -> Option<&GaugeConfig> {
        for i in 0..self.gauge_count {
            let gauge_name = str::from_utf8(&self.gauges[i].name).unwrap_or("");
            if gauge_name.starts_with(name) {
                return Some(&self.gauges[i]);
            }
        }
        None
    }

    /// Map ECU variable name to gauge value
    /// Translates from MegaSquirt OutputChannels names to gauge parameters
    pub fn get_ecu_variable_value(
        &self,
        var_name: &str,
        ecu_data: &crate::mock_ecu::MockECUData,
    ) -> f32 {
        match var_name {
            "rpm" => ecu_data.rpm,
            "map" | "mapPressure" => ecu_data.map_pressure,
            "coolantTemp" | "coolant" => ecu_data.coolant_temp,
            "intakeTemp" | "intake" => ecu_data.intake_temp,
            "airFuelRatio" | "afr" => ecu_data.air_fuel_ratio,
            "oilPressure" | "oil" => ecu_data.oil_pressure,
            "fuelPressure" | "fuel" => ecu_data.fuel_pressure,
            "batteryVoltage" | "battery" => ecu_data.battery_voltage,
            "throttlePosition" | "tps" => ecu_data.throttle_position,
            "boostPressure" | "boost" => ecu_data.boost_pressure,
            "ignitionAdvance" | "timing" => ecu_data.ignition_advance,
            "injectorDuty" | "injectorPw" => ecu_data.injector_duty,
            "vehicleSpeed" | "speed" => ecu_data.vehicle_speed,
            _ => 0.0,
        }
    }
}

/// Create default gauge objects for rendering
pub fn create_default_gauges() -> (
    crate::ts_gauge::TSGauge,
    crate::ts_gauge::TSGauge,
    crate::ts_gauge::TSGauge,
) {
    let config = DashboardConfig::new();

    // RPM gauge - circular analog
    let rpm_gauge = crate::ts_gauge::TSGauge::new(
        config.gauges[0],
        TSGaugeStyle::Circular,
        100,
        100,
        300,
        300,
    );

    // MAP gauge - horizontal bar
    let map_gauge = crate::ts_gauge::TSGauge::new(
        config.gauges[1],
        TSGaugeStyle::HorizontalBar,
        550,
        100,
        300,
        300,
    );

    // Coolant gauge - vertical bar
    let coolant_gauge = crate::ts_gauge::TSGauge::new(
        config.gauges[2],
        TSGaugeStyle::VerticalBar,
        1000,
        100,
        240,
        300,
    );

    (rpm_gauge, map_gauge, coolant_gauge)
}
