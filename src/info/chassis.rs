#[cfg(windows)]
use serde::Deserialize;

#[cfg(windows)]
use wmi::{COMLibrary, WMIConnection};

#[derive(Debug, Clone)]
pub struct ChassisInfo {
    pub manufacturer: String,
    pub chassis_type: String,
    pub version: String,
    pub serial_number: String,
    pub asset_tag: String,
    pub sku: String,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32SystemEnclosure {
    #[serde(default)]
    manufacturer: Option<String>,
    #[serde(default)]
    chassis_types: Option<Vec<u16>>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    serial_number: Option<String>,
    #[serde(default)]
    #[serde(rename = "SMBIOSAssetTag")]
    smbios_asset_tag: Option<String>,
    #[serde(default)]
    #[serde(rename = "SKU")]
    sku: Option<String>,
}

impl ChassisInfo {
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

        let enclosures: Vec<Win32SystemEnclosure> = wmi_con
            .raw_query("SELECT * FROM Win32_SystemEnclosure")
            .unwrap_or_default();
        
        let enclosure = enclosures.first();

        let chassis_type = enclosure
            .and_then(|e| e.chassis_types.as_ref())
            .and_then(|types| types.first())
            .map(|t| Self::chassis_type_name(*t))
            .unwrap_or_else(|| "Unknown".to_string());

        Self {
            manufacturer: enclosure
                .and_then(|e| e.manufacturer.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            chassis_type,
            version: enclosure
                .and_then(|e| e.version.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            serial_number: enclosure
                .and_then(|e| e.serial_number.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            asset_tag: enclosure
                .and_then(|e| e.smbios_asset_tag.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            sku: enclosure
                .and_then(|e| e.sku.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
        }
    }

    #[cfg(windows)]
    fn chassis_type_name(type_id: u16) -> String {
        match type_id {
            1 => "Other",
            2 => "Unknown",
            3 => "Desktop",
            4 => "Low Profile Desktop",
            5 => "Pizza Box",
            6 => "Mini Tower",
            7 => "Tower",
            8 => "Portable",
            9 => "Laptop",
            10 => "Notebook",
            11 => "Hand Held",
            12 => "Docking Station",
            13 => "All in One",
            14 => "Sub Notebook",
            15 => "Space-saving",
            16 => "Lunch Box",
            17 => "Main System Chassis",
            18 => "Expansion Chassis",
            19 => "SubChassis",
            20 => "Bus Expansion Chassis",
            21 => "Peripheral Chassis",
            22 => "Storage Chassis",
            23 => "Rack Mount Chassis",
            24 => "Sealed-Case PC",
            25 => "Multi-system Chassis",
            26 => "Compact PCI",
            27 => "Advanced TCA",
            28 => "Blade",
            29 => "Blade Enclosure",
            30 => "Tablet",
            31 => "Convertible",
            32 => "Detachable",
            33 => "IoT Gateway",
            34 => "Embedded PC",
            35 => "Mini PC",
            36 => "Stick PC",
            _ => "Unknown",
        }.to_string()
    }
}

impl Default for ChassisInfo {
    fn default() -> Self {
        Self {
            manufacturer: "N/A".to_string(),
            chassis_type: "N/A".to_string(),
            version: "N/A".to_string(),
            serial_number: "N/A".to_string(),
            asset_tag: "N/A".to_string(),
            sku: "N/A".to_string(),
        }
    }
}

/// Check if a string is a placeholder/empty value
fn is_placeholder(s: &str) -> bool {
    let lower = s.trim().to_lowercase();
    lower.is_empty() 
        || lower.contains("to be filled")
        || lower.contains("o.e.m")
        || lower == "default string"
        || lower == "not specified"
        || lower == "none"
        || lower == "unknown"
        || lower == "chassis serial number"
}
