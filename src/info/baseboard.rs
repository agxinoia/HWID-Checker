#[cfg(windows)]
use serde::Deserialize;

#[cfg(windows)]
use wmi::{COMLibrary, WMIConnection};

#[derive(Debug, Clone)]
pub struct BaseboardInfo {
    pub manufacturer: String,
    pub product_name: String,
    pub version: String,
    pub serial_number: String,
    pub asset_tag: String,
    pub location: String,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32BaseBoard {
    #[serde(default)]
    manufacturer: Option<String>,
    #[serde(default)]
    product: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    serial_number: Option<String>,
    #[serde(default)]
    tag: Option<String>,
}

impl BaseboardInfo {
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

        let boards: Vec<Win32BaseBoard> = wmi_con
            .raw_query("SELECT * FROM Win32_BaseBoard")
            .unwrap_or_default();
        
        let board = boards.first();

        Self {
            manufacturer: board
                .and_then(|b| b.manufacturer.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            product_name: board
                .and_then(|b| b.product.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            version: board
                .and_then(|b| b.version.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            serial_number: board
                .and_then(|b| b.serial_number.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            asset_tag: board
                .and_then(|b| b.tag.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            location: "(Integrated)".to_string(),
        }
    }
}

impl Default for BaseboardInfo {
    fn default() -> Self {
        Self {
            manufacturer: "N/A".to_string(),
            product_name: "N/A".to_string(),
            version: "N/A".to_string(),
            serial_number: "N/A".to_string(),
            asset_tag: "N/A".to_string(),
            location: "N/A".to_string(),
        }
    }
}

/// Check if a string is a placeholder/empty value
fn is_placeholder(s: &str) -> bool {
    let lower = s.trim().to_lowercase();
    lower.is_empty() 
        || lower.contains("to be filled")
        || lower.contains("o.e.m")
        || lower.contains("oem")
        || lower == "default string"
        || lower == "not specified"
        || lower == "none"
        || lower == "n/a"
        || lower == "unknown"
        || lower == "system serial number"
        || lower == "system product name"
        || lower == "base board serial number"
        || lower == "chassis serial number"
}
