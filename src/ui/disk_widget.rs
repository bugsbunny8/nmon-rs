//! Disk utilization rendering widget.

use crate::metrics::snapshot::MetricSnapshot;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::cell::Cell;

/// Renders the disk utilization SMP style graphs (d) or tabular filesystem list (D) with read/write bandwidth information.
pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, area: Rect, show_disk: u8, _peak_disk_kb: &Cell<f64>) {
    let io = &snapshot.disk_io;
    
    // Check if logical I/O speeds are available
    let has_any_logical_io = snapshot.filesystems.iter().any(|fs| fs.read_bps.is_some() || fs.write_bps.is_some());

    // Calculate system read/write totals based on sum of logical drives if available
    let (read_kb, write_kb) = if has_any_logical_io {
        let total_r: u64 = snapshot.filesystems.iter().map(|fs| fs.read_bps.unwrap_or(0)).sum();
        let total_w: u64 = snapshot.filesystems.iter().map(|fs| fs.write_bps.unwrap_or(0)).sum();
        (crate::metrics::bytes_to_kb(total_r as f64), crate::metrics::bytes_to_kb(total_w as f64))
    } else {
        (crate::metrics::bytes_to_kb(io.read_bps as f64), crate::metrics::bytes_to_kb(io.write_bps as f64))
    };
    let _total_kb = read_kb + write_kb;

    if show_disk == 2 {
        // Detailed Disk Stats Table Mode
        let block = Block::default()
            .title(" Disk Stats ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
            
        let mut lines = Vec::new();

        // Table Header
        lines.push(Line::from(vec![
            Span::styled(format!("{:<15}", "Disk Name"), Style::default().bold()),
            Span::styled(format!("{:>14}", "Read (KB/s)"), Style::default().fg(Color::Green).bold()),
            Span::styled(format!("{:>14}", "Write (KB/s)"), Style::default().fg(Color::Red).bold()),
            Span::styled(format!("{:>14}", "Total Size"), Style::default().bold()),
            Span::styled(format!("{:>14}", "Free Space"), Style::default().bold()),
        ]));

        // Calculate total summed space across filesystems
        let total_sys_bytes: u64 = snapshot.filesystems.iter().map(|fs| fs.total_bytes).sum();
        let free_sys_bytes: u64 = snapshot.filesystems.iter().map(|fs| fs.available_bytes).sum();
        let total_sys_gb = crate::metrics::bytes_to_gb(total_sys_bytes as f64);
        let free_sys_gb = crate::metrics::bytes_to_gb(free_sys_bytes as f64);

        // System aggregate row
        lines.push(Line::from(vec![
            Span::styled(format!("{:<15}", "system (total)"), Style::default().fg(Color::Cyan)),
            Span::raw(format!("{:>14.1}", read_kb)),
            Span::raw(format!("{:>14.1}", write_kb)),
            Span::raw(format!("{:>12.1} GB", total_sys_gb)),
            Span::raw(format!("{:>12.1} GB", free_sys_gb)),
        ]));

        // Filesystems storage space and per-logical-drive throughput
        for fs in &snapshot.filesystems {
            let total_gb = crate::metrics::bytes_to_gb(fs.total_bytes as f64);
            let free_gb = crate::metrics::bytes_to_gb(fs.available_bytes as f64);
            
            // Limit filesystem name length safely using char count to avoid UTF-8 indexing panics
            let display_name: String = if fs.name.chars().count() > 14 {
                fs.name.chars().take(14).collect()
            } else {
                fs.name.clone()
            };

            let read_str = match fs.read_bps {
                Some(bps) => format!("{:>14.1}", crate::metrics::bytes_to_kb(bps as f64)),
                None => format!("{:>14}", "N/A"),
            };
            let write_str = match fs.write_bps {
                Some(bps) => format!("{:>14.1}", crate::metrics::bytes_to_kb(bps as f64)),
                None => format!("{:>14}", "N/A"),
            };

            lines.push(Line::from(vec![
                Span::raw(format!("{:<15}", display_name)),
                Span::raw(read_str),
                Span::raw(write_str),
                Span::raw(format!("{:>12.1} GB", total_gb)),
                Span::raw(format!("{:>12.1} GB", free_gb)),
            ]));
        }

        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, area);
        return;
    }

    // Default Graph Mode (show_disk == 1)
    let block = Block::default()
        .title(" Disk I/O ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = Vec::new();

    // Line 1: Header Scale matching classic nmon spacing
    lines.push(Line::from(vec![
        Span::styled("DiskName Busy  Read Write|", Style::default().bold()),
        Span::styled("0----------25-----------50----------75---------100|", Style::default().fg(Color::DarkGray)),
    ]));

    // Render a row for each filesystem/partition
    for fs in &snapshot.filesystems {
        let fs_read_kb = crate::metrics::bytes_to_kb(fs.read_bps.unwrap_or(0) as f64);
        let fs_write_kb = crate::metrics::bytes_to_kb(fs.write_bps.unwrap_or(0) as f64);
        let fs_total_kb = fs_read_kb + fs_write_kb;

        // Calculate Busy% (est as total_kb / 10MB/s * 100)
        let busy_pct = (fs_total_kb / (10.0 * crate::KB!()) * 100.0).min(100.0);
        let busy_str = format!("{:>3.0}%", busy_pct);

        // Calculate bar width (50 chars total)
        let max_bar_width = 50;
        let busy_chars = ((busy_pct / 100.0) * max_bar_width as f64).round() as usize;

        let r_chars = if fs_total_kb > 0.0 {
            (((fs_read_kb / fs_total_kb) * busy_chars as f64).round() as usize).min(busy_chars)
        } else {
            0
        };
        let w_chars = busy_chars.saturating_sub(r_chars);

        // Format name safely using char count to avoid UTF-8 indexing panics (limit to 8 chars)
        let display_name: String = if fs.name.chars().count() > 8 {
            fs.name.chars().take(8).collect()
        } else {
            fs.name.clone()
        };

        // Prefix string length = 8 (display_name) + 5 (busy_str) + 6 (fs_read_kb) + 6 (fs_write_kb) + 1 (|) = 26 chars
        let mut row_spans = vec![
            Span::raw(format!("{:<8}{:>5}{:>6.1}{:>6.1}|", display_name, busy_str, fs_read_kb, fs_write_kb)),
        ];

        for _ in 0..r_chars {
            row_spans.push(Span::styled("R", Style::default().bg(Color::Green).fg(Color::Black)));
        }
        for _ in 0..w_chars {
            row_spans.push(Span::styled("W", Style::default().bg(Color::Red).fg(Color::Black)));
        }
        let remaining = max_bar_width - busy_chars;
        for _ in 0..remaining {
            row_spans.push(Span::raw(" "));
        }
        row_spans.push(Span::styled("|", Style::default().fg(Color::DarkGray)));

        lines.push(Line::from(row_spans));
    }

    // Totals line at the bottom
    let total_read_mb = read_kb / crate::KB!();
    let total_write_mb = write_kb / crate::KB!();
    let transfers = (read_kb + write_kb) / 4.0; // default IOPS estimate

    lines.push(Line::from(vec![
        Span::styled(
            format!(
                "Totals Read-MB/s={:<9.1} Writes-MB/s={:<9.1} Transfers/sec={:<6.1}",
                total_read_mb, total_write_mb, transfers
            ),
            Style::default().bold().fg(Color::Cyan)
        )
    ]));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

