# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-06-11

This initial release brings workable version, add stability improvements, cross-platform fixes, layout optimizations, and code documentation to `nmon-rs`.

### Added
- **Global Documentation Comments**: Added file-level (`//!`) and function-level/struct-level (`///`) documentation comments.
- **Unit Conversion Utilities**: Implemented `KB!()`, `MB!()`, and `GB!()` macros and helper functions (`bytes_to_kb`, `bytes_to_mb`, `bytes_to_gb`) in `src/metrics/mod.rs` to encapsulate byte-unit conversions and eliminate raw numeric literals (`1024.0`).

### Changed
- **Dynamic Process List Layout (`t`)**:
  - The process list now dynamically expands to fill the entire remaining vertical terminal window height.
- **Disk %Busy Map Refinement**:
  - Refactored `diskmap_widget.rs` to calculate `%Busy` based on real-time I/O activity (estimated from read/write speed).

### Fixed
- **Unicode Slicing Panic**: Replaced unsafe byte-based string slicing (`[0..N]`) with character-based Unicode-safe truncation (`.chars().take(N).collect()`) in `top_widget.rs`, `disk_widget.rs`, and `filesystem_widget.rs`. This prevents runtime panics when processing commands or mount paths containing multi-byte characters (e.g. Chinese characters).
- **Windows nameless drives display**: Added a fallback in `metrics/disk.rs` to use drive mount points (e.g., `C:`) when volume names/labels are empty, ensuring disk map widgets and filesystem graphs display properly on Windows.
