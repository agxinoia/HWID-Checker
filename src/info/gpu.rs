#[cfg(windows)]
use serde::Deserialize;

#[cfg(windows)]
use wmi::{COMLibrary, WMIConnection};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

#[derive(Debug, Clone)]
pub struct GpuEntry {
    pub pci_device: String,
    pub name: String,
    pub guid: String,
    pub vram: String,
    pub vendor: String,
}

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub gpus: Vec<GpuEntry>,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32VideoController {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    adapter_ram: Option<u64>,
    #[serde(default)]
    adapter_compatibility: Option<String>,
    #[serde(default)]
    video_processor: Option<String>,
    #[serde(default)]
    #[serde(rename = "PNPDeviceID")]
    pnp_device_id: Option<String>,
    #[serde(default)]
    driver_version: Option<String>,
}

impl GpuInfo {
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

        let controllers: Vec<Win32VideoController> = wmi_con
            .raw_query("SELECT * FROM Win32_VideoController")
            .unwrap_or_default();

        let mut gpus = Vec::new();

        for controller in controllers.iter() {
            let name = controller.name.clone()
                .or_else(|| controller.video_processor.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            
            let pci_device = controller.pnp_device_id.clone()
                .unwrap_or_else(|| "N/A".to_string());
            
            // Format VRAM
            let vram = controller.adapter_ram
                .map(|ram| {
                    if ram >= 1024 * 1024 * 1024 {
                        format!("{} GB", ram / (1024 * 1024 * 1024))
                    } else if ram >= 1024 * 1024 {
                        format!("{} MB", ram / (1024 * 1024))
                    } else {
                        format!("{} bytes", ram)
                    }
                })
                .unwrap_or_else(|| "N/A".to_string());
            
            let vendor = controller.adapter_compatibility.clone()
                .unwrap_or_else(|| {
                    // Try to determine vendor from name
                    if name.contains("NVIDIA") || name.contains("GeForce") || name.contains("RTX") || name.contains("GTX") {
                        "NVIDIA".to_string()
                    } else if name.contains("AMD") || name.contains("Radeon") {
                        "AMD".to_string()
                    } else if name.contains("Intel") {
                        "Intel".to_string()
                    } else {
                        "Unknown".to_string()
                    }
                });

            // Try to get GUID from registry
            let guid = Self::get_gpu_guid(&pci_device, &vendor);

            gpus.push(GpuEntry {
                pci_device,
                name,
                guid,
                vram,
                vendor,
            });
        }

        Self { gpus }
    }

    #[cfg(windows)]
    fn get_gpu_guid(pnp_id: &str, vendor: &str) -> String {
        // Try to get GPU GUID from registry
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        
        // Check under Display adapters
        if let Ok(key) = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Enum") {
            // The PNP ID contains the path, try to find matching device
            if let Ok(subkey) = key.open_subkey(pnp_id.replace('\\', "\\")) {
                if let Ok(guid) = subkey.get_value::<String, _>("ClassGUID") {
                    return guid;
                }
            }
        }

        // AMD GPUs often don't expose GUID in the same way
        if vendor.contains("AMD") || vendor.contains("Radeon") {
            return "N/A (LIKELY AMD GPU)".to_string();
        }

        "N/A".to_string()
    }
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self { gpus: Vec::new() }
    }
}
