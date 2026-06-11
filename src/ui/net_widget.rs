//! Network interface bandwidth statistics rendering widget.

use crate::metrics::snapshot::MetricSnapshot;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Renders network bandwidth statistics, displaying RX/TX rates in KB/s and a log-scale traffic gauge per interface.
pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, area: Rect) {
    let mut lines = Vec::new();

    let block = Block::default()
        .title(" Network I/O ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Table Header
    lines.push(Line::from(vec![
        Span::styled(format!("{:<15}", "Interface"), Style::default().bold()),
        Span::styled(format!("{:>14}", "Recv (KB/s)"), Style::default().fg(Color::Green).bold()),
        Span::styled(format!("{:>14}", "Trans (KB/s)"), Style::default().fg(Color::Yellow).bold()),
        Span::raw("  Traffic Gauge"),
    ]));

    for net in &snapshot.networks {
        // Convert to KB/s
        let rx_kb = net.rx_bytes_sec as f64 / 1024.0;
        let tx_kb = net.tx_bytes_sec as f64 / 1024.0;
        let total_kb = rx_kb + tx_kb;

        let mut spans = vec![
            Span::raw(format!("{:<15}", net.name)),
            Span::styled(format!("{:>14.1}", rx_kb), Style::default().fg(Color::Green)),
            Span::styled(format!("{:>14.1}", tx_kb), Style::default().fg(Color::Yellow)),
            Span::raw(" ["),
        ];

        // Draw a small traffic gauge (log scale or simple threshold)
        // Let's use simple threshold bins: each '#' represents a power of 2 or linear step
        // We will do a 15-char gauge.
        let bar_width = 15;
        let filled = if total_kb > 1024.0 * 10.0 {
            bar_width // > 10MB/s is full
        } else if total_kb > 0.1 {
            // Logarithmic representation
            let ratio = (total_kb.log2().max(0.0) / (1024.0 * 10.0f64).log2()) * bar_width as f64;
            (ratio.round() as usize).clamp(1, bar_width)
        } else {
            0
        };
        let empty = bar_width - filled;

        for _ in 0..filled {
            spans.push(Span::styled("#", Style::default().fg(Color::Cyan)));
        }
        for _ in 0..empty {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::raw("]"));

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
