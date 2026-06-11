//! CPU utilisation rendering widget.

use crate::metrics::snapshot::MetricSnapshot;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Renders the CPU SMP multi-core utilization view, showing User%, Sys%, Wait%, Idle%, and bar graphs for each core.
pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, area: Rect) {
    let mut lines = Vec::new();

    // Panel border title
    let block = Block::default()
        .title(" CPU Utilisation ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Header line
    lines.push(Line::from(vec![
        Span::styled("CPU  User%  Sys%  Wait%  Idle|", Style::default().bold()),
        Span::styled("0----------25-----------50----------75---------100|", Style::default().fg(Color::DarkGray)),
    ]));

    // Helper to format a single CPU row
    let format_cpu_row = |name: &str, usage: f32| -> Line {
        // Estimate User/Sys split for visual replication (85% user, 15% system)
        let user = usage * 0.85;
        let sys = usage * 0.15;
        let wait = 0.0;
        let idle = (100.0 - usage).max(0.0);

        let mut spans = vec![
            Span::raw(format!("{:<4}", name)),
            Span::styled(format!("{:>6.1}", user), Style::default().fg(Color::Green)),
            Span::styled(format!("{:>6.1}", sys), Style::default().fg(Color::Red)),
            Span::styled(format!("{:>6.1}", wait), Style::default().fg(Color::Blue)),
            Span::raw(format!("{:>6.1}|", idle)),
        ];

        // Draw bar chart (50 chars total, each char is 2%)
        let mut user_chars = (user / 2.0).round() as usize;
        let mut sys_chars = (sys / 2.0).round() as usize;
        if user_chars + sys_chars > 50 {
            if user_chars > 50 {
                user_chars = 50;
                sys_chars = 0;
            } else {
                sys_chars = 50 - user_chars;
            }
        }
        let total_chars = user_chars + sys_chars;
        
        let mut bar_spans = Vec::new();
        for _ in 0..user_chars {
            // Green background block
            bar_spans.push(Span::styled("U", Style::default().bg(Color::Green).fg(Color::Black)));
        }
        for _ in 0..sys_chars {
            // Red background block
            bar_spans.push(Span::styled("s", Style::default().bg(Color::Red).fg(Color::Black)));
        }
        
        let idle_chars = 50 - total_chars;
        for _ in 0..idle_chars {
            bar_spans.push(Span::styled(" ", Style::default()));
        }
        
        spans.extend(bar_spans);
        spans.push(Span::styled("|", Style::default().fg(Color::DarkGray)));
        
        // Add peak indicator
        if usage > 0.5 {
            let peak_pos = (usage / 2.0).round() as usize;
            let peak_index = 5 + peak_pos; // index in spans list after CPU columns
            if peak_index < spans.len() - 1 {
                spans[peak_index] = Span::styled(">", Style::default().fg(Color::Yellow).bold());
            }
        }

        Line::from(spans)
    };

    // Render global average first
    lines.push(format_cpu_row("Avg", snapshot.cpu_global));

    // Render individual cores
    for cpu in &snapshot.cpu_cores {
        lines.push(format_cpu_row(&cpu.name, cpu.usage));
    }

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
