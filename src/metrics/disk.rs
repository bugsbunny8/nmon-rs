//! Disk and filesystem performance metrics collector.

use sysinfo::{Disks, System};
use super::snapshot::FileSystemInfo;

#[cfg(target_os = "windows")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "windows")]
use std::collections::HashMap;

/// Collector for gathering global disk I/O metrics and filesystem storage utilization.
pub struct DiskCollector {
    sys: System,
    disks_list: Disks,
    prev_read: u64,
    prev_write: u64,
    #[cfg(target_os = "windows")]
    windows_logical_io: Arc<Mutex<HashMap<String, (u64, u64)>>>,
    #[cfg(target_os = "linux")]
    linux_disk_io: std::collections::HashMap<String, (u64, u64, std::time::Instant)>,
}

impl DiskCollector {
    /// Creates a new `DiskCollector` instance and initializes disk size and counter values.
    /// On Windows, it spawns a background thread to collect logical disk counter stats via `typeperf`.
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks_list = Disks::new_with_refreshed_list();

        let (read, write) = Self::aggregate(&sys);

        #[cfg(target_os = "windows")]
        let windows_logical_io = {
            let io_map = Arc::new(Mutex::new(HashMap::new()));
            let io_map_clone = Arc::clone(&io_map);
            std::thread::spawn(move || {
                loop {
                    let output = std::process::Command::new("typeperf")
                        .args(&[
                            "\\LogicalDisk(*)\\Disk Read Bytes/sec",
                            "\\LogicalDisk(*)\\Disk Write Bytes/sec",
                            "-sc", "1"
                        ])
                        .output();
                    if let Ok(out) = output {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let parsed = parse_typeperf(&stdout);
                        if !parsed.is_empty() {
                            if let Ok(mut map) = io_map_clone.lock() {
                                *map = parsed;
                            }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            });
            io_map
        };

        DiskCollector {
            sys,
            disks_list,
            prev_read: read,
            prev_write: write,
            #[cfg(target_os = "windows")]
            windows_logical_io,
            #[cfg(target_os = "linux")]
            linux_disk_io: std::collections::HashMap::new(),
        }
    }

    /// Aggregates total disk read and write bytes from all active processes.
    fn aggregate(sys: &System) -> (u64, u64) {
        let mut read = 0;
        let mut write = 0;
        for process in sys.processes().values() {
            let usage = process.disk_usage();
            read += usage.total_read_bytes;
            write += usage.total_written_bytes;
        }
        (read, write)
    }

    /// Computes delta global disk I/O bytes since the last collection tick.
    pub fn collect_io(&mut self) -> (u64, u64) {
        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let (read, write) = Self::aggregate(&self.sys);
        
        let delta_read = read.saturating_sub(self.prev_read);
        let delta_write = write.saturating_sub(self.prev_write);
        
        self.prev_read = read;
        self.prev_write = write;
        
        (delta_read, delta_write)
    }

    /// Reads and parses `/proc/diskstats` on Linux to extract detailed per-disk I/O rates.
    #[cfg(target_os = "linux")]
    fn get_linux_disk_io(&mut self) -> std::collections::HashMap<String, (u64, u64)> {
        let mut current_stats = std::collections::HashMap::new();
        if let Ok(content) = std::fs::read_to_string("/proc/diskstats") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 10 {
                    let dev_name = parts[2].to_string();
                    let sectors_read = parts[5].parse::<u64>().unwrap_or(0);
                    let sectors_written = parts[9].parse::<u64>().unwrap_or(0);
                    
                    // Convert sectors to bytes (1 sector = 512 bytes)
                    let bytes_read = sectors_read * 512;
                    let bytes_written = sectors_written * 512;
                    
                    current_stats.insert(dev_name, (bytes_read, bytes_written));
                }
            }
        }

        let now = std::time::Instant::now();
        let mut speed_map = std::collections::HashMap::new();

        for (dev_name, &(bytes_read, bytes_written)) in &current_stats {
            if let Some(&(prev_r, prev_w, prev_time)) = self.linux_disk_io.get(dev_name) {
                let elapsed = now.duration_since(prev_time).as_secs_f64();
                if elapsed > 0.0 {
                    let r_bps = ((bytes_read.saturating_sub(prev_r)) as f64 / elapsed) as u64;
                    let w_bps = ((bytes_written.saturating_sub(prev_w)) as f64 / elapsed) as u64;
                    speed_map.insert(dev_name.clone(), (r_bps, w_bps));
                }
            }
        }

        // Save for next tick
        for (dev_name, (bytes_read, bytes_written)) in current_stats {
            self.linux_disk_io.insert(dev_name, (bytes_read, bytes_written, now));
        }

        speed_map
    }

