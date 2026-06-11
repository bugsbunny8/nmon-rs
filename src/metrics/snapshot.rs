//! System resource snapshot structures for metric serialization.

use chrono::{DateTime, Local};
use serde::Serialize;

/// Information about a single CPU core.
#[derive(Serialize, Debug, Clone)]
pub struct CpuInfo {
    /// Name of the CPU core (e.g., "cpu0").
    pub name: String,
    /// CPU core usage percentage.
    pub usage: f32,
    /// Operating frequency of the core in MHz.
    pub frequency: u64,
}

/// Consolidated system disk I/O metrics.
#[derive(Serialize, Debug, Clone)]
pub struct DiskIoInfo {
    /// Read throughput in bytes per second.
    pub read_bps: u64,
    /// Write throughput in bytes per second.
    pub write_bps: u64,
}

/// Statistics and metadata of a mounted filesystem.
#[derive(Serialize, Debug, Clone)]
pub struct FileSystemInfo {
    /// Filesystem volume name or identifier.
    pub name: String,
    /// Mount path.
    pub mount_point: String,
    /// Filesystem type (e.g., NTFS, ext4).
    pub fs_type: String,
    /// Total capacity in bytes.
    pub total_bytes: u64,
    /// Free capacity available to non-privileged users in bytes.
    pub available_bytes: u64,
    /// Read rate of this filesystem in bytes per second (if supported).
    pub read_bps: Option<u64>,
    /// Write rate of this filesystem in bytes per second (if supported).
    pub write_bps: Option<u64>,
}

/// Statistics for a network interface.
#[derive(Serialize, Debug, Clone)]
pub struct NetworkInterfaceInfo {
    /// Name of the network interface (e.g., "eth0").
    pub name: String,
    /// Received throughput in bytes per second.
    pub rx_bytes_sec: u64,
    /// Transmitted throughput in bytes per second.
    pub tx_bytes_sec: u64,
}

/// Resource usage and metadata of a running process.
#[derive(Serialize, Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID.
    pub pid: u32,
    /// Process executable name.
    pub name: String,
    /// CPU usage percentage of the process.
    pub cpu_usage: f32,
    /// Resident set size (RSS) memory consumption in bytes.
    pub memory_bytes: u64,
    /// Disk read rate in bytes per second.
    pub disk_read_bps: u64,
    /// Disk write rate in bytes per second.
    pub disk_write_bps: u64,
    /// Command line arguments of the process.
    pub command: String,
}

/// Static system metadata and load averages.
#[derive(Serialize, Debug, Clone)]
pub struct SystemResources {
    /// Host name.
    pub hostname: String,
    /// Operating system name.
    pub os_name: String,
    /// Operating system version.
    pub os_version: String,
    /// Kernel release version.
    pub kernel_version: String,
    /// CPU architecture.
    pub cpu_arch: String,
    /// CPU vendor identifier.
    pub cpu_vendor: String,
    /// CPU model name/brand.
    pub cpu_model: String,
    /// Count of physical CPU cores.
    pub physical_cores: usize,
    /// Count of logical CPU cores.
    pub logical_cores: usize,
    /// System uptime in seconds.
    pub uptime_secs: u64,
    /// 1-minute system load average.
    pub load_avg_1m: f64,
    /// 5-minute system load average.
    pub load_avg_5m: f64,
    /// 15-minute system load average.
    pub load_avg_15m: f64,
}

/// Memory and swap space statistics.
#[derive(Serialize, Debug, Clone)]
pub struct MemorySnapshot {
    /// Total RAM capacity in bytes.
    pub total_ram: u64,
    /// Free RAM capacity in bytes.
    pub free_ram: u64,
    /// Used RAM capacity in bytes.
    pub used_ram: u64,
    /// Total virtual swap capacity in bytes.
    pub total_swap: u64,
    /// Free virtual swap capacity in bytes.
    pub free_swap: u64,
    /// Used virtual swap capacity in bytes.
    pub used_swap: u64,
}

/// Comprehensive snapshot containing all gathered system performance metrics.
#[derive(Serialize, Debug, Clone)]
pub struct MetricSnapshot {
    /// Snapshot collection timestamp.
    pub timestamp: DateTime<Local>,
    /// Global CPU usage percentage.
    pub cpu_global: f32,
    /// Metrics per CPU core.
    pub cpu_cores: Vec<CpuInfo>,
    /// Memory usage snapshot.
    pub memory: MemorySnapshot,
    /// Aggregate disk I/O metrics.
    pub disk_io: DiskIoInfo,
    /// Filesystems statistics.
    pub filesystems: Vec<FileSystemInfo>,
    /// Network interfaces statistics.
    pub networks: Vec<NetworkInterfaceInfo>,
    /// List of running processes.
    pub processes: Vec<ProcessInfo>,
    /// Static system resource details.
    pub system: SystemResources,
}
