//! Main dashboard TUI screen orchestration and rendering layout logic.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use crate::metrics::snapshot::MetricSnapshot;
use super::{
    cpu_widget, memory_widget, disk_widget, filesystem_widget,
    net_widget, kernel_widget, resource_widget, top_widget,
    longterm_widget::{self, CpuSnap},
    virtual_memory_widget, diskmap_widget,
};

/// UI display toggles and state settings.
pub struct UiState {
    pub show_help: bool,
    pub show_cpu: bool,
    pub show_memory: bool,
    pub show_disk: u8, // 0 = None, 1 = Graph (d), 2 = Stats (D)
    pub show_filesystem: bool,
    pub show_network: bool,
    pub show_kernel: bool,
    pub show_resources: bool,
    pub show_processes: bool,
    pub show_longterm: bool,
    pub show_vm: bool,
    pub show_diskmap: bool,
    pub process_sort_by_cpu: bool,
    pub refresh_interval_secs: u64,
    pub peak_disk_kb: std::cell::Cell<f64>,
}

impl UiState {
    /// Returns true if no toggle sections are active, indicating the welcome screen should be shown.
    pub fn is_welcome(&self) -> bool {
        !self.show_help
            && !self.show_cpu
            && !self.show_memory
            && self.show_disk == 0
            && !self.show_filesystem
            && !self.show_network
            && !self.show_kernel
            && !self.show_resources
            && !self.show_processes
            && !self.show_longterm
            && !self.show_vm
            && !self.show_diskmap
    }
}

/// Renders the entire dashboard header, toggled resource widgets, and layout boxes based on `UiState`.
pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, state: &UiState, history: &[CpuSnap], history_cursor: usize) {
    let area = f.area();

    // Overall vertical layouts: Header is always 1 line.
    // Rest is for main dashboard.
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    // Draw header (nmon title, Hostname, Refresh, Clock)
    let header_spans = vec![
        Span::styled("nmon-rs ", Style::default().bold().fg(Color::Cyan)),
        Span::raw("16s   "),
        Span::raw("Hostname="),
        Span::styled(&snapshot.system.hostname, Style::default().fg(Color::Green)),
        Span::raw("   Refresh="),
        Span::styled(format!("{}s", state.refresh_interval_secs), Style::default().fg(Color::Yellow)),
        Span::raw("      "),
        Span::raw(snapshot.timestamp.format("%H:%M:%S").to_string()),
    ];
    let header_line = Line::from(header_spans);
    f.render_widget(Paragraph::new(header_line), main_layout[0]);

    let display_area = main_layout[1];

    if state.is_welcome() {
        render_welcome(f, snapshot, state, display_area);
        return;
    }

    // Determine active sections
    let mut active_sections = Vec::new();

    if state.show_help {
        active_sections.push((SectionType::Help, 10));
    }
    if state.show_cpu {
        let height = (snapshot.cpu_cores.len() + 4) as u16;
        active_sections.push((SectionType::Cpu, height));
    }
    if state.show_longterm {
        active_sections.push((SectionType::LongTerm, 23));
    }
    if state.show_memory {
        active_sections.push((SectionType::Memory, 9));
    }
    if state.show_vm {
        active_sections.push((SectionType::VirtualMemory, 8));
    }
    if state.show_disk > 0 {
        let height = if state.show_disk == 2 {
            (snapshot.filesystems.len() + 5) as u16
        } else {
            (snapshot.filesystems.len() + 4) as u16
        };
        active_sections.push((SectionType::Disk, height));
    }
    if state.show_diskmap {
        active_sections.push((SectionType::DiskMap, 5));
    }
    if state.show_filesystem {
        let height = (snapshot.filesystems.len() + 4) as u16;
        active_sections.push((SectionType::FileSystem, height));
    }
    if state.show_network {
        let height = (snapshot.networks.len() + 4) as u16;
        active_sections.push((SectionType::Network, height));
    }
    if state.show_kernel {
        active_sections.push((SectionType::Kernel, 5));
    }
    if state.show_resources {
        active_sections.push((SectionType::Resources, 7));
    }
    if state.show_processes {
        active_sections.push((SectionType::Processes, 15));
    }

    // Build constraints layout
    let mut constraints = Vec::new();
    let has_processes = active_sections.iter().any(|(sec, _)| matches!(sec, SectionType::Processes));
    for &(sec, height) in &active_sections {
        if let SectionType::Processes = sec {
            constraints.push(Constraint::Min(5));
        } else {
            constraints.push(Constraint::Length(height));
        }
    }
    // Add extra constraint to consume overflow cleanly only if Processes is not taking up the remaining space
    if !has_processes {
        constraints.push(Constraint::Min(0));
    }

    let layout_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(display_area);

    // Render active widgets
    for (i, &(sec_type, _)) in active_sections.iter().enumerate() {
        let render_area = layout_rects[i];
        if render_area.height == 0 {
            continue; // Skip if no space
        }
        match sec_type {
            SectionType::Help => render_mini_help(f, render_area),
            SectionType::Cpu => cpu_widget::render(f, snapshot, render_area),
            SectionType::LongTerm => longterm_widget::render(f, history, history_cursor, render_area),
            SectionType::Memory => memory_widget::render(f, snapshot, render_area),
            SectionType::VirtualMemory => virtual_memory_widget::render(f, snapshot, render_area),
            SectionType::Disk => disk_widget::render(f, snapshot, render_area, state.show_disk, &state.peak_disk_kb),
            SectionType::DiskMap => diskmap_widget::render(f, snapshot, render_area),
            SectionType::FileSystem => filesystem_widget::render(f, snapshot, render_area),
            SectionType::Network => net_widget::render(f, snapshot, render_area),
            SectionType::Kernel => kernel_widget::render(f, snapshot, render_area),
            SectionType::Resources => resource_widget::render(f, snapshot, render_area),
            SectionType::Processes => top_widget::render(f, snapshot, render_area, state.process_sort_by_cpu),
        }
    }
}

