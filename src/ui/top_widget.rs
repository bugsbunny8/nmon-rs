//! Top active processes rendering widget.

use crate::metrics::snapshot::MetricSnapshot;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Renders the list of running processes sorted dynamically by either CPU usage percentage or memory consumption.
pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, area: Rect, sort_by_cpu: bool) {
    let block = Block::default()
        .title(format!(
            " Top Processes (Sorted by {}) ",
            if sort_by_cpu { "CPU%" } else { "Memory" }
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = Vec::new();

    // Table Header
    lines.push(Line::from(vec![
        Span::styled(format!("{:<8}", "PID"), Style::default().bold()),
        Span::styled(format!("{:>8}", "CPU%"), Style::default().fg(Color::Green).bold()),
        Span::styled(format!("{:>12}", "Mem (MB)"), Style::default().fg(Color::Yellow).bold()),
        Span::styled(format!("{:>12}", "Read(KB/s)"), Style::default().fg(Color::LightGreen).bold()),
        Span::styled(format!("{:>12}", "Write(KB/s)"), Style::default().fg(Color::LightRed).bold()),
        Span::raw("  Command"),
    ]));

    // Copy and sort processes
    let mut procs = snapshot.processes.clone();
    if sort_by_cpu {
        procs.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        procs.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    }

    // Limit processes display to fill the available height dynamically (excluding borders and header line)
    let max_rows = (area.height.saturating_sub(3)) as usize;
    let display_count = procs.len().min(max_rows);

    for proc in procs.iter().take(display_count) {
        let mem_mb = crate::metrics::bytes_to_mb(proc.memory_bytes as f64);
        let read_kb = crate::metrics::bytes_to_kb(proc.disk_read_bps as f64);
        let write_kb = crate::metrics::bytes_to_kb(proc.disk_write_bps as f64);

        // Truncate command name to fit terminal space safely (using char boundaries)
        let command_max_len = (area.width.saturating_sub(58)) as usize;
        let cmd_display = if proc.command.chars().count() > command_max_len {
            let truncated: String = proc.command.chars().take(command_max_len.saturating_sub(3)).collect();
            format!("{}...", truncated)
        } else {
            proc.command.clone()
        };

        lines.push(Line::from(vec![
            Span::raw(format!("{:<8}", proc.pid)),
            Span::styled(format!("{:>7.1}%", proc.cpu_usage), Style::default().fg(Color::Green)),
            Span::styled(format!("{:>12.1}", mem_mb), Style::default().fg(Color::Yellow)),
            Span::styled(format!("{:>12.1}", read_kb), Style::default().fg(Color::LightGreen)),
            Span::styled(format!("{:>12.1}", write_kb), Style::default().fg(Color::LightRed)),
            Span::styled(format!("  {}", cmd_display), Style::default().fg(Color::Cyan)),
        ]));
    }

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
