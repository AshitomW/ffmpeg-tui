use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

pub fn render_help(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10)])
        .split(area);

    let rows = vec![
        Row::new(vec![Cell::from("Global"), Cell::from("1 - 5"), Cell::from("Switch screen (1:Dashboard, 2:Builder, 3:Queue, 4:Logs, 5:Inspector)")]),
        Row::new(vec![Cell::from("Global"), Cell::from("? / h"), Cell::from("Open Help Screen")]),
        Row::new(vec![Cell::from("Global"), Cell::from("q / Esc"), Cell::from("Quit application")]),
        Row::new(vec![Cell::from("Builder"), Cell::from("Tab / Shift+Tab"), Cell::from("Navigate between configuration fields")]),
        Row::new(vec![Cell::from("Builder"), Cell::from("Left / Right"), Cell::from("Cycle through options for selected field")]),
        Row::new(vec![Cell::from("Builder"), Cell::from("r"), Cell::from("Toggle Raw Command Mode")]),
        Row::new(vec![Cell::from("Builder"), Cell::from("p"), Cell::from("Load preset")]),
        Row::new(vec![Cell::from("Builder"), Cell::from("b / Enter"), Cell::from("Build & Queue Job")]),
        Row::new(vec![Cell::from("Queue"), Cell::from("Up / Down"), Cell::from("Select job in queue")]),
        Row::new(vec![Cell::from("Queue"), Cell::from("s / p"), Cell::from("Start / Pause queue processing")]),
        Row::new(vec![Cell::from("Queue"), Cell::from("c"), Cell::from("Cancel selected job")]),
        Row::new(vec![Cell::from("Queue"), Cell::from("r"), Cell::from("Retry failed job")]),
        Row::new(vec![Cell::from("Queue"), Cell::from("K / J"), Cell::from("Move selected job Up / Down in queue")]),
        Row::new(vec![Cell::from("Queue"), Cell::from("x"), Cell::from("Clear completed jobs")]),
        Row::new(vec![Cell::from("Logs"), Cell::from("t"), Cell::from("Toggle between clean progress and raw FFmpeg log view")]),
        Row::new(vec![Cell::from("Logs"), Cell::from("a"), Cell::from("Toggle AutoScroll")]),
        Row::new(vec![Cell::from("Logs"), Cell::from("Up / Down"), Cell::from("Scroll log view")]),
        Row::new(vec![Cell::from("Inspector"), Cell::from("i"), Cell::from("Inspect builder input file")]),
    ];

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(20),
            Constraint::Percentage(60),
        ],
    )
    .header(
        Row::new(vec!["Context", "Keybinding", "Action Description"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .title(" Keyboard Shortcuts & User Guide ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(table, chunks[0]);
}