/// Internal representation of each toggleable screen widget section.
#[derive(Debug, Clone, Copy)]
enum SectionType {
    Help,
    Cpu,
    LongTerm,
    Memory,
    VirtualMemory,
    Disk,
    DiskMap,
    FileSystem,
    Network,
    Kernel,
    Resources,
    Processes,
}

/// Renders the dashboard welcome and key-toggles helper page on startup.
fn render_welcome(f: &mut Frame, snapshot: &MetricSnapshot, _state: &UiState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = Vec::new();

    // ASCII Art Logo
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("   ------------------------------", Style::default().fg(Color::Green))));
    lines.push(Line::from(Span::styled("    _ __  _ __ ___   ___  _ __   ", Style::default().fg(Color::Green))));
    lines.push(Line::from(Span::styled("   | '_ \\| '_ ` _ \\ / _ \\| '_ \\  ", Style::default().fg(Color::Green))));
    lines.push(Line::from(Span::styled("   | | | | | | | | | (_) | | | | ", Style::default().fg(Color::Green))));
    lines.push(Line::from(Span::styled("   |_| |_|_| |_| |_|\\___/|_| |_| ", Style::default().fg(Color::Green))));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("   ------------------------------", Style::default().fg(Color::Green))));
    lines.push(Line::from(""));

    // Quick hint
    lines.push(Line::from(vec![
        Span::raw("   For help type "),
        Span::styled("h", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" or review the toggles below."),
    ]));
    lines.push(Line::from(vec![
        Span::raw("   To stop nmon-rs type "),
        Span::styled("q", Style::default().fg(Color::Red).bold()),
        Span::raw(" to Quit."),
    ]));
    lines.push(Line::from(""));

    // System info summary
    lines.push(Line::from(vec![
        Span::styled("   Host Information: ", Style::default().bold()),
        Span::styled(&snapshot.system.hostname, Style::default().fg(Color::Cyan)),
        Span::raw(" | OS: "),
        Span::styled(&snapshot.system.os_name, Style::default().fg(Color::Green)),
        Span::raw(" | Uptime: "),
        Span::styled(format!("{} hours", snapshot.system.uptime_secs / 3600), Style::default().fg(Color::Yellow)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("   CPU Specs:        ", Style::default().bold()),
        Span::styled(&snapshot.system.cpu_model, Style::default().fg(Color::LightBlue)),
        Span::raw(format!(" (Cores: {} physical, {} logical)", snapshot.system.physical_cores, snapshot.system.logical_cores)),
    ]));
    lines.push(Line::from(""));

    // Toggle Menu
    lines.push(Line::from(Span::styled("   Use these keys to toggle statistics on/off:", Style::default().fg(Color::Cyan))));
    lines.push(Line::from("     c = CPU Utilisation SMP view    | l = CPU Long-term averages"));
    lines.push(Line::from("     m = Memory & Swap               | V = Virtual Memory stats"));
    lines.push(Line::from("     d = Disk Aggregate I/O Graph    | D = Disk detailed Stats"));
    lines.push(Line::from("     o = Disks %Busy Map             | j = File Systems (JFS)"));
    lines.push(Line::from("     n = Network Interfaces          | t = Top processes list"));
    lines.push(Line::from("     r = System Resources metadata   | k = Kernel / Loadavg / Uptime"));
    lines.push(Line::from(""));
    lines.push(Line::from("     Space = Refresh screen now      | h = Mini Help & Toggles list"));
    lines.push(Line::from("     q = Quit program                | + = Double refresh / - = Halve"));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

/// Renders a small help menu bar listing current keyboard shortcut commands.
fn render_mini_help(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Help: Toggles Menu ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let lines = vec![
        Line::from("  Interactive Toggles:"),
        Line::from("    c = CPU Util   l = Long-term  m = Memory     V = Virtual Mem  d = Disk Graph  D = Disk Stats"),
        Line::from("    o = Disk Map   j = Filesystem n = Network    k = Kernel       r = Resources   t = Processes"),
        Line::from("  Process Sorting (Top Processes must be active):"),
        Line::from("    4 = Sort by Memory   5 = Sort by CPU%"),
        Line::from("  Controls:"),
        Line::from("    Space = Force Refresh Now   q = Quit"),
        Line::from("    + = Slower Updates          - = Faster Updates"),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
