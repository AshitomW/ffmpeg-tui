use crate::app::ApplicationState;
use crate::domain::StreamInfo;
use crate::ui::theme::{active_border_style, inactive_border_style};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

pub fn render_inspector(frame: &mut Frame, area: Rect, state: &ApplicationState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Min(8),
        ])
        .split(area);

    let path_str = state
        .inspector
        .current_path
        .as_ref()
        .map_or("No file selected. Press [i] to select or inspect input file.", |p| p.to_str().unwrap_or(""));

    let header = Paragraph::new(Line::from(vec![
        Span::styled("Inspecting File: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(path_str),
    ]))
    .block(
        Block::default()
            .title(" Media Inspector (ffprobe) ")
            .borders(Borders::ALL)
            .border_style(active_border_style()),
    );
    frame.render_widget(header, chunks[0]);

    if state.inspector.loading {
        let loading_p = Paragraph::new("Running ffprobe inspection... Please wait.").block(
            Block::default()
                .title(" Analysis ")
                .borders(Borders::ALL)
                .border_style(inactive_border_style()),
        );
        frame.render_widget(loading_p, chunks[1]);
        return;
    }

    if let Some(err) = &state.inspector.error {
        let error_p = Paragraph::new(format!("Inspection Error: {err}")).block(
            Block::default()
                .title(" Analysis Failed ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );
        frame.render_widget(error_p, chunks[1]);
        return;
    }

    if let Some(info) = &state.inspector.media_info {
        let duration_str = info.duration_string();
        let size_str = info
            .file_size()
            .map_or("N/A".to_string(), |s| format!("{:.2} MB", s as f64 / 1024.0 / 1024.0));
        let bitrate_str = info
            .format
            .bit_rate
            .map_or("N/A".to_string(), |b| format!("{} kbps", b / 1000));

        let summary = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Format: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(&info.format.format_name),
                Span::raw(" ("),
                Span::raw(info.format.format_long_name.as_deref().unwrap_or("N/A")),
                Span::raw(")"),
            ]),
            Line::from(vec![
                Span::styled("Duration: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(duration_str),
                Span::raw(" | "),
                Span::styled("Size: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(size_str),
                Span::raw(" | "),
                Span::styled("Bitrate: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(bitrate_str),
            ]),
            Line::from(vec![
                Span::styled("Resolution: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(info.resolution_string()),
                Span::raw(" | "),
                Span::styled("Total Streams: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(info.streams.len().to_string()),
            ]),
        ])
        .block(
            Block::default()
                .title(" Container Metadata ")
                .borders(Borders::ALL)
                .border_style(inactive_border_style()),
        );
        frame.render_widget(summary, chunks[1]);

        let rows: Vec<Row> = info
            .streams
            .iter()
            .map(|stream| match stream {
                StreamInfo::Video(v) => Row::new(vec![
                    Cell::from(v.index.to_string()).style(Style::default().fg(Color::Cyan)),
                    Cell::from("Video").style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Cell::from(v.codec_name.clone()),
                    Cell::from(format!("{}x{} ({})", v.width, v.height, v.resolution_class())),
                    Cell::from(v.frame_rate.map_or("-".to_string(), |f| format!("{f:.2} fps"))),
                    Cell::from(v.pix_fmt.as_deref().unwrap_or("-")),
                ]),
                StreamInfo::Audio(a) => Row::new(vec![
                    Cell::from(a.index.to_string()).style(Style::default().fg(Color::Cyan)),
                    Cell::from("Audio").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Cell::from(a.codec_name.clone()),
                    Cell::from(a.channel_description()),
                    Cell::from(a.sample_rate_khz().map_or("-".to_string(), |r| format!("{r:.1} kHz"))),
                    Cell::from(a.bit_rate.map_or("-".to_string(), |b| format!("{} kbps", b / 1000))),
                ]),
                StreamInfo::Subtitle(s) => Row::new(vec![
                    Cell::from(s.index.to_string()).style(Style::default().fg(Color::Cyan)),
                    Cell::from("Subtitle").style(Style::default().fg(Color::Magenta)),
                    Cell::from(s.codec_name.clone()),
                    Cell::from(s.language.as_deref().unwrap_or("Unknown")),
                    Cell::from("-"),
                    Cell::from("-"),
                ]),
                StreamInfo::Data(d) => Row::new(vec![
                    Cell::from(d.index.to_string()).style(Style::default().fg(Color::Cyan)),
                    Cell::from("Data"),
                    Cell::from(d.codec_name.clone()),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("-"),
                ]),
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(6),
                Constraint::Length(10),
                Constraint::Percentage(20),
                Constraint::Percentage(30),
                Constraint::Percentage(20),
                Constraint::Percentage(14),
            ],
        )
        .header(
            Row::new(vec!["#", "Type", "Codec", "Details", "Rate", "Extra"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        )
        .block(
            Block::default()
                .title(" Streams ")
                .borders(Borders::ALL)
                .border_style(active_border_style()),
        );

        frame.render_widget(table, chunks[2]);
    } else {
        let no_data_p = Paragraph::new("No inspection data available. Select a file and press [i] to inspect.").block(
            Block::default()
                .title(" Analysis ")
                .borders(Borders::ALL)
                .border_style(inactive_border_style()),
        );
        frame.render_widget(no_data_p, chunks[1]);
    }
}
