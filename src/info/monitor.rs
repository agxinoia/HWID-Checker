#[cfg(windows)]
use serde::Deserialize;

#[cfg(windows)]
use wmi::{COMLibrary, WMIConnection};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

#[derive(Debug, Clone)]
pub struct MonitorEntry {
    pub display_name: String,
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub id_serial: String,
    pub resolution: String,
}

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub monitors: Vec<MonitorEntry>,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32DesktopMonitor {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    monitor_manufacturer: Option<String>,
    #[serde(default)]
    monitor_type: Option<String>,
    #[serde(default)]
    #[serde(rename = "PNPDeviceID")]
    pnp_device_id: Option<String>,
    #[serde(default)]
    screen_width: Option<u32>,
    #[serde(default)]
    screen_height: Option<u32>,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct WmiMonitorId {
    #[serde(default)]
    manufacturer_name: Option<Vec<u16>>,
    #[serde(default)]
    product_code_id: Option<Vec<u16>>,
    #[serde(default)]
    serial_number_id: Option<Vec<u16>>,
    #[serde(default)]
    user_friendly_name: Option<Vec<u16>>,
}

impl MonitorInfo {
    pub fn collect() -> Self {
        #[cfg(windows)]
        {
            Self::collect_windows()
        }
        #[cfg(not(windows))]
        {
            Self::default()
        }
    }

    #[cfg(windows)]
    fn collect_windows() -> Self {
        let com_con = COMLibrary::new();
        if com_con.is_err() {
            return Self::default();
        }
        let com_con = com_con.unwrap();
        
        let wmi_con = WMIConnection::new(com_con);
        if wmi_con.is_err() {
            return Self::default();
        }
        let wmi_con = wmi_con.unwrap();

        // Try WmiMonitorID for detailed monitor info (requires admin on some systems)
        let wmi_ids: Vec<WmiMonitorId> = WMIConnection::with_namespace_path("ROOT\\WMI", com_con)
            .ok()
            .and_then(|con| con.raw_query("SELECT * FROM WmiMonitorID").ok())
            .unwrap_or_default();

        // Fallback to Win32_DesktopMonitor
        let desktop_monitors: Vec<Win32DesktopMonitor> = wmi_con
            .raw_query("SELECT * FROM Win32_DesktopMonitor")
            .unwrap_or_default();

        let mut monitors = Vec::new();

        // Process WmiMonitorID results (more detailed)
        for (i, wmi_id) in wmi_ids.iter().enumerate() {
            let manufacturer = Self::decode_wmi_string(&wmi_id.manufacturer_name);
            let model = Self::decode_wmi_string(&wmi_id.user_friendly_name)
                .or_else(|| Self::decode_wmi_string(&wmi_id.product_code_id))
                .unwrap_or_else(|| "Unknown".to_string());
            let serial = Self::decode_wmi_string(&wmi_id.serial_number_id)
                .unwrap_or_else(|| "N/A".to_string());

            // Try to get resolution from desktop monitor
            let resolution = desktop_monitors.get(i)
                .map(|m| {
                    match (m.screen_width, m.screen_height) {
                        (Some(w), Some(h)) if w > 0 && h > 0 => format!("{}x{}", w, h),
                        _ => "N/A".to_string(),
                    }
                })
                .unwrap_or_else(|| "N/A".to_string());

            monitors.push(MonitorEntry {
                display_name: format!("\\DISPLAY{}", i + 1),
                manufacturer: manufacturer.unwrap_or_else(|| "Unknown".to_string()),
                model,
                serial_number: serial.clone(),
                id_serial: serial,
                resolution,
            });
        }

        // If WmiMonitorID didn't work, use Win32_DesktopMonitor
        if monitors.is_empty() {
            for (i, monitor) in desktop_monitors.iter().enumerate() {
                let resolution = match (monitor.screen_width, monitor.screen_height) {
                    (Some(w), Some(h)) if w > 0 && h > 0 => format!("{}x{}", w, h),
                    _ => "N/A".to_string(),
                };

                monitors.push(MonitorEntry {
                    display_name: format!("\\DISPLAY{}", i + 1),
                    manufacturer: monitor.monitor_manufacturer.clone()
                        .unwrap_or_else(|| "Unknown".to_string()),
                    model: monitor.monitor_type.clone()
                        .or_else(|| monitor.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    serial_number: Self::extract_serial_from_pnp(&monitor.pnp_device_id),
                    id_serial: monitor.pnp_device_id.clone().unwrap_or_else(|| "N/A".to_string()),
                    resolution,
                });
            }
        }

        Self { monitors }
    }

    #[cfg(windows)]
    fn decode_wmi_string(data: &Option<Vec<u16>>) -> Option<String> {
        data.as_ref().map(|v| {
            v.iter()
                .take_while(|&&c| c != 0)
                .filter_map(|&c| char::from_u32(c as u32))
                .collect::<String>()
                .trim()
                .to_string()
        }).filter(|s| !s.is_empty())
    }

    #[cfg(windows)]
    fn extract_serial_from_pnp(pnp_id: &Option<String>) -> String {
        // PNP ID format: DISPLAY\DEL404D\5&12345678&0&UID256
        // Try to extract a serial-like component
        pnp_id.as_ref()
            .and_then(|id| id.split('\\').last())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "N/A".to_string())
    }
}

impl Default for MonitorInfo {
    fn default() -> Self {
        Self { monitors: Vec::new() }
    }
}
