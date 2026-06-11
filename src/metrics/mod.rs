//! Metrics gathering and logging components.
//!
//! This module includes collectors for CPU, disk, memory, and network metrics,
//! defines the structured metrics snapshot, and implements the CSV logger.

pub mod cpu;
pub mod disk;
pub mod memory;
pub mod network;
pub mod snapshot;
pub mod csv_logger;

/// Macro representing 1 KB (1024.0 bytes)
#[macro_export]
macro_rules! KB {
    () => { 1024.0 };
}

/// Macro representing 1 MB (1024.0 * 1024.0 bytes)
#[macro_export]
macro_rules! MB {
    () => { 1024.0 * $crate::KB!() };
}

/// Macro representing 1 GB (1024.0 * 1024.0 * 1024.0 bytes)
#[macro_export]
macro_rules! GB {
    () => { 1024.0 * $crate::MB!() };
}

/// Converts bytes (f64) to Kilobytes (KB).
#[inline]
pub fn bytes_to_kb(bytes: f64) -> f64 {
    bytes / crate::KB!()
}

/// Converts bytes (f64) to Megabytes (MB).
#[inline]
pub fn bytes_to_mb(bytes: f64) -> f64 {
    bytes / crate::MB!()
}

/// Converts bytes (f64) to Gigabytes (GB).
#[inline]
pub fn bytes_to_gb(bytes: f64) -> f64 {
    bytes / crate::GB!()
}
