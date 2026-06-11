//! Virtual memory and swap statistics rendering widget.

use crate::metrics::snapshot::MetricSnapshot;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Renders virtual memory metrics, including total/free/used swap sizes, a usage bar chart, and Linux paging metrics if available.
pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, area: Rect) {
    let block = Block::default()
        .title(" Virtual Memory ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = Vec::new();

    let mem = &snapshot.memory;
    let total_swap_mb = mem.total_swap as f64 / 1024.0 / 1024.0;
    let free_swap_mb = mem.free_swap as f64 / 1024.0 / 1024.0;
    let used_swap_mb = total_swap_mb - free_swap_mb;
    let used_swap_pct = if mem.total_swap > 0 {
        (used_swap_mb / total_swap_mb) * 100.0
    } else {
        0.0
    };

    lines.push(Line::from(vec![
        Span::styled("Swap Space:  ", Style::default().bold()),
        Span::raw(format!("Total: {:.1} MB | Free: {:.1} MB | Used: {:.1} MB ({:.1}%)", total_swap_mb, free_swap_mb, used_swap_mb, used_swap_pct)),
    ]));

    // Draw swap usage progress bar
    let mut bar_spans = vec![Span::raw("Swap Usage: [")];
    let bar_width = 30;
    let filled = ((used_swap_pct / 100.0) * bar_width as f64).round() as usize;
    let empty = bar_width - filled;
    for _ in 0..filled {
        bar_spans.push(Span::styled("#", Style::default().fg(Color::Yellow)));
    }
    for _ in 0..empty {
        bar_spans.push(Span::raw(" "));
    }
    bar_spans.push(Span::raw("]"));
    lines.push(Line::from(bar_spans));

    // Try to read Linux /proc/vmstat details if available
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/vmstat") {
            let mut vm_stats = std::collections::HashMap::new();
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 2 {
                    if let Ok(val) = parts[1].parse::<u64>() {
                        vm_stats.insert(parts[0], val);
                    }
                }
            }

            let pgpgin = vm_stats.get("pgpgin").copied().unwrap_or(0);
            let pgpgout = vm_stats.get("pgpgout").copied().unwrap_or(0);
            let pgfault = vm_stats.get("pgfault").copied().unwrap_or(0);
            let pgmajfault = vm_stats.get("pgmajfault").copied().unwrap_or(0);

            lines.push(Line::from(Span::styled("--------------------------------------------------------------------------------", Style::default().fg(Color::DarkGray))));
            lines.push(Line::from(vec![
                Span::raw("Linux Paging: pgpgin="),
                Span::styled(pgpgin.to_string(), Style::default().fg(Color::Green)),
                Span::raw(" | pgpgout="),
                Span::styled(pgpgout.to_string(), Style::default().fg(Color::Red)),
                Span::raw(" | pgfault="),
                Span::styled(pgfault.to_string(), Style::default().fg(Color::Yellow)),
                Span::raw(" | pgmajfault="),
                Span::styled(pgmajfault.to_string(), Style::default().fg(Color::LightRed)),
            ]));
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        lines.push(Line::from(Span::styled("--------------------------------------------------------------------------------", Style::default().fg(Color::DarkGray))));
        lines.push(Line::from(vec![
            Span::raw("Swap Paging Stats: "),
            Span::styled("N/A (Linux /proc/vmstat stats only)", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
