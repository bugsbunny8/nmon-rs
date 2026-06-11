//! CPU long-term utilization rendering widget.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

#[allow(dead_code)]
/// Historical snapshot representing CPU utilization metrics breakdown.
#[derive(Clone, Copy, Debug)]
pub struct CpuSnap {
    /// Percentage user space CPU usage.
    pub user: f32,
    /// Percentage kernel space CPU usage.
    pub sys: f32,
    /// Percentage CPU waiting for I/O.
    pub wait: f32,
    /// Percentage idle time.
    pub idle: f32,
    /// Percentage virtual steal time.
    pub steal: f32,
}

/// Renders the long-term CPU usage trends as a scrolling historical bar chart.
pub fn render(f: &mut Frame, history: &[CpuSnap], cursor_idx: usize, area: Rect) {
    let block = Block::default()
        .title(" CPU Long-Term Averages ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = Vec::new();

    // Render 20 rows of the graph (from 100% down to 5%, step 5%)
    for row_idx in (0..20).rev() {
        let pct = (row_idx + 1) * 5;
        let label = format!("{:>3}%-|", pct);
        let mut spans = vec![Span::styled(label, Style::default().fg(Color::DarkGray))];

        for col in 0..72 {
            if col == cursor_idx {
                // Moving vertical cursor line
                spans.push(Span::styled("|", Style::default().fg(Color::White).bold()));
            } else if col < history.len() {
                let snap = &history[col];
                if snap.user < 0.0 {
                    // Unwritten history slot
                    spans.push(Span::raw(" "));
                } else {
                    let user_h = (snap.user / 5.0).round() as usize;
                    let sys_h = (snap.sys / 5.0).round() as usize;
                    let wait_h = (snap.wait / 5.0).round() as usize;
                    let idle_h = (snap.idle / 5.0).round() as usize;
                    
                    let current_row = row_idx;

                    if current_row < user_h {
                        spans.push(Span::styled("U", Style::default().bg(Color::Green).fg(Color::Black)));
                    } else if current_row < user_h + sys_h {
                        spans.push(Span::styled("s", Style::default().bg(Color::Red).fg(Color::Black)));
                    } else if current_row < user_h + sys_h + wait_h {
                        spans.push(Span::styled("w", Style::default().bg(Color::Blue).fg(Color::Black)));
                    } else if current_row < user_h + sys_h + wait_h + idle_h {
                        spans.push(Span::raw(" "));
                    } else {
                        spans.push(Span::styled("S", Style::default().fg(Color::Magenta)));
                    }
                }
            } else {
                spans.push(Span::raw(" "));
            }
        }
        spans.push(Span::styled("|", Style::default().fg(Color::DarkGray)));
        lines.push(Line::from(spans));
    }

    // Axis Bottom Row (representing the 0% line with User/System/Wait labels)
    let mut bottom_spans = vec![Span::styled("     +", Style::default().fg(Color::DarkGray))];

    for col in 0..72 {
        if col == cursor_idx {
            // The moving '+' cursor on the 0% line
            bottom_spans.push(Span::styled("+", Style::default().fg(Color::White).bold()));
        } else {
            let (ch, style) = if col >= 20 && col < 24 {
                // "User"
                let c = match col - 20 {
                    0 => 'U',
                    1 => 'u',
                    2 => 's',
                    3 => 'r',
                    _ => ' ',
                };
                (c, Style::default().fg(Color::Green).bold())
            } else if col >= 33 && col < 39 {
                // "System"
                let c = match col - 33 {
                    0 => 'S',
                    1 => 'y',
                    2 => 's',
                    3 => 't',
                    4 => 'e',
                    5 => 'm',
                    _ => ' ',
                };
                (c, Style::default().fg(Color::Red).bold())
            } else if col >= 47 && col < 51 {
                // "Wait"
                let c = match col - 47 {
                    0 => 'W',
                    1 => 'a',
                    2 => 'i',
                    3 => 't',
                    _ => ' ',
                };
                (c, Style::default().fg(Color::Blue).bold())
            } else {
                // Dash or separator
                let c = if col == 29 { '+' } else { '-' };
                (c, Style::default().fg(Color::DarkGray))
            };
            bottom_spans.push(Span::styled(ch.to_string(), style));
        }
    }

    bottom_spans.push(Span::styled("+", Style::default().fg(Color::DarkGray)));
    lines.push(Line::from(bottom_spans));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
