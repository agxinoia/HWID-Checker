#[cfg(windows)]
use serde::Deserialize;

#[cfg(windows)]
use wmi::{COMLibrary, WMIConnection};

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: String,
    pub ip_address: String,
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterface>,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32NetworkAdapter {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    #[serde(rename = "MACAddress")]
    mac_address: Option<String>,
    #[serde(default)]
    net_connection_status: Option<u16>,
    #[serde(default)]
    adapter_type: Option<String>,
    #[serde(default)]
    physical_adapter: Option<bool>,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32NetworkAdapterConfiguration {
    #[serde(default)]
    #[serde(rename = "MACAddress")]
    mac_address: Option<String>,
    #[serde(default)]
    #[serde(rename = "IPAddress")]
    ip_address: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "IPEnabled")]
    ip_enabled: Option<bool>,
    #[serde(default)]
    description: Option<String>,
}

impl NetworkInfo {
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

        // Get physical adapters with MAC addresses
        let adapters: Vec<Win32NetworkAdapter> = wmi_con
            .raw_query("SELECT * FROM Win32_NetworkAdapter WHERE PhysicalAdapter = TRUE AND MACAddress IS NOT NULL")
            .unwrap_or_default();

        // Get configurations for IP addresses
        let configs: Vec<Win32NetworkAdapterConfiguration> = wmi_con
            .raw_query("SELECT * FROM Win32_NetworkAdapterConfiguration WHERE IPEnabled = TRUE")
            .unwrap_or_default();

        let mut interfaces = Vec::new();

        for adapter in adapters.iter() {
            let mac = match &adapter.mac_address {
                Some(m) => m.clone(),
                None => continue,
            };

            let name = adapter.name.clone().unwrap_or_else(|| "Unknown".to_string());
            
            // Find matching configuration for IP address
            let ip = configs.iter()
                .find(|c| c.mac_address.as_ref() == Some(&mac))
                .and_then(|c| c.ip_address.as_ref())
                .and_then(|ips| ips.first())
                .cloned()
                .unwrap_or_default();

            interfaces.push(NetworkInterface {
                name,
                mac_address: mac,
                ip_address: ip,
            });
        }

        Self { interfaces }
    }
}

impl Default for NetworkInfo {
    fn default() -> Self {
        Self { interfaces: Vec::new() }
    }
}
