# nmon-rs

[![rust](https://img.shields.io/badge/rust-%23e24d2c.svg?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![ratatui](https://img.shields.io/badge/ratatui-%23e63946.svg?style=flat-square&logo=ratatui&logoColor=white)](https://crates.io/crates/ratatui)
[![crossterm](https://img.shields.io/badge/crossterm-%232a9d8f.svg?style=flat-square&logo=terminal&logoColor=white)](https://crates.io/crates/crossterm)[![GitHub release (latest by date)](https://img.shields.io/github/v/release/bugsbunny8/nmon-rs?style=flat-square&logo=github)](https://github.com/bugsbunny8/nmon-rs/releases/latest)
[![github actions](https://img.shields.io/github/actions/workflow/status/bugsbunny8/nmon-rs/release.yml?style=flat-square&logo=github)](https://github.com/bugsbunny8/nmon-rs/actions)

A terminal-based system performance monitor written in **Rust**, reimplement the classic **nmon** (Nigel's Monitor) utility. It supports both an interactive terminal user interface (TUI) and a non-interactive CSV logging mode compatible with standard nmon analysis tools.

---

## Features

- **Interactive TUI Dashboard**: Built on top of `ratatui` and `crossterm`. Displays rich, real-time dashboards of various system metrics.
- **CSV Logging Mode**: Records snapshots to a `.nmon` format file, structured in the nmon format (metadata, timestamp, and metrics section), suitable for spreadsheets or nmon analyzer tools.
- **Asynchronous Alerting**: Simple threshold-based alerting evaluates system metrics asynchronously and appends warnings/notices to `alerts.log` (e.g., CPU utilization > 85%, RAM usage > 90%).
- **Cross-Platform Metrics Support**:
  - Gathers CPU core details (load, vendor, model, speed, logical & physical core counts).
  - Tracks RAM and Swap space.
  - Monitors individual disks, partition storage space, read/write throughput, and busy map indicators.
  - Queries network interface speeds (RX/TX bytes per second).
  - Displays top active processes (CPU/Memory usage list).
  - Retrieves OS metadata (OS version, kernel version, uptime, load averages).

---

## Interactive Keyboard Controls (TUI Mode)

When running in interactive TUI mode, you can use the following single-character toggles to display or hide specific metric panels:

| Key | Description |
|---|---|
| **`c`** | Toggle CPU Utilisation (SMP multi-core view) |
| **`l`** | Toggle CPU Long-Term historical averages |
| **`m`** | Toggle Memory & Swap Space utilization |
| **`V`** | Toggle Virtual Memory statistics (includes swap paging) |
| **`d`** | Toggle Disk Aggregate I/O Graph |
| **`D`** | Toggle Disk detailed stats table (drives throughput & capacity) |
| **`o`** | Toggle Disk %Busy Map (nmon-style character indicator) |
| **`j`** | Toggle File Systems (JFS) usage list & bars |
| **`n`** | Toggle Network Interfaces throughput |
| **`t`** | Toggle Top processes list |
| **`r`** | Toggle System Resources metadata (hardware specs & OS info) |
| **`k`** | Toggle Kernel info, Load average, and Uptime |
| **`h`** | Toggle Mini Help & Toggle shortcuts reference |
| **`+`** | Double the snapshot update interval (slower updates) |
| **`-`** | Halve the snapshot update interval (faster updates) |
| **`Space`** | Force refresh metrics immediately |
| **`q`** | Quit program |

*When the **Top processes list (`t`)** is active:*
- Press **`4`** to sort processes by **Memory Consumption**.
- Press **`5`** to sort processes by **CPU Usage**.

---

## Installation & Building

Make sure you have Rust and Cargo installed. Then, clone the repository and build:

```bash
# Build in debug/dev mode
cargo build

# Build optimized production binary
cargo build --release
```

The compiled binary will be located at `target/debug/nmon-rs` or `target/release/nmon-rs`.

---

## Command Line Interface (CLI) Usage

```text
nmon-rs: Curses based Performance Monitor (written in Rust)
Usage: monitor-rs [options]

Options:
  -f          Start CSV logging mode to a default named file.
  -F <file>   Start CSV logging mode to the specified file.
  -s <secs>   Set snapshot refresh interval in seconds (default: 2).
  -c <count>  Set the snapshot count (default: 288, logging mode only).
  -h, -?      Display this help menu and exit.
  -V          Print version and exit.
```

### Examples:

1. **Start TUI mode** with a custom 3-second update interval:
   ```bash
   cargo run -- -s 3
   ```

2. **Start CSV Logging mode** saving to `my_server.nmon` capturing 60 snapshots every 5 seconds (5 minutes total execution):
   ```bash
   cargo run -- -F my_server.nmon -s 5 -c 60
   ```

---

## System Architecture & Implementation Details

`nmon-rs` is cleanly structured into modular components:

- **Metrics Gathering (`src/metrics/`)**:
  - `cpu.rs`: Gathers CPU info.
  - `memory.rs`: Gathers RAM/Swap utilization.
  - `disk.rs`: Collects disk utilization and filesystem capacities. On Linux, it parses `/proc/diskstats`; on Windows, it spawns a background thread running `typeperf` to query LogicalDisk performance counters.
  - `network.rs`: Track RX/TX bytes on active interface networks.
  - `snapshot.rs`: Defines the serialize-ready `MetricSnapshot` data structure holding a complete set of system metrics.
  - `csv_logger.rs`: Handles writing header lines (`AAA`) and individual data rows (`ZZZZ` timestamps, `CPU_ALL`, `MEM`, `NET`, etc.) in standard nmon-compliant format.

- **Alerting (`src/alerting/`)**:
  - `rules.rs`: Contains the `AlertRule` configuration representing checks against the snapshot data.
  - `handler.rs`: Asynchronously runs rules evaluation in a spawned background thread to avoid blockages on main loops, appending warning logs directly to `alerts.log`.

- **User Interface (`src/ui/`)**:
  - `dashboard.rs`: Manages the overall layout rendering grid and layout heights according to the active panels in `UiState`.
  - `*_widget.rs`: Modular widgets rendering specific resource charts, maps, or statistics paragraphs via `ratatui`.
  - `theme.rs`: Basic coloring definitions.
