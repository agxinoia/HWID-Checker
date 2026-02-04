#[cfg(windows)]
use serde::Deserialize;

#[cfg(windows)]
use wmi::{COMLibrary, WMIConnection};

#[derive(Debug, Clone)]
pub struct ProcessorInfo {
    pub manufacturer: String,
    pub processor_type: String,
    pub serial_number: String,
    pub part_number: String,
    pub asset_tag: String,
    pub socket: String,
    pub core_count: String,
    pub thread_count: String,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32Processor {
    #[serde(default)]
    manufacturer: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    processor_id: Option<String>,
    #[serde(default)]
    socket_designation: Option<String>,
    #[serde(default)]
    number_of_cores: Option<u32>,
    #[serde(default)]
    number_of_logical_processors: Option<u32>,
    #[serde(default)]
    part_number: Option<String>,
    #[serde(default)]
    serial_number: Option<String>,
    #[serde(default)]
    asset_tag: Option<String>,
}

impl ProcessorInfo {
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

        let procs: Vec<Win32Processor> = wmi_con
            .raw_query("SELECT * FROM Win32_Processor")
            .unwrap_or_default();
        
        let proc = procs.first();

        Self {
            manufacturer: proc
                .and_then(|p| p.manufacturer.clone())
                .unwrap_or_else(|| "N/A".to_string()),
            processor_type: proc
                .and_then(|p| p.name.clone())
                .unwrap_or_else(|| "N/A".to_string()),
            serial_number: proc
                .and_then(|p| p.serial_number.clone())
                .filter(|s| !s.trim().is_empty() && s.trim() != "To Be Filled By O.E.M.")
                .unwrap_or_else(|| "(Not Exposed)".to_string()),
            part_number: proc
                .and_then(|p| p.part_number.clone())
                .filter(|s| !s.trim().is_empty() && s.trim() != "To Be Filled By O.E.M.")
                .unwrap_or_else(|| "(Not Exposed)".to_string()),
            asset_tag: proc
                .and_then(|p| p.asset_tag.clone())
                .filter(|s| !s.trim().is_empty() && s.trim() != "To Be Filled By O.E.M.")
                .unwrap_or_else(|| "(Not Exposed)".to_string()),
            socket: proc
                .and_then(|p| p.socket_designation.clone())
                .unwrap_or_else(|| "N/A".to_string()),
            core_count: proc
                .and_then(|p| p.number_of_cores.map(|n| n.to_string()))
                .unwrap_or_else(|| "N/A".to_string()),
            thread_count: proc
                .and_then(|p| p.number_of_logical_processors.map(|n| n.to_string()))
                .unwrap_or_else(|| "N/A".to_string()),
        }
    }
}

impl Default for ProcessorInfo {
    fn default() -> Self {
        Self {
            manufacturer: "N/A".to_string(),
            processor_type: "N/A".to_string(),
            serial_number: "N/A".to_string(),
            part_number: "N/A".to_string(),
            asset_tag: "N/A".to_string(),
            socket: "N/A".to_string(),
            core_count: "N/A".to_string(),
            thread_count: "N/A".to_string(),
        }
    }
}
