#[cfg(windows)]
use serde::Deserialize;

#[cfg(windows)]
use wmi::{COMLibrary, WMIConnection};

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub manufacturer: String,
    pub product_name: String,
    pub version: String,
    pub serial_number: String,
    pub uuid: String,
    pub family: String,
    pub sku: String,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32ComputerSystemProduct {
    #[serde(default)]
    vendor: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    identifying_number: Option<String>,
    #[serde(default)]
    #[serde(rename = "UUID")]
    uuid: Option<String>,
    #[serde(default)]
    #[serde(rename = "SKUNumber")]
    sku_number: Option<String>,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32ComputerSystem {
    #[serde(default)]
    manufacturer: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    system_family: Option<String>,
    #[serde(default)]
    system_sku_number: Option<String>,
}

impl SystemInfo {
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

        // Query Win32_ComputerSystemProduct for UUID, Serial, SKU
        let products: Vec<Win32ComputerSystemProduct> = wmi_con
            .raw_query("SELECT * FROM Win32_ComputerSystemProduct")
            .unwrap_or_default();
        
        let product = products.first();

        // Query Win32_ComputerSystem for Manufacturer, Model, Family
        let systems: Vec<Win32ComputerSystem> = wmi_con
            .raw_query("SELECT * FROM Win32_ComputerSystem")
            .unwrap_or_default();
        
        let system = systems.first();

        Self {
            manufacturer: system
                .and_then(|s| s.manufacturer.clone())
                .or_else(|| product.and_then(|p| p.vendor.clone()))
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            product_name: system
                .and_then(|s| s.model.clone())
                .or_else(|| product.and_then(|p| p.name.clone()))
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            version: product
                .and_then(|p| p.version.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            serial_number: product
                .and_then(|p| p.identifying_number.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            uuid: product
                .and_then(|p| p.uuid.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            family: system
                .and_then(|s| s.system_family.clone())
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
            sku: system
                .and_then(|s| s.system_sku_number.clone())
                .or_else(|| product.and_then(|p| p.sku_number.clone()))
                .filter(|s| !is_placeholder(s))
                .unwrap_or_else(|| "N/A".to_string()),
        }
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            manufacturer: "N/A".to_string(),
            product_name: "N/A".to_string(),
            version: "N/A".to_string(),
            serial_number: "N/A".to_string(),
            uuid: "N/A".to_string(),
            family: "N/A".to_string(),
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
        || lower == "system serial number"
        || lower == "system product name"
}
