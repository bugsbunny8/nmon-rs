//! Alert rules configuration for monitoring system metrics.

use crate::metrics::snapshot::MetricSnapshot;

/// Represents an alert rule checking system resource metrics against thresholds.
pub struct AlertRule {
    /// The name of the alert rule.
    pub name: &'static str,
    /// The warning/critical threshold value.
    pub threshold: f64,
    /// The checker function taking a snapshot and returning true if triggered.
    pub check: fn(&MetricSnapshot) -> bool,
}

/// Returns the default list of alert rules (e.g., high CPU usage, high Memory usage).
pub fn default_rules() -> Vec<AlertRule> {
    vec![
        AlertRule {
            name: "High CPU Usage",
            threshold: 85.0,
            check: |snap| snap.cpu_global > 85.0,
        },
        AlertRule {
            name: "High Memory Usage",
            threshold: 90.0,
            check: |snap| {
                if snap.memory.total_ram == 0 {
                    false
                } else {
                    (snap.memory.used_ram as f64 / snap.memory.total_ram as f64) * 100.0 > 90.0
                }
            },
        },
    ]
}
