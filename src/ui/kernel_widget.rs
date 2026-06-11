//! Kernel statistics rendering widget.

use crate::metrics::snapshot::MetricSnapshot;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Renders kernel metrics, system load averages, and uptime values.
pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, area: Rect) {
    let mut lines = Vec::new();

    let block = Block::default()
        .title(" Kernel Stats ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let sys = &snapshot.system;

    // Format Uptime
    let days = sys.uptime_secs / (24 * 3600);
    let hours = (sys.uptime_secs % (24 * 3600)) / 3600;
    let mins = (sys.uptime_secs % 3600) / 60;
    let secs = sys.uptime_secs % 60;

    let uptime_str = if days > 0 {
        format!("{} days, {:02}:{:02}:{:02}", days, hours, mins, secs)
    } else {
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    };

    // Load Average
    // Handle N/A on platforms that don't support it (e.g. Windows)
    let load_str = if sys.load_avg_1m > 0.0 || sys.load_avg_5m > 0.0 || sys.load_avg_15m > 0.0 {
        format!("{:.2}, {:.2}, {:.2}", sys.load_avg_1m, sys.load_avg_5m, sys.load_avg_15m)
    } else {
        "N/A (Not supported on this platform)".to_string()
    };

    lines.push(Line::from(vec![
        Span::styled("Load Average: ", Style::default().bold()),
        Span::styled(load_str, Style::default().fg(Color::Green)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("System Uptime: ", Style::default().bold()),
        Span::styled(uptime_str, Style::default().fg(Color::Yellow)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Kernel Info:   ", Style::default().bold()),
        Span::styled(format!("{} (Arch: {})", sys.kernel_version, sys.cpu_arch), Style::default().fg(Color::Magenta)),
    ]));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
