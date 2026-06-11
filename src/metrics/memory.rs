//! Memory metrics collection module using the sysinfo crate.

use sysinfo::System;
use super::snapshot::MemorySnapshot;

/// Collector for gathering RAM and Swap memory utilization.
pub struct MemoryCollector {
    sys: System,
}

impl MemoryCollector {
    /// Creates a new `MemoryCollector` and refreshes the initial memory state.
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_memory();
        MemoryCollector { sys }
    }

    /// Refreshes and returns a snapshot of the current RAM and Swap memory usage.
    pub fn collect(&mut self) -> MemorySnapshot {
        self.sys.refresh_memory();
        let total_ram = self.sys.total_memory();
        let used_ram = self.sys.used_memory();
        let free_ram = total_ram.saturating_sub(used_ram);

        let total_swap = self.sys.total_swap();
        let used_swap = self.sys.used_swap();
        let free_swap = total_swap.saturating_sub(used_swap);

        MemorySnapshot {
            total_ram,
            free_ram,
            used_ram,
            total_swap,
            free_swap,
            used_swap,
        }
    }
}
