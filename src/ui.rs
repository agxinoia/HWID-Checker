use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, Tab};
use crate::info::advanced::SerialStatus;

pub fn draw_ui(frame: &mut Frame, app: &App) {
    let size = frame.area();
    
    // Main layout: sidebar (20%) + content (80%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22),
            Constraint::Min(40),
        ])
        .split(size);

    draw_sidebar(frame, app, main_chunks[0]);
    draw_content(frame, app, main_chunks[1]);
}

fn draw_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = Tab::all()
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let style = if i == app.current_tab {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            let content = format!(" {} {}", tab.icon(), tab.label());
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" ◆ Serial Checker ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        );

    frame.render_widget(list, area);
}

fn draw_content(frame: &mut Frame, app: &App, area: Rect) {
    let current_tab = app.current_tab();
    
    let content = match current_tab {
        Tab::System => format_system_info(&app.system_info),
        Tab::Bios => format_bios_info(&app.bios_info),
        Tab::Baseboard => format_baseboard_info(&app.baseboard_info),
        Tab::Disk => format_disk_info(&app.disk_info),
        Tab::Processor => format_processor_info(&app.processor_info),
        Tab::Chassis => format_chassis_info(&app.chassis_info),
        Tab::Network => format_network_info(&app.network_info),
        Tab::Monitor => format_monitor_info(&app.monitor_info),
        Tab::Gpu => format_gpu_info(&app.gpu_info),
        Tab::Advanced => format_advanced_info(app),
    };

    let title = format!(" {} {} Information ", current_tab.icon(), current_tab.label());
    
    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(title)
                .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0));

    frame.render_widget(paragraph, area);

    // Draw help bar at bottom
    let help_text = if let Some(status) = &app.status_message {
        format!(" {} │ A: Advanced │ Tab: Export │ q: Quit ", status)
    } else {
        " ↑↓/jk: Navigate │ ←→/hl: Scroll │ A: Advanced │ Tab: Export │ q: Quit ".to_string()
    };
    
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    
    if area.height > 3 {
        let help_style = if app.status_message.is_some() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let help = Paragraph::new(help_text)
            .style(help_style);
        frame.render_widget(help, help_area);
    }
}

