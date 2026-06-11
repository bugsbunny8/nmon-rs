//! Filesystem storage utilization rendering widget.

use crate::metrics::snapshot::MetricSnapshot;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Renders the JFS filesystem utilization list, displaying total size, used space, percentage used, and a graphical bar chart for each filesystem.
pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, area: Rect) {
    let mut lines = Vec::new();

    let block = Block::default()
        .title(" File Systems (JFS) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Table Header
    lines.push(Line::from(vec![
        Span::styled(format!("{:<15}", "Filesystem"), Style::default().bold()),
        Span::styled(format!("{:<12}", "Type"), Style::default().bold()),
        Span::styled(format!("{:<15}", "Mount Point"), Style::default().bold()),
        Span::styled(format!("{:>10}", "Total(GB)"), Style::default().bold()),
        Span::styled(format!("{:>10}", "Used(GB)"), Style::default().bold()),
        Span::styled(format!("{:>8}", "Used%"), Style::default().bold()),
        Span::raw("  Usage Graph"),
    ]));

    for fs in &snapshot.filesystems {
        // Convert to GB
        let total_gb = crate::metrics::bytes_to_gb(fs.total_bytes as f64);
        let free_gb = crate::metrics::bytes_to_gb(fs.available_bytes as f64);
        let used_gb = total_gb - free_gb;
        
        let used_pct = if fs.total_bytes > 0 {
            (used_gb / total_gb) * 100.0
        } else {
            0.0
        };

        // Render filesystem details
        // Truncate strings safely to prevent overflowing columns or panicking on Unicode boundaries
        let fs_name: String = if fs.name.chars().count() > 14 {
            fs.name.chars().take(14).collect()
        } else {
            fs.name.clone()
        };
        let fs_type: String = if fs.fs_type.chars().count() > 11 {
            fs.fs_type.chars().take(11).collect()
        } else {
            fs.fs_type.clone()
        };
        let fs_mount: String = if fs.mount_point.chars().count() > 14 {
            fs.mount_point.chars().take(14).collect()
        } else {
            fs.mount_point.clone()
        };

        let mut spans = vec![
            Span::raw(format!("{:<15}", fs_name)),
            Span::styled(format!("{:<12}", fs_type), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:<15}", fs_mount), Style::default().fg(Color::Magenta)),
            Span::raw(format!("{:>10.1}", total_gb)),
            Span::raw(format!("{:>10.1}", used_gb)),
            Span::styled(format!("{:>7.1}%", used_pct), if used_pct > 85.0 { Style::default().fg(Color::Red).bold() } else { Style::default().fg(Color::Green) }),
            Span::raw(" ["),
        ];

        // Draw a small 15-char usage bar
        let bar_width = 15;
        let filled = ((used_pct / 100.0) * bar_width as f64).round() as usize;
        let empty = bar_width - filled;
        
        let bar_style = if used_pct > 85.0 {
            Style::default().fg(Color::Red)
        } else if used_pct > 60.0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        };

        for _ in 0..filled {
            spans.push(Span::styled("#", bar_style));
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
