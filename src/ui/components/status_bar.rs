use crate::app::ApplicationState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn render_status_bar(frame: &mut Frame, area: Rect, state: &ApplicationState) {
    let status_text: String = if let Some(msg) = &state.status_message {
        let s: String = format!(" Status: {msg} ");
        s
    } else {
        " Keys: [1-5]: Screens | [q]: Quit | [Tab]: Focus | [Space]: Toggle ".to_string()
    };

    let paragraph = Paragraph::new(Line::from(vec![
        Span::styled(
            status_text,
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::raw(" "),
    ]));

    frame.render_widget(paragraph, area);
}