    /// Queries the OS for mounted disk filesystems, storage space, and read/write speeds.
    pub fn collect_filesystems(&mut self) -> Vec<FileSystemInfo> {
        self.disks_list.refresh(true);

        #[cfg(target_os = "windows")]
        let logical_io = self.windows_logical_io.lock().ok().map(|g| g.clone()).unwrap_or_default();

        #[cfg(target_os = "linux")]
        let linux_io = self.get_linux_disk_io();

        self.disks_list
            .iter()
            .map(|disk| {
                let mount_point = disk.mount_point().to_string_lossy().into_owned();
                let mut name = disk.name().to_string_lossy().into_owned();
                if name.is_empty() {
                    name = mount_point.trim_end_matches('\\').to_string();
                }
                
                #[cfg(target_os = "windows")]
                let (read_bps, write_bps) = {
                    let drive_key = mount_point.trim_end_matches('\\').to_string();
                    if let Some(&(r, w)) = logical_io.get(&drive_key) {
                        (Some(r), Some(w))
                    } else {
                        (None, None)
                    }
                };

                #[cfg(target_os = "linux")]
                let (read_bps, write_bps) = {
                    let dev_name = std::path::Path::new(&name)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| name.clone());
                    
                    if let Some(&(r, w)) = linux_io.get(&dev_name) {
                        (Some(r), Some(w))
                    } else {
                        let mut found = (None, None);
                        for (k, &(r, w)) in &linux_io {
                            if dev_name.starts_with(k) {
                                found = (Some(r), Some(w));
                                break;
                            }
                        }
                        found
                    }
                };

                #[cfg(not(any(target_os = "windows", target_os = "linux")))]
                let (read_bps, write_bps) = (None, None);

                FileSystemInfo {
                    name,
                    mount_point,
                    fs_type: disk.file_system().to_string_lossy().into_owned(),
                    total_bytes: disk.total_space(),
                    available_bytes: disk.available_space(),
                    read_bps,
                    write_bps,
                }
            })
            .collect()
    }
}

/// Helper function to parse CSV formatted output from Windows `typeperf` utility.
#[cfg(target_os = "windows")]
fn parse_typeperf(stdout: &str) -> HashMap<String, (u64, u64)> {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() < 2 {
        return HashMap::new();
    }
    let header_line = lines[0];
    let values_line = lines[1];

    fn parse_csv_line(line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut in_quotes = false;
        let mut current = String::new();
        for c in line.chars() {
            match c {
                '"' => in_quotes = !in_quotes,
                ',' if !in_quotes => {
                    fields.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        fields.push(current.trim().to_string());
        fields
    }

    let headers = parse_csv_line(header_line);
    let values = parse_csv_line(values_line);

    if headers.len() != values.len() {
        return HashMap::new();
    }

    let mut drive_stats = HashMap::new();

    for i in 1..headers.len() {
        let header = &headers[i];
        let val_str = &values[i];
        let value = val_str.parse::<f64>().unwrap_or(0.0) as u64;

        if let Some(start_idx) = header.find("LogicalDisk(") {
            let sub = &header[start_idx + "LogicalDisk(".len()..];
            if let Some(end_idx) = sub.find(')') {
                let drive = sub[..end_idx].to_string(); // e.g. "C:"
                if drive == "_Total" || drive.contains("HarddiskVolume") {
                    continue;
                }
                let is_write = header.contains("Disk Write Bytes/sec");
                let stats = drive_stats.entry(drive).or_insert((0, 0));
                if is_write {
                    stats.1 = value;
                } else {
                    stats.0 = value;
                }
            }
        }
    }

    drive_stats
}

