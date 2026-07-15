use crate::domain::JobStatus;
use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub active_border: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            background: Color::Reset,
            surface: Color::Reset,
            border: Color::DarkGray,
            active_border: Color::Cyan,
            text: Color::White,
            muted: Color::Gray,
            accent: Color::Yellow,
        }
    }
}

pub fn status_color(status: &JobStatus) -> Color {
    match status {
        JobStatus::Pending => Color::Yellow,
        JobStatus::Queued => Color::Blue,
        JobStatus::Running { .. } => Color::Cyan,
        JobStatus::Paused { .. } => Color::Magenta,
        JobStatus::Completed { .. } => Color::Green,
        JobStatus::Failed { .. } => Color::Red,
        JobStatus::Cancelled => Color::DarkGray,
    }
}

pub fn status_style(status: &JobStatus) -> Style {
    Style::default()
        .fg(status_color(status))
        .add_modifier(Modifier::BOLD)
}

pub fn header_style() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

pub fn active_header_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

pub fn active_border_style() -> Style {
    Style::default().fg(Color::Cyan)
}

pub fn inactive_border_style() -> Style {
    Style::default().fg(Color::DarkGray)
}
