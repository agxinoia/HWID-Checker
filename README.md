# HWID-Checker
Basic HWID checker in Rust with a Terminal User Interface (TUI)

## Features

- **System Information** - View system serial number, UUID, manufacturer, model, and SKU
- **BIOS Information** - Access BIOS serial number, version, manufacturer, and release date
- **Baseboard Information** - Check motherboard serial number, manufacturer, product, version, and asset tag
- **Disk Information** - Display all connected disks with model, serial numbers, WWN, and storage capacity
- **Processor Information** - View CPU details including serial number, part number, manufacturer, and specifications
- **Chassis Information** - Access chassis serial number, manufacturer, type, asset tag, and SKU
- **Network Information** - List all network interfaces with MAC addresses, descriptions, and connection IDs
- **Monitor Information** - Display connected monitors with serial numbers, manufacturer, model, and resolution
- **GPU Information** - View graphics card details including name, PCI device, GUID, and VRAM

## Advanced Features

Press `A` to access the Advanced tab with:

- **Locked Motherboard Detection** - Automatically detects OEM systems (Dell, HP, Lenovo, etc.), Secure Boot status, TPM state, and BIOS write protection
- **Serial Comparison** - Compares current hardware serials with a previous export (`serials_export.txt`):
  - 🟢 Green = Serial unchanged
  - 🔴 Red = Serial changed (shows old value)
  - 🟡 Yellow = New serial (not in previous export)

## Controls

| Key | Action |
|-----|--------|
| `↑` / `k` | Previous tab |
| `↓` / `j` | Next tab |
| `←` / `h` | Scroll up |
| `→` / `l` | Scroll down |
| `A` | Jump to Advanced tab |
| `Tab` | Export all serials to `serials_export.txt` |
| `q` / `Esc` | Quit application |

## Requirements

- Windows OS (uses WMI and Win32 APIs)
- Rust toolchain

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```
