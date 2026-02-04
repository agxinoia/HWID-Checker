use std::fs::File;
use std::io::Write;

use crate::info::{
    system::SystemInfo,
    bios::BiosInfo,
    baseboard::BaseboardInfo,
    disk::DiskInfo,
    processor::ProcessorInfo,
    chassis::ChassisInfo,
    network::NetworkInfo,
    monitor::MonitorInfo,
    gpu::GpuInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    System,
    Bios,
    Baseboard,
    Disk,
    Processor,
    Chassis,
    Network,
    Monitor,
    Gpu,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[
            Tab::System,
            Tab::Bios,
            Tab::Baseboard,
            Tab::Disk,
            Tab::Processor,
            Tab::Chassis,
            Tab::Network,
            Tab::Monitor,
            Tab::Gpu,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Tab::System => "System",
            Tab::Bios => "BIOS",
            Tab::Baseboard => "Baseboard",
            Tab::Disk => "Disk",
            Tab::Processor => "Processor",
            Tab::Chassis => "Chassis",
            Tab::Network => "Network",
            Tab::Monitor => "Monitor",
            Tab::Gpu => "GPU",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Tab::System => "💻",
            Tab::Bios => "🔧",
            Tab::Baseboard => "🔌",
            Tab::Disk => "💾",
            Tab::Processor => "⚡",
            Tab::Chassis => "📦",
            Tab::Network => "🌐",
            Tab::Monitor => "🖥️",
            Tab::Gpu => "🎮",
        }
    }
}

pub struct App {
    pub current_tab: usize,
    pub scroll_offset: u16,
    pub status_message: Option<String>,
    pub system_info: SystemInfo,
    pub bios_info: BiosInfo,
    pub baseboard_info: BaseboardInfo,
    pub disk_info: DiskInfo,
    pub processor_info: ProcessorInfo,
    pub chassis_info: ChassisInfo,
    pub network_info: NetworkInfo,
    pub monitor_info: MonitorInfo,
    pub gpu_info: GpuInfo,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_tab: 0,
            scroll_offset: 0,
            status_message: None,
            system_info: SystemInfo::collect(),
            bios_info: BiosInfo::collect(),
            baseboard_info: BaseboardInfo::collect(),
            disk_info: DiskInfo::collect(),
            processor_info: ProcessorInfo::collect(),
            chassis_info: ChassisInfo::collect(),
            network_info: NetworkInfo::collect(),
            monitor_info: MonitorInfo::collect(),
            gpu_info: GpuInfo::collect(),
        }
    }

    pub fn current_tab(&self) -> Tab {
        Tab::all()[self.current_tab]
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % Tab::all().len();
        self.scroll_offset = 0;
    }

    pub fn previous_tab(&mut self) {
        if self.current_tab == 0 {
            self.current_tab = Tab::all().len() - 1;
        } else {
            self.current_tab -= 1;
        }
        self.scroll_offset = 0;
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn export_serials(&self) -> Result<String, std::io::Error> {
        let mut content = String::new();
        
        content.push_str("=== SERIAL EXPORT ===\n");
        content.push_str(&format!("Generated: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        // System Info
        content.push_str("=== SYSTEM ===\n");
        content.push_str(&format!("Serial Number: {}\n", self.system_info.serial_number));
        content.push_str(&format!("UUID: {}\n", self.system_info.uuid));
        content.push_str(&format!("SKU: {}\n\n", self.system_info.sku));
        
        // Baseboard Info
        content.push_str("=== BASEBOARD ===\n");
        content.push_str(&format!("Serial Number: {}\n", self.baseboard_info.serial_number));
        content.push_str(&format!("Asset Tag: {}\n\n", self.baseboard_info.asset_tag));
        
        // Processor Info
        content.push_str("=== PROCESSOR ===\n");
        content.push_str(&format!("Serial Number: {}\n", self.processor_info.serial_number));
        content.push_str(&format!("Part Number: {}\n\n", self.processor_info.part_number));
        
        // Chassis Info
        content.push_str("=== CHASSIS ===\n");
        content.push_str(&format!("Serial Number: {}\n", self.chassis_info.serial_number));
        content.push_str(&format!("Asset Tag: {}\n", self.chassis_info.asset_tag));
        content.push_str(&format!("SKU: {}\n\n", self.chassis_info.sku));
        
        // Disk Info
        content.push_str("=== DISKS ===\n");
        for (i, disk) in self.disk_info.disks.iter().enumerate() {
            content.push_str(&format!("Disk {}: {}\n", i + 1, disk.model));
            content.push_str(&format!("  Serial (Storage Query): {}\n", disk.storage_query));
            content.push_str(&format!("  WWN: {}\n", disk.wwn));
        }
        content.push('\n');
        
        // Network Info
        content.push_str("=== NETWORK ===\n");
        for iface in &self.network_info.interfaces {
            content.push_str(&format!("{}: {}\n", iface.name, iface.mac_address));
        }
        content.push('\n');
        
        // Monitor Info
        content.push_str("=== MONITORS ===\n");
        for monitor in &self.monitor_info.monitors {
            content.push_str(&format!("{}: {}\n", monitor.display_name, monitor.model));
            content.push_str(&format!("  Serial Number: {}\n", monitor.serial_number));
            content.push_str(&format!("  ID Serial: {}\n", monitor.id_serial));
        }
        content.push('\n');
        
        // GPU Info
        content.push_str("=== GPU ===\n");
        for gpu in &self.gpu_info.gpus {
            content.push_str(&format!("{}\n", gpu.name));
            content.push_str(&format!("  PCI Device: {}\n", gpu.pci_device));
            content.push_str(&format!("  GUID: {}\n", gpu.guid));
        }
        
        // Write to file
        let filename = "serials_export.txt";
        let mut file = File::create(filename)?;
        file.write_all(content.as_bytes())?;
        
        Ok(filename.to_string())
    }
}
