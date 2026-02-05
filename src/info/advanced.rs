#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Represents the lock status of the motherboard/BIOS
#[derive(Debug, Clone)]
pub struct LockedMotherboardInfo {
    pub is_oem_system: bool,
    pub oem_vendor: String,
    pub bios_write_protected: bool,
    pub secure_boot_enforced: bool,
    pub tpm_locked: bool,
    pub overall_locked: bool,
    pub lock_reasons: Vec<String>,
}

/// Serial comparison result
#[derive(Debug, Clone, PartialEq)]
pub enum SerialStatus {
    Unchanged,
    Changed { old: String },
    New,
}

/// Parsed previous serials from export file
#[derive(Debug, Clone, Default)]
pub struct PreviousSerials {
    pub system_serial: Option<String>,
    pub system_uuid: Option<String>,
    pub system_sku: Option<String>,
    pub baseboard_serial: Option<String>,
    pub processor_serial: Option<String>,
    pub chassis_serial: Option<String>,
    pub disk_serials: Vec<String>,
    pub network_macs: Vec<String>,
    pub monitor_serials: Vec<String>,
    pub gpu_guids: Vec<String>,
}

impl LockedMotherboardInfo {
    pub fn detect() -> Self {
        #[cfg(windows)]
        {
            Self::detect_windows()
        }
        #[cfg(not(windows))]
        {
            Self::default()
        }
    }

    #[cfg(windows)]
    fn detect_windows() -> Self {
        let mut info = Self::default();
        let mut lock_reasons = Vec::new();

        // Detect OEM vendor from registry
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        
        // Check system manufacturer
        if let Ok(key) = hklm.open_subkey("HARDWARE\\DESCRIPTION\\System\\BIOS") {
            if let Ok(manufacturer) = key.get_value::<String, _>("SystemManufacturer") {
                let manufacturer_lower = manufacturer.to_lowercase();
                
                // Known OEM vendors with locked BIOS
                let oem_vendors = [
                    ("dell", "Dell"),
                    ("hp", "HP"),
                    ("hewlett", "HP"),
                    ("lenovo", "Lenovo"),
                    ("asus", "ASUS"),
                    ("acer", "Acer"),
                    ("msi", "MSI"),
                    ("gigabyte", "Gigabyte"),
                    ("asrock", "ASRock"),
                ];
                
                for (pattern, vendor) in oem_vendors {
                    if manufacturer_lower.contains(pattern) {
                        info.oem_vendor = vendor.to_string();
                        
                        // Dell, HP, Lenovo typically have locked BIOS
                        if ["dell", "hp", "hewlett", "lenovo"].contains(&pattern) {
                            info.is_oem_system = true;
                            lock_reasons.push(format!("{} OEM system detected - BIOS typically locked", vendor));
                        }
                        break;
                    }
                }
            }
        }

        // Check Secure Boot
        if let Ok(key) = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\SecureBoot\\State") {
            if let Ok(value) = key.get_value::<u32, _>("UEFISecureBootEnabled") {
                if value == 1 {
                    info.secure_boot_enforced = true;
                    lock_reasons.push("Secure Boot enabled - EFI modifications restricted".to_string());
                }
            }
        }

        // Check TPM
        if let Ok(key) = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Services\\TPM") {
            if let Ok(start) = key.get_value::<u32, _>("Start") {
                if start != 4 {
                    info.tpm_locked = true;
                    lock_reasons.push("TPM active - Hardware attestation may detect changes".to_string());
                }
            }
        }

        // Check for BIOS write protection indicators
        if let Ok(key) = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\DeviceGuard") {
            if let Ok(value) = key.get_value::<u32, _>("EnableVirtualizationBasedSecurity") {
                if value == 1 {
                    info.bios_write_protected = true;
                    lock_reasons.push("VBS enabled - Kernel-level protections active".to_string());
                }
            }
        }

        // Check HVCI
        if let Ok(key) = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\DeviceGuard\\Scenarios\\HypervisorEnforcedCodeIntegrity") {
            if let Ok(value) = key.get_value::<u32, _>("Enabled") {
                if value == 1 {
                    info.bios_write_protected = true;
                    if !lock_reasons.iter().any(|r| r.contains("HVCI")) {
                        lock_reasons.push("HVCI enabled - Driver signing enforced".to_string());
                    }
                }
            }
        }

        info.lock_reasons = lock_reasons;
        info.overall_locked = info.is_oem_system || info.secure_boot_enforced || info.bios_write_protected;
        
        info
    }
}

