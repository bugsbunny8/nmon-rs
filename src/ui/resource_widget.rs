//! System hardware resources and operating system metadata rendering widget.

use crate::metrics::snapshot::MetricSnapshot;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Renders system details, including operating system version, CPU architecture, model name, vendor, and core counts.
pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, area: Rect) {
    let mut lines = Vec::new();

    let block = Block::default()
        .title(" System Resources ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let sys = &snapshot.system;

    lines.push(Line::from(vec![
        Span::styled("OS Name:       ", Style::default().bold()),
        Span::styled(&sys.os_name, Style::default().fg(Color::Green)),
        Span::styled("   |   OS Version: ", Style::default().bold()),
        Span::styled(&sys.os_version, Style::default().fg(Color::Yellow)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("CPU Model:     ", Style::default().bold()),
        Span::styled(&sys.cpu_model, Style::default().fg(Color::Cyan)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("CPU Vendor:    ", Style::default().bold()),
        Span::styled(&sys.cpu_vendor, Style::default().fg(Color::Magenta)),
        Span::styled("   |   Architecture: ", Style::default().bold()),
        Span::styled(&sys.cpu_arch, Style::default().fg(Color::LightBlue)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Physical Cores:", Style::default().bold()),
        Span::styled(format!(" {:<10}", sys.physical_cores), Style::default().fg(Color::Green)),
        Span::styled("   |   Logical Cores: ", Style::default().bold()),
        Span::styled(format!("{}", sys.logical_cores), Style::default().fg(Color::Green)),
    ]));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
