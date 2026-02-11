# HWID-Checker
Basic HWID checker in Rust with a Terminal User Interface (TUI).
This is my first rust project so there might be bugs 

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