impl Default for LockedMotherboardInfo {
    fn default() -> Self {
        Self {
            is_oem_system: false,
            oem_vendor: "Unknown".to_string(),
            bios_write_protected: false,
            secure_boot_enforced: false,
            tpm_locked: false,
            overall_locked: false,
            lock_reasons: Vec::new(),
        }
    }
}

impl PreviousSerials {
    /// Parse serials from the export file content
    pub fn parse(content: &str) -> Self {
        let mut serials = Self::default();
        let mut current_section = "";

        for line in content.lines() {
            let line = line.trim();
            
            // Detect section headers
            if line.starts_with("===") && line.ends_with("===") {
                current_section = line.trim_matches('=').trim();
                continue;
            }

            // Parse key-value pairs
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim().to_string();
                
                if value.is_empty() || value == "N/A" {
                    continue;
                }

                match current_section {
                    "SYSTEM" => match key {
                        "Serial Number" => serials.system_serial = Some(value),
                        "UUID" => serials.system_uuid = Some(value),
                        "SKU" => serials.system_sku = Some(value),
                        _ => {}
                    },
                    "BASEBOARD" => {
                        if key == "Serial Number" {
                            serials.baseboard_serial = Some(value);
                        }
                    }
                    "PROCESSOR" => {
                        if key == "Serial Number" {
                            serials.processor_serial = Some(value);
                        }
                    }
                    "CHASSIS" => {
                        if key == "Serial Number" {
                            serials.chassis_serial = Some(value);
                        }
                    }
                    "DISKS" => {
                        if key.contains("Serial") {
                            serials.disk_serials.push(value);
                        }
                    }
                    "NETWORK" => {
                        // Format is "Interface: MAC"
                        serials.network_macs.push(value);
                    }
                    "MONITORS" => {
                        if key == "Serial Number" {
                            serials.monitor_serials.push(value);
                        }
                    }
                    "GPU" => {
                        if key.contains("GUID") {
                            serials.gpu_guids.push(value);
                        }
                    }
                    _ => {}
                }
            }
        }

        serials
    }

    /// Compare a current serial with previous
    pub fn compare(&self, category: &str, current: &str) -> SerialStatus {
        if current.is_empty() || current == "N/A" {
            return SerialStatus::New;
        }

        let previous = match category {
            "system_serial" => self.system_serial.as_deref(),
            "system_uuid" => self.system_uuid.as_deref(),
            "system_sku" => self.system_sku.as_deref(),
            "baseboard_serial" => self.baseboard_serial.as_deref(),
            "processor_serial" => self.processor_serial.as_deref(),
            "chassis_serial" => self.chassis_serial.as_deref(),
            _ => None,
        };

        match previous {
            Some(old) if old == current => SerialStatus::Unchanged,
            Some(old) => SerialStatus::Changed { old: old.to_string() },
            None => SerialStatus::New,
        }
    }

    /// Check if a value exists in a list of previous serials
    pub fn compare_list(&self, category: &str, current: &str) -> SerialStatus {
        if current.is_empty() || current == "N/A" {
            return SerialStatus::New;
        }

        let list = match category {
            "disk" => &self.disk_serials,
            "network" => &self.network_macs,
            "monitor" => &self.monitor_serials,
            "gpu" => &self.gpu_guids,
            _ => return SerialStatus::New,
        };

        if list.is_empty() {
            return SerialStatus::New;
        }

        if list.iter().any(|s| s == current) {
            SerialStatus::Unchanged
        } else {
            SerialStatus::Changed { old: "(different from previous)".to_string() }
        }
    }
}
