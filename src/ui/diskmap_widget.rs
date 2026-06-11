//! Disk %Busy map rendering widget.

use crate::metrics::snapshot::MetricSnapshot;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Renders the disk busy map dashboard segment mapping volume load or usage percentage to specific characters.
pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, area: Rect) {
    let block = Block::default()
        .title(" Disk %Busy Map ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = Vec::new();

    // 1. Key Legend
    lines.push(Line::from(vec![
        Span::styled("Key: ", Style::default().bold()),
        Span::styled("_ = 0%", Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::styled(". = 5%", Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::styled("- = 10%", Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::styled("+ = 20%", Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::styled("o = 30%", Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled("0 = 40%", Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled("O = 50%", Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled("8 = 60%", Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled("X = 70%", Style::default().fg(Color::Red)),
        Span::raw(" | "),
        Span::styled("# = 80%", Style::default().fg(Color::Red)),
        Span::raw(" | "),
        Span::styled("@ = 90%+", Style::default().fg(Color::Red).bold()),
    ]));

    lines.push(Line::from(Span::styled("--------------------------------------------------------------------------------", Style::default().fg(Color::DarkGray))));

    // nmon busy map characters definition
    let map_chars = "_____.....----------++++++++++oooooooooo0000000000OOOOOOOOOO8888888888XXXXXXXXXX##########@@@@@@@@@@*";

    // Build the grid representing disk activity as a busy map
    let mut spans = vec![Span::styled("Disk Map:  ", Style::default().bold())];

    for fs in &snapshot.filesystems {
        if fs.total_bytes == 0 {
            continue;
        }
        let fs_read_kb = crate::metrics::bytes_to_kb(fs.read_bps.unwrap_or(0) as f64);
        let fs_write_kb = crate::metrics::bytes_to_kb(fs.write_bps.unwrap_or(0) as f64);
        let fs_total_kb = fs_read_kb + fs_write_kb;

        // Calculate Busy% (est as total_kb / 10MB/s * 100)
        let busy_pct = (fs_total_kb / (10.0 * crate::KB!() as f64) * 100.0).min(100.0);
        let used_pct = busy_pct.round() as usize;
        let char_idx = used_pct.min(100);
        let map_char = map_chars.chars().nth(char_idx).unwrap_or('_');

        let char_color = if used_pct >= 90 {
            Color::Red
        } else if used_pct >= 70 {
            Color::LightRed
        } else if used_pct >= 30 {
            Color::Yellow
        } else {
            Color::Green
        };

        // Format: fs_name [map_char]
        spans.push(Span::raw(format!("{} [", fs.name)));
        spans.push(Span::styled(map_char.to_string(), Style::default().fg(char_color).bold().bg(Color::Black)));
        spans.push(Span::raw("]   "));
    }

    lines.push(Line::from(spans));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
