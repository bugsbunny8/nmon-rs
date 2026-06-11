//! CPU metrics collection module using the sysinfo crate.

use sysinfo::System;
use super::snapshot::CpuInfo;

/// Collector for gathering CPU metrics (total and core-level usage, frequencies, brand, cores count).
pub struct CpuCollector {
    sys: System,
}

impl CpuCollector {
    /// Creates a new `CpuCollector` and refreshes the initial CPU state.
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        CpuCollector { sys }
    }

    /// Refreshes CPU metrics and returns a tuple containing the global CPU usage percentage
    /// and a vector of information for each individual CPU core.
    pub fn collect(&mut self) -> (f32, Vec<CpuInfo>) {
        self.sys.refresh_cpu_all();
        let global = self.sys.global_cpu_usage();
        let cores = self.sys
            .cpus()
            .iter()
            .map(|cpu| CpuInfo {
                name: cpu.name().to_string(),
                usage: cpu.cpu_usage(),
                frequency: cpu.frequency(),
            })
            .collect();
        (global, cores)
    }

    /// Returns the CPU brand/model name as a String.
    pub fn cpu_brand(&self) -> String {
        self.sys
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string())
    }

    /// Returns the CPU vendor ID identifier as a String.
    pub fn cpu_vendor(&self) -> String {
        self.sys
            .cpus()
            .first()
            .map(|cpu| cpu.vendor_id().to_string())
            .unwrap_or_else(|| "Unknown Vendor".to_string())
    }

    /// Returns the number of logical cores.
    pub fn logical_cores(&self) -> usize {
        self.sys.cpus().len()
    }

    /// Returns the number of physical cores. Fallbacks to logical cores count if unavailable.
    pub fn physical_cores(&self) -> usize {
        sysinfo::System::physical_core_count().unwrap_or(self.logical_cores())
    }
}
