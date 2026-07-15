use crate::app::ApplicationState;
use crate::ui::theme::{active_border_style, inactive_border_style};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render_logs(
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
        ])
        .split(area);

    let selected_job_id = state.logs_state.selected_job;
    let selected_job = jobs_data
        .iter()
        .find(|j| Some(j.id()) == selected_job_id)
        .or_else(|| jobs_data.first());

    let job_title = match selected_job {
        Some(j) => format!(" Job: {} ({}) ", j.id(), j.status().name()),
        None => "No Job Selected".to_string(),
    };

    let view_mode = if state.logs_state.show_raw_output {
        "RAW LOGS"
    } else {
        "CLEAN PROGRESS"
    };

    let autoscroll_str = if state.logs_state.auto_scroll {
        "ON"
    } else {
        "OFF"
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(job_title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | Mode: "),
        Span::styled(view_mode, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" | AutoScroll: "),
        Span::styled(autoscroll_str, Style::default().fg(Color::Green)),
        Span::raw(" | [t]: Toggle Raw [a]: AutoScroll [↑/↓]: Scroll"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(inactive_border_style()),
    );
    frame.render_widget(header, chunks[0]);

    if let Some(job) = selected_job {
        let logs = job.log();
        if logs.is_empty() {
            let empty_p = Paragraph::new("No log output recorded yet for this job.").block(
                Block::default()
                    .title(" Output Log ")
                    .borders(Borders::ALL)
                    .border_style(active_border_style()),
            );
            frame.render_widget(empty_p, chunks[1]);
        } else {
            let items: Vec<ListItem> = if state.logs_state.show_raw_output {
                logs.iter()
                    .map(|line| {
                        let color = if line.contains("Error") || line.contains("error") {
                            Color::Red
                        } else if line.contains("frame=") || line.contains("time=") {
                            Color::Cyan
                        } else {
                            Color::Gray
                        };
                        ListItem::new(Span::styled(line.as_str(), Style::default().fg(color)))
                    })
                    .collect()
            } else {
                logs.iter()
                    .filter(|l| l.contains("frame=") || l.contains("time=") || l.contains("Error") || l.contains("error"))
                    .map(|line| {
                        let color = if line.contains("Error") { Color::Red } else { Color::Cyan };
                        ListItem::new(Span::styled(line.as_str(), Style::default().fg(color)))
                    })
                    .collect()
            };

            let total_items = items.len();
            let scroll = if state.logs_state.auto_scroll {
                total_items.saturating_sub(chunks[1].height as usize - 2)
            } else {
                state.logs_state.scroll_position
            };

            let visible_items: Vec<ListItem> = items.into_iter().skip(scroll).collect();

            let log_list = List::new(visible_items).block(
                Block::default()
                    .title(format!(" Logs ({total_items} lines) "))
                    .borders(Borders::ALL)
                    .border_style(active_border_style()),
            );

            frame.render_widget(log_list, chunks[1]);
        }
    } else {
        let no_job_p = Paragraph::new("No jobs available in queue to display logs.").block(
            Block::default()
                .title(" Output Log ")
                .borders(Borders::ALL)
                .border_style(inactive_border_style()),
        );
        frame.render_widget(no_job_p, chunks[1]);
    }
}
