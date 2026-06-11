//! Memory usage statistics rendering widget.

use crate::metrics::snapshot::MetricSnapshot;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Renders RAM and Swap utilization tabular stats including Total, Free, and Free Percent values.
pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, area: Rect) {
    let mut lines = Vec::new();

    let block = Block::default()
        .title(" Memory & Swap ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mem = &snapshot.memory;

    // Convert bytes to MB
    let total_ram_mb = mem.total_ram as f64 / 1024.0 / 1024.0;
    let free_ram_mb = mem.free_ram as f64 / 1024.0 / 1024.0;
    let ram_free_pct = if mem.total_ram > 0 {
        (mem.free_ram as f64 / mem.total_ram as f64) * 100.0
    } else {
        0.0
    };

    let total_swap_mb = mem.total_swap as f64 / 1024.0 / 1024.0;
    let free_swap_mb = mem.free_swap as f64 / 1024.0 / 1024.0;
    let swap_free_pct = if mem.total_swap > 0 {
        (mem.free_swap as f64 / mem.total_swap as f64) * 100.0
    } else {
        0.0
    };

    // Row 1: Header
    lines.push(Line::from(vec![
        Span::styled(format!("{:<15}", "Memory Option"), Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{:>14}", "RAM-Memory"), Style::default().fg(Color::Green).bold()),
        Span::styled(format!("{:>14}", "Swap-Space"), Style::default().fg(Color::Yellow).bold()),
        Span::styled(format!("{:>14}", "High-Memory"), Style::default().fg(Color::Blue)),
        Span::styled(format!("{:>14}", "Low-Memory"), Style::default().fg(Color::Magenta)),
    ]));

    // Row 2: Total
    lines.push(Line::from(vec![
        Span::raw(format!("{:<15}", "Total (MB)")),
        Span::raw(format!("{:>14.1}", total_ram_mb)),
        Span::raw(format!("{:>14.1}", total_swap_mb)),
        Span::raw(format!("{:>14}", " - not in use")),
        Span::raw(format!("{:>14}", " - not in use")),
    ]));

    // Row 3: Free
    lines.push(Line::from(vec![
        Span::raw(format!("{:<15}", "Free  (MB)")),
        Span::raw(format!("{:>14.1}", free_ram_mb)),
        Span::raw(format!("{:>14.1}", free_swap_mb)),
        Span::raw(format!("{:>14}", "")),
        Span::raw(format!("{:>14}", "")),
    ]));

    // Row 4: Free Percent
    lines.push(Line::from(vec![
        Span::raw(format!("{:<15}", "Free Percent")),
        Span::styled(format!("{:>13.1}%", ram_free_pct), Style::default().fg(Color::Green)),
        Span::styled(format!("{:>13.1}%", swap_free_pct), Style::default().fg(Color::Yellow)),
        Span::raw(format!("{:>14}", "")),
        Span::raw(format!("{:>14}", "")),
    ]));

    // Row 5: Divider
    lines.push(Line::from(Span::styled("--------------------------------------------------------------------------------", Style::default().fg(Color::DarkGray))));

    // Row 6: Platform notes/extra info
    let used_ram_mb = (mem.total_ram - mem.free_ram) as f64 / 1024.0 / 1024.0;
    let used_swap_mb = (mem.total_swap - mem.free_swap) as f64 / 1024.0 / 1024.0;
    lines.push(Line::from(vec![
        Span::raw("Used RAM: "),
        Span::styled(format!("{:.1} MB", used_ram_mb), Style::default().fg(Color::LightGreen)),
        Span::raw("   |   Used Swap: "),
        Span::styled(format!("{:.1} MB", used_swap_mb), Style::default().fg(Color::LightYellow)),
        Span::raw("   |   OS Memory Unit: "),
        Span::styled("MB", Style::default().fg(Color::Cyan)),
    ]));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
