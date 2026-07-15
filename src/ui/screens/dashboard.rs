use crate::ui::theme::{active_border_style, inactive_border_style, status_color};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

pub fn render_dashboard(frame: &mut Frame, area: Rect, jobs_data: &[crate::domain::Job]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(6),
        ])
        .split(area);

    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(chunks[0]);

    let pending_count = jobs_data
        .iter()
        .filter(|j| matches!(j.status(), crate::domain::JobStatus::Queued | crate::domain::JobStatus::Pending))
        .count();
    let active_count = jobs_data
        .iter()
        .filter(|j| j.status().is_running())
        .count();
    let completed_count = jobs_data
        .iter()
        .filter(|j| matches!(j.status(), crate::domain::JobStatus::Completed { .. }))
        .count();
    let failed_count = jobs_data
        .iter()
        .filter(|j| matches!(j.status(), crate::domain::JobStatus::Failed { .. }))
        .count();

    render_stat_card(frame, stats_chunks[0], "PENDING", &pending_count.to_string(), Color::Yellow);
    render_stat_card(frame, stats_chunks[1], "ACTIVE", &active_count.to_string(), Color::Cyan);
    render_stat_card(frame, stats_chunks[2], "COMPLETED", &completed_count.to_string(), Color::Green);
    render_stat_card(frame, stats_chunks[3], "FAILED", &failed_count.to_string(), Color::Red);

    let rows: Vec<Row> = jobs_data
        .iter()
        .take(10)
        .map(|job| {
            let status_str = job.status().name();
            let status_col = status_color(job.status());
            let progress_str = match job.progress() {
                Some(p) => format!("{:.1}%", p.percentage),
                None => "-".to_string(),
            };

            Row::new(vec![
                Cell::from(job.id().to_string()).style(Style::default().fg(Color::Cyan)),
                Cell::from(job.input_name()),
                Cell::from(job.output_name()),
                Cell::from(status_str).style(Style::default().fg(status_col).add_modifier(Modifier::BOLD)),
                Cell::from(progress_str),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Length(12),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["ID", "Input File", "Output File", "Status", "Progress"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .title(" Recent Jobs ")
            .borders(Borders::ALL)
            .border_style(active_border_style()),
    );

    frame.render_widget(table, chunks[1]);

    let quick_info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Quick Navigation: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("[2] Job Builder  |  [3] Queue Manager  |  [4] Log Viewer  |  [5] File Inspector"),
        ]),
        Line::from(vec![
            Span::styled("Queue Controls:   ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("[s] Start Queue  |  [p] Pause Queue  |  [c] Cancel Job  |  [r] Retry Failed"),
        ]),
    ])
    .block(
        Block::default()
            .title(" Quick Tips ")
            .borders(Borders::ALL)
            .border_style(inactive_border_style()),
    );

    frame.render_widget(quick_info, chunks[2]);
}

fn render_stat_card(frame: &mut Frame, area: Rect, title: &str, value: &str, color: Color) {
    let card = Paragraph::new(vec![
        Line::from(Span::styled(title, Style::default().fg(color).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(value, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color)),
    );
    frame.render_widget(card, area);
}