fn format_system_info(info: &crate::info::system::SystemInfo) -> Text<'static> {
    let lines = vec![
        Line::from(vec![
            Span::styled("Manufacturer:       ", Style::default().fg(Color::Yellow)),
            Span::styled(info.manufacturer.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Product Name:       ", Style::default().fg(Color::Yellow)),
            Span::styled(info.product_name.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Version Index:      ", Style::default().fg(Color::Yellow)),
            Span::styled(info.version.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("System Serial:      ", Style::default().fg(Color::Yellow)),
            Span::styled(info.serial_number.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("System UUID:        ", Style::default().fg(Color::Yellow)),
            Span::styled(info.uuid.clone(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Family Serial:      ", Style::default().fg(Color::Yellow)),
            Span::styled(info.family.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("SKU Number:         ", Style::default().fg(Color::Yellow)),
            Span::styled(info.sku.clone(), Style::default().fg(Color::White)),
        ]),
    ];
    
    Text::from(lines)
}

fn format_bios_info(info: &crate::info::bios::BiosInfo) -> Text<'static> {
    let status_style = |enabled: bool| {
        if enabled {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        }
    };
    
    let status_text = |enabled: bool| {
        if enabled { "Enabled" } else { "Disabled" }
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("BIOS Vendor:        ", Style::default().fg(Color::Yellow)),
            Span::styled(info.vendor.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("BIOS Version:       ", Style::default().fg(Color::Yellow)),
            Span::styled(info.version.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Release Date:       ", Style::default().fg(Color::Yellow)),
            Span::styled(info.release_date.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Core Isolation:     ", Style::default().fg(Color::Yellow)),
            Span::styled(status_text(info.core_isolation).to_string(), status_style(info.core_isolation)),
        ]),
        Line::from(vec![
            Span::styled("Virtualization:     ", Style::default().fg(Color::Yellow)),
            Span::styled(status_text(info.virtualization).to_string(), status_style(info.virtualization)),
        ]),
        Line::from(vec![
            Span::styled("Secure Boot:        ", Style::default().fg(Color::Yellow)),
            Span::styled(status_text(info.secure_boot).to_string(), status_style(info.secure_boot)),
        ]),
        Line::from(vec![
            Span::styled("TPM Status:         ", Style::default().fg(Color::Yellow)),
            Span::styled(status_text(info.tpm_enabled).to_string(), status_style(info.tpm_enabled)),
        ]),
    ];
    
    Text::from(lines)
}

fn format_baseboard_info(info: &crate::info::baseboard::BaseboardInfo) -> Text<'static> {
    let lines = vec![
        Line::from(vec![
            Span::styled("Manufacturer:       ", Style::default().fg(Color::Yellow)),
            Span::styled(info.manufacturer.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Product Name:       ", Style::default().fg(Color::Yellow)),
            Span::styled(info.product_name.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Version Index:      ", Style::default().fg(Color::Yellow)),
            Span::styled(info.version.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Serial Number:      ", Style::default().fg(Color::Yellow)),
            Span::styled(info.serial_number.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Asset Number:       ", Style::default().fg(Color::Yellow)),
            Span::styled(info.asset_tag.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("(CS) Location:      ", Style::default().fg(Color::Yellow)),
            Span::styled(info.location.clone(), Style::default().fg(Color::White)),
        ]),
    ];
    
    Text::from(lines)
}

fn format_disk_info(info: &crate::info::disk::DiskInfo) -> Text<'static> {
    let mut lines = vec![];
    
    for (i, disk) in info.disks.iter().enumerate() {
        if i > 0 {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("─".repeat(40), Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(""));
        }
        
        lines.push(Line::from(vec![
            Span::styled(format!("▸ Disk {}", i + 1), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("DISK_STORAGE_MODEL:     ", Style::default().fg(Color::Yellow)),
            Span::styled(disk.model.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("STORAGE_QUERY_PROPERTY: ", Style::default().fg(Color::Yellow)),
            Span::styled(disk.storage_query.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("SMART_RCV_DRIVE_DATA:   ", Style::default().fg(Color::Yellow)),
            Span::styled(disk.smart_data.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("STORAGE_QUERY_WWN:      ", Style::default().fg(Color::Yellow)),
            Span::styled(disk.wwn.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("SCSI_PASS_THROUGH:      ", Style::default().fg(Color::Yellow)),
            Span::styled(disk.scsi.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("ATA_PASS_THROUGH:       ", Style::default().fg(Color::Yellow)),
            Span::styled(disk.ata.clone(), Style::default().fg(Color::White)),
        ]));
    }
    
    if info.disks.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("No disk information available", Style::default().fg(Color::DarkGray)),
        ]));
    }
    
    Text::from(lines)
}

fn format_processor_info(info: &crate::info::processor::ProcessorInfo) -> Text<'static> {
    let lines = vec![
        Line::from(vec![
            Span::styled("CPU Manufacturer:   ", Style::default().fg(Color::Yellow)),
            Span::styled(info.manufacturer.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Processor Type:     ", Style::default().fg(Color::Yellow)),
            Span::styled(info.processor_type.clone(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Serial Number:      ", Style::default().fg(Color::Yellow)),
            Span::styled(info.serial_number.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Part Number:        ", Style::default().fg(Color::Yellow)),
            Span::styled(info.part_number.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Asset Number:       ", Style::default().fg(Color::Yellow)),
            Span::styled(info.asset_tag.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Processor Socket:   ", Style::default().fg(Color::Yellow)),
            Span::styled(info.socket.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Core Count:         ", Style::default().fg(Color::Yellow)),
            Span::styled(info.core_count.clone(), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Thread Count:       ", Style::default().fg(Color::Yellow)),
            Span::styled(info.thread_count.clone(), Style::default().fg(Color::Green)),
        ]),
    ];
    
    Text::from(lines)
}

fn format_chassis_info(info: &crate::info::chassis::ChassisInfo) -> Text<'static> {
    let lines = vec![
        Line::from(vec![
            Span::styled("Manufacturer:       ", Style::default().fg(Color::Yellow)),
            Span::styled(info.manufacturer.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Chassis Type:       ", Style::default().fg(Color::Yellow)),
            Span::styled(info.chassis_type.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Version Index:      ", Style::default().fg(Color::Yellow)),
            Span::styled(info.version.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Serial Number:      ", Style::default().fg(Color::Yellow)),
            Span::styled(info.serial_number.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Asset Number:       ", Style::default().fg(Color::Yellow)),
            Span::styled(info.asset_tag.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("SKU Number:         ", Style::default().fg(Color::Yellow)),
            Span::styled(info.sku.clone(), Style::default().fg(Color::White)),
        ]),
    ];
    
    Text::from(lines)
}

fn format_network_info(info: &crate::info::network::NetworkInfo) -> Text<'static> {
    let mut lines = vec![];
    
    if info.interfaces.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("No Network data available", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        for (i, iface) in info.interfaces.iter().enumerate() {
            if i > 0 {
                lines.push(Line::from(""));
            }
            
            lines.push(Line::from(vec![
                Span::styled(format!("▸ {}", iface.name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  MAC Address:      ", Style::default().fg(Color::Yellow)),
                Span::styled(iface.mac_address.clone(), Style::default().fg(Color::White)),
            ]));
            if !iface.ip_address.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  IP Address:       ", Style::default().fg(Color::Yellow)),
                    Span::styled(iface.ip_address.clone(), Style::default().fg(Color::Green)),
                ]));
            }
        }
    }
    
    Text::from(lines)
}

fn format_monitor_info(info: &crate::info::monitor::MonitorInfo) -> Text<'static> {
    let mut lines = vec![];
    
    for (i, monitor) in info.monitors.iter().enumerate() {
        if i > 0 {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("─".repeat(40), Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(""));
        }
        
        lines.push(Line::from(vec![
            Span::styled(format!("Active Monitor: {}", monitor.display_name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Manufacturer:       ", Style::default().fg(Color::Yellow)),
            Span::styled(monitor.manufacturer.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Model Name:         ", Style::default().fg(Color::Yellow)),
            Span::styled(monitor.model.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Monitor Serial:     ", Style::default().fg(Color::Yellow)),
            Span::styled(monitor.serial_number.clone(), Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("ID Serial Number:   ", Style::default().fg(Color::Yellow)),
            Span::styled(monitor.id_serial.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Resolution:         ", Style::default().fg(Color::Yellow)),
            Span::styled(monitor.resolution.clone(), Style::default().fg(Color::Green)),
        ]));
    }
    
    if info.monitors.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("No monitor information available", Style::default().fg(Color::DarkGray)),
        ]));
    }
    
    Text::from(lines)
}

fn format_gpu_info(info: &crate::info::gpu::GpuInfo) -> Text<'static> {
    let mut lines = vec![];
    
    for (i, gpu) in info.gpus.iter().enumerate() {
        if i > 0 {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("─".repeat(40), Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(""));
        }
        
        lines.push(Line::from(vec![
            Span::styled(format!("▸ GPU {}", i + 1), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("PCI Device:         ", Style::default().fg(Color::Yellow)),
            Span::styled(gpu.pci_device.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("GPU Name:           ", Style::default().fg(Color::Yellow)),
            Span::styled(gpu.name.clone(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("GUID Serial:        ", Style::default().fg(Color::Yellow)),
            Span::styled(gpu.guid.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("VRAM:               ", Style::default().fg(Color::Yellow)),
            Span::styled(gpu.vram.clone(), Style::default().fg(Color::Green)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Vendor:             ", Style::default().fg(Color::Yellow)),
            Span::styled(gpu.vendor.clone(), Style::default().fg(Color::White)),
        ]));
    }
    
    if info.gpus.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("No GPU information available", Style::default().fg(Color::DarkGray)),
        ]));
    }
    
    Text::from(lines)
}

fn format_advanced_info(app: &crate::app::App) -> Text<'static> {
    let mut lines = vec![];
    
    // === MOTHERBOARD LOCK STATUS ===
    lines.push(Line::from(vec![
        Span::styled("═══ MOTHERBOARD LOCK STATUS ═══", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));
    
    let locked_info = &app.locked_info;
    
    let lock_style = if locked_info.overall_locked {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    };
    
    let lock_icon = if locked_info.overall_locked { "🔒" } else { "🔓" };
    let lock_text = if locked_info.overall_locked { "LOCKED" } else { "UNLOCKED" };
    
    lines.push(Line::from(vec![
        Span::styled("Overall Status:     ", Style::default().fg(Color::Yellow)),
        Span::styled(format!("{} {}", lock_icon, lock_text), lock_style),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("OEM Vendor:         ", Style::default().fg(Color::Yellow)),
        Span::styled(locked_info.oem_vendor.clone(), Style::default().fg(Color::White)),
    ]));
    
    let bool_style = |v: bool| if v { Style::default().fg(Color::Red) } else { Style::default().fg(Color::Green) };
    let bool_text = |v: bool| if v { "Yes" } else { "No" };
    
    lines.push(Line::from(vec![
        Span::styled("OEM System:         ", Style::default().fg(Color::Yellow)),
        Span::styled(bool_text(locked_info.is_oem_system).to_string(), bool_style(locked_info.is_oem_system)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Secure Boot:        ", Style::default().fg(Color::Yellow)),
        Span::styled(bool_text(locked_info.secure_boot_enforced).to_string(), bool_style(locked_info.secure_boot_enforced)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("TPM Active:         ", Style::default().fg(Color::Yellow)),
        Span::styled(bool_text(locked_info.tpm_locked).to_string(), bool_style(locked_info.tpm_locked)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("BIOS Protected:     ", Style::default().fg(Color::Yellow)),
        Span::styled(bool_text(locked_info.bios_write_protected).to_string(), bool_style(locked_info.bios_write_protected)),
    ]));
    
    if !locked_info.lock_reasons.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Lock Reasons:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        for reason in &locked_info.lock_reasons {
            lines.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(Color::DarkGray)),
                Span::styled(reason.clone(), Style::default().fg(Color::Red)),
            ]));
        }
    }
    
    // === SERIAL COMPARISON ===
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("═══ SERIAL COMPARISON ═══", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));
    
    if let Some(prev) = &app.previous_serials {
        lines.push(Line::from(vec![
            Span::styled("Comparing with previous serials_export.txt", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("🟢 Unchanged  🔴 Changed  🟡 New", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(""));
        
        // Compare key serials
        let comparisons = [
            ("System Serial", prev.compare("system_serial", &app.system_info.serial_number), app.system_info.serial_number.clone()),
            ("System UUID", prev.compare("system_uuid", &app.system_info.uuid), app.system_info.uuid.clone()),
            ("Baseboard Serial", prev.compare("baseboard_serial", &app.baseboard_info.serial_number), app.baseboard_info.serial_number.clone()),
            ("Chassis Serial", prev.compare("chassis_serial", &app.chassis_info.serial_number), app.chassis_info.serial_number.clone()),
        ];
        
        for (label, status, current) in comparisons {
            let (icon, style, extra) = match &status {
                SerialStatus::Unchanged => ("🟢", Style::default().fg(Color::Green), String::new()),
                SerialStatus::Changed { old } => ("🔴", Style::default().fg(Color::Red), format!(" (was: {})", old)),
                SerialStatus::New => ("🟡", Style::default().fg(Color::Yellow), " (new)".to_string()),
            };
            
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default()),
                Span::styled(format!("{}: ", label), Style::default().fg(Color::Yellow)),
                Span::styled(current, style),
                Span::styled(extra, Style::default().fg(Color::DarkGray)),
            ]));
        }
    } else {
        lines.push(Line::from(vec![
            Span::styled("⚠ No previous export found", Style::default().fg(Color::Yellow)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Press Tab to export serials first", Style::default().fg(Color::DarkGray)),
        ]));
    }
    
    // === SPOOFING ADVICE ===
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("═══ SPOOFING ADVICE ═══", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));
    
    for advice in &app.spoofing_advice {
        let difficulty_style = match advice.difficulty.as_str() {
            "Easy" => Style::default().fg(Color::Green),
            "Medium" => Style::default().fg(Color::Yellow),
            "Advanced" => Style::default().fg(Color::Red),
            _ => Style::default().fg(Color::White),
        };
        
        lines.push(Line::from(vec![
            Span::styled(format!("▸ {}", advice.category), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Method:     ", Style::default().fg(Color::Yellow)),
            Span::styled(advice.method.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Difficulty: ", Style::default().fg(Color::Yellow)),
            Span::styled(advice.difficulty.clone(), difficulty_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Details:    ", Style::default().fg(Color::Yellow)),
            Span::styled(advice.details.clone(), Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(""));
    }
    
    Text::from(lines)
}
