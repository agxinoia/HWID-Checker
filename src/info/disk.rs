#[cfg(windows)]
use serde::Deserialize;

#[cfg(windows)]
use wmi::{COMLibrary, WMIConnection};

#[derive(Debug, Clone)]
pub struct DiskEntry {
    pub model: String,
    pub storage_query: String,
    pub smart_data: String,
    pub wwn: String,
    pub scsi: String,
    pub ata: String,
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub disks: Vec<DiskEntry>,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32DiskDrive {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    serial_number: Option<String>,
    #[serde(default)]
    interface_type: Option<String>,
    #[serde(default)]
    firmware_revision: Option<String>,
    #[serde(default)]
    media_type: Option<String>,
    #[serde(default)]
    #[serde(rename = "PNPDeviceID")]
    pnp_device_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct MsftDisk {
    #[serde(default)]
    friendly_name: Option<String>,
    #[serde(default)]
    serial_number: Option<String>,
    #[serde(default)]
    unique_id: Option<String>,
}

impl DiskInfo {
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

        // Query Win32_DiskDrive for disk information
        let drives: Vec<Win32DiskDrive> = wmi_con
            .raw_query("SELECT * FROM Win32_DiskDrive")
            .unwrap_or_default();

        // Try to get additional info from MSFT_Disk (StorageWMI namespace)
        let msft_disks: Vec<MsftDisk> = WMIConnection::with_namespace_path("ROOT\\Microsoft\\Windows\\Storage", com_con)
            .ok()
            .and_then(|con| con.raw_query("SELECT * FROM MSFT_Disk").ok())
            .unwrap_or_default();

        let mut disks = Vec::new();

        for (i, drive) in drives.iter().enumerate() {
            let model = drive.model.clone().unwrap_or_else(|| "Unknown".to_string());
            let interface = drive.interface_type.clone().unwrap_or_else(|| "Unknown".to_string());
            
            // STORAGE_QUERY_PROPERTY equivalent - Serial from WMI
            let storage_query = drive.serial_number.clone()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "N/A".to_string());
            
            // SMART status
            let smart_data = drive.status.clone()
                .map(|s| if s == "OK" { "OK".to_string() } else { format!("Status: {}", s) })
                .unwrap_or_else(|| "N/A".to_string());
            
            // WWN - try to get from MSFT_Disk UniqueId
            let wwn = msft_disks.get(i)
                .and_then(|d| d.unique_id.clone())
                .unwrap_or_else(|| "N/A".to_string());
            
            // SCSI/ATA based on interface type
            let (scsi, ata) = match interface.as_str() {
                "SCSI" => ("Supported".to_string(), "N/A".to_string()),
                "IDE" | "ATA" => ("N/A".to_string(), "Supported".to_string()),
                "USB" => ("USB".to_string(), "N/A".to_string()),
                _ if interface.contains("NVMe") => ("NVMe".to_string(), "N/A".to_string()),
                _ => (interface.clone(), "N/A".to_string()),
            };

            disks.push(DiskEntry {
                model,
                storage_query,
                smart_data,
                wwn,
                scsi,
                ata,
            });
        }

        Self { disks }
    }
}

impl Default for DiskInfo {
    fn default() -> Self {
        Self { disks: Vec::new() }
    }
}
