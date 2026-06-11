//! UI styling and theme configuration.

use ratatui::style::{Color, Style};
#[allow(dead_code)]
/// Returns the default Style (white foreground, black background).
pub fn default_style() -> Style {
    Style::default().fg(Color::White).bg(Color::Black)
}

