//! Logic for evaluating system metric snapshots against rules and handling alert outputs.

use crate::alerting::rules::default_rules;
use crate::metrics::snapshot::MetricSnapshot;
use std::fs::OpenOptions;
use std::io::Write;

/// Evaluates a system metric snapshot against default alert rules and writes triggered alerts to 'alerts.log'.
pub fn evaluate_alerts(snapshot: &MetricSnapshot) {
    let rules = default_rules();
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("alerts.log")
        .expect("Unable to open alert log file");

    for rule in rules {
        if (rule.check)(snapshot) {
            let log_entry = format!(
                "[ALERT] {} triggered at {}. Threshold: {}\n",
                rule.name,
                snapshot.timestamp.format("%Y-%m-%d %H:%M:%S"),
                rule.threshold
            );
            log_file
                .write_all(log_entry.as_bytes())
                .expect("Failed to write to alert log file");
        }
    }
}
