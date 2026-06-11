//! Terminal User Interface (TUI) components and dashboard widgets.
//!
//! This module defines widgets for individual resource views (CPU, Memory, Disk, etc.)
//! and assembles them into the main interactive dashboard layout.

pub mod cpu_widget;
pub mod dashboard;
pub mod disk_widget;
pub mod memory_widget;
pub mod net_widget;
pub mod filesystem_widget;
pub mod kernel_widget;
pub mod resource_widget;
pub mod top_widget;
pub mod longterm_widget;
pub mod virtual_memory_widget;
pub mod diskmap_widget;
pub mod theme;
