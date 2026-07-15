use crate::app::Screen;
use crate::ui::theme::{active_header_style, header_style};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Tabs};

pub fn render_nav_bar(frame: &mut Frame, area: Rect, current_screen: Screen) {
    let screens = [
        (Screen::Dashboard, "1: Dashboard"),
        (Screen::Builder, "2: Builder"),
        (Screen::Queue, "3: Job Queue"),
        (Screen::Logs, "4: Logs"),
        (Screen::Inspector, "5: Inspector"),
        (Screen::Help, "?: Help"),
    ];

    let titles: Vec<Line> = screens
        .iter()
        .map(|(screen, label)| {
            if *screen == current_screen {
                Line::from(Span::styled(format!(" {label} "), active_header_style()))
            } else {
                Line::from(Span::styled(format!(" {label} "), header_style()))
            }
        })
        .collect();

    let selected = screens
        .iter()
        .position(|(s, _)| *s == current_screen)
        .unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .title(" FFmpeg TUI ")
                .title_style(header_style()),
        )
        .select(selected)
        .highlight_style(active_header_style());

    frame.render_widget(tabs, area);
}
