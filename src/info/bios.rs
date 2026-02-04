#[cfg(windows)]
use serde::Deserialize;

#[cfg(windows)]
use wmi::{COMLibrary, WMIConnection};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

#[derive(Debug, Clone)]
pub struct BiosInfo {
    pub vendor: String,
    pub version: String,
    pub release_date: String,
    pub core_isolation: bool,
    pub virtualization: bool,
    pub secure_boot: bool,
    pub tpm_enabled: bool,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32Bios {
    #[serde(default)]
    manufacturer: Option<String>,
    #[serde(default)]
    #[serde(rename = "SMBIOSBIOSVersion")]
    smbios_bios_version: Option<String>,
    #[serde(default)]
    release_date: Option<String>,
    #[serde(default)]
    version: Option<String>,
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32Tpm {
    #[serde(default)]
    is_activated_initial_value: Option<bool>,
    #[serde(default)]
    is_enabled_initial_value: Option<bool>,
}

impl BiosInfo {
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

        // Query Win32_BIOS
        let bioses: Vec<Win32Bios> = wmi_con
            .raw_query("SELECT * FROM Win32_BIOS")
            .unwrap_or_default();
        
        let bios = bioses.first();

        // Check Secure Boot via registry
        let secure_boot = Self::check_secure_boot();
        
        // Check Core Isolation (HVCI) via registry
        let core_isolation = Self::check_core_isolation();
        
        // Check Virtualization via WMI
        let virtualization = Self::check_virtualization(&wmi_con);
        
        // Check TPM
        let tpm_enabled = Self::check_tpm();

        // Parse release date (WMI returns format like "20231015000000.000000+000")
        let release_date = bios
            .and_then(|b| b.release_date.clone())
            .map(|d| {
                if d.len() >= 8 {
                    format!("{}/{}/{}", &d[4..6], &d[6..8], &d[0..4])
                } else {
                    d
                }
            })
            .unwrap_or_else(|| "N/A".to_string());

        Self {
            vendor: bios
                .and_then(|b| b.manufacturer.clone())
                .unwrap_or_else(|| "N/A".to_string()),
            version: bios
                .and_then(|b| b.smbios_bios_version.clone())
                .or_else(|| bios.and_then(|b| b.version.clone()))
                .unwrap_or_else(|| "N/A".to_string()),
            release_date,
            core_isolation,
            virtualization,
            secure_boot,
            tpm_enabled,
        }
    }

    #[cfg(windows)]
    fn check_secure_boot() -> bool {
        // Check UEFI Secure Boot status via registry
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\SecureBoot\\State") {
            if let Ok(value) = key.get_value::<u32, _>("UEFISecureBootEnabled") {
                return value == 1;
            }
        }
        false
    }

    #[cfg(windows)]
    fn check_core_isolation() -> bool {
        // Check HVCI / Core Isolation via registry
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\DeviceGuard\\Scenarios\\HypervisorEnforcedCodeIntegrity") {
            if let Ok(value) = key.get_value::<u32, _>("Enabled") {
                return value == 1;
            }
        }
        false
    }

    #[cfg(windows)]
    fn check_virtualization(wmi_con: &WMIConnection) -> bool {
        // Check if Hyper-V is available via Win32_ComputerSystem
        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "PascalCase")]
        struct Win32Processor {
            #[serde(default)]
            virtualization_firmware_enabled: Option<bool>,
        }
        
        let procs: Vec<Win32Processor> = wmi_con
            .raw_query("SELECT VirtualizationFirmwareEnabled FROM Win32_Processor")
            .unwrap_or_default();
        
        procs.first()
            .and_then(|p| p.virtualization_firmware_enabled)
            .unwrap_or(false)
    }

    #[cfg(windows)]
    fn check_tpm() -> bool {
        // Try to check TPM via registry
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Services\\TPM") {
            if let Ok(value) = key.get_value::<u32, _>("Start") {
                // If TPM service exists and is not disabled (4), TPM is likely enabled
                return value != 4;
            }
        }
        
        // Also check for TPM 2.0
        if hklm.open_subkey("SOFTWARE\\Microsoft\\Tpm").is_ok() {
            return true;
        }
        
        false
    }
}

impl Default for BiosInfo {
    fn default() -> Self {
        Self {
            vendor: "N/A".to_string(),
            version: "N/A".to_string(),
            release_date: "N/A".to_string(),
            core_isolation: false,
            virtualization: false,
            secure_boot: false,
            tpm_enabled: false,
        }
    }
}
