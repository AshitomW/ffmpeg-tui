use crate::app::ApplicationState;
use crate::ui::components::render_progress_bar;
use crate::ui::theme::{active_border_style, inactive_border_style, status_color};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

pub fn render_queue(
    frame: &mut Frame,
    area: Rect,
    state: &ApplicationState,
    jobs_data: &[crate::domain::Job],
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(6),
        ])
        .split(area);

    let queue_status = if state.queue_state.is_paused {
        Span::styled(
            " QUEUE PAUSED ",
            Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " QUEUE RUNNING ",
            Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
        )
    };

    let header = Paragraph::new(Line::from(vec![
        queue_status,
        Span::raw("  "),
        Span::styled(
            "[s] Start  [p] Pause  [c] Cancel  [r] Retry  [K/J] Reorder  [x] Clear Completed",
            Style::default().fg(Color::Cyan),
        ),
    ]))
    .block(
        Block::default()
            .title(" Queue Control ")
            .borders(Borders::ALL)
            .border_style(inactive_border_style()),
    );
    frame.render_widget(header, chunks[0]);

    let selected_index = state.queue_state.selected_index.min(jobs_data.len().saturating_sub(1));

    let rows: Vec<Row> = jobs_data
        .iter()
        .enumerate()
        .map(|(idx, job)| {
            let is_selected = idx == selected_index;
            let status_str = job.status().name();
            let status_col = status_color(job.status());

            let progress_str = match job.progress() {
                Some(p) => format!("{:.1}% ({} fps)", p.percentage, p.fps),
                None => match job.status() {
                    crate::domain::JobStatus::Completed { output_size, .. } => {
                        format!("Done ({} MB)", output_size / 1024 / 1024)
                    }
                    crate::domain::JobStatus::Failed { error, .. } => {
                        format!("Error: {error}")
                    }
                    _ => "-".to_string(),
                },
            };

            let row_style = if is_selected {
                Style::default().bg(Color::Rgb(30, 40, 60)).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if is_selected { "> " } else { "  " };

            Row::new(vec![
                Cell::from(format!("{}{}", prefix, job.id())).style(Style::default().fg(Color::Cyan)),
                Cell::from(job.config().priority.to_string()),
                Cell::from(job.input_name()),
                Cell::from(job.output_name()),
                Cell::from(status_str).style(Style::default().fg(status_col).add_modifier(Modifier::BOLD)),
                Cell::from(progress_str),
            ])
            .style(row_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Length(12),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["ID", "Priority", "Input", "Output", "Status", "Progress / Result"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .title(" Job Queue ([Up]/[Down] to select job) ")
            .borders(Borders::ALL)
            .border_style(active_border_style()),
    );

    frame.render_widget(table, chunks[1]);

    if let Some(selected_job) = jobs_data.get(selected_index) {
        if let Some(progress) = selected_job.progress() {
            render_progress_bar(
                frame,
                chunks[2],
                progress,
                selected_job.source_duration(),
            );
        } else {
            let detail = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("Selected Job ID: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(selected_job.id().to_string()),
                    Span::raw(" | Status: "),
                    Span::styled(
                        selected_job.status().name(),
                        Style::default().fg(status_color(selected_job.status())).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Command: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(selected_job.config().command.to_command_string()),
                ]),
            ])
            .block(
                Block::default()
                    .title(" Job Detail ")
                    .borders(Borders::ALL)
                    .border_style(inactive_border_style()),
            );
            frame.render_widget(detail, chunks[2]);
        }
    } else {
        let empty_msg = Paragraph::new("No jobs in queue. Go to [2] Builder to create jobs.").block(
            Block::default()
                .title(" Job Detail ")
                .borders(Borders::ALL)
                .border_style(inactive_border_style()),
        );
        frame.render_widget(empty_msg, chunks[2]);
    }
}
