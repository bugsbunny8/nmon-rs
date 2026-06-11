//! Network interface bandwidth statistics collector.

use sysinfo::Networks;
use std::collections::HashMap;
use super::snapshot::NetworkInterfaceInfo;

/// Collector for gathering network interface RX/TX traffic rates.
pub struct NetworkCollector {
    networks: Networks,
    prev_rx: HashMap<String, u64>,
    prev_tx: HashMap<String, u64>,
}

impl NetworkCollector {
    /// Creates a new `NetworkCollector` and populates the initial interface data.
    pub fn new() -> Self {
        let networks = Networks::new_with_refreshed_list();
        let mut prev_rx = HashMap::new();
        let mut prev_tx = HashMap::new();

        for (name, data) in &networks {
            prev_rx.insert(name.clone(), data.total_received());
            prev_tx.insert(name.clone(), data.total_transmitted());
        }

        Self {
            networks,
            prev_rx,
            prev_tx,
        }
    }

    /// Refreshes network stats and returns a list of bandwidth statistics for each active interface.
    pub fn collect(&mut self) -> Vec<NetworkInterfaceInfo> {
        self.networks.refresh(true);
        let mut results = Vec::new();

        for (name, data) in &self.networks {
            let rx = data.total_received();
            let tx = data.total_transmitted();

            let old_rx = self.prev_rx.get(name).copied().unwrap_or(rx);
            let old_tx = self.prev_tx.get(name).copied().unwrap_or(tx);

            let rx_bytes_sec = rx.saturating_sub(old_rx);
            let tx_bytes_sec = tx.saturating_sub(old_tx);

            self.prev_rx.insert(name.clone(), rx);
            self.prev_tx.insert(name.clone(), tx);

            results.push(NetworkInterfaceInfo {
                name: name.clone(),
                rx_bytes_sec,
                tx_bytes_sec,
            });
        }

        results.sort_by(|a, b| a.name.cmp(&b.name));
        results
    }
}
