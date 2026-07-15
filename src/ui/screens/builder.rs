use crate::app::{ApplicationState, BuilderField};
use crate::ui::theme::{active_border_style, inactive_border_style};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render_builder(frame: &mut Frame, area: Rect, state: &ApplicationState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(5),
        ])
        .split(area);

    let mode_title = if state.builder.raw_command_mode {
        " Command Builder (RAW COMMAND MODE) "
    } else {
        " Command Builder (INTERACTIVE FORM) "
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(mode_title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" Press [r] to toggle Raw Command Mode | Press [p] to apply preset"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(inactive_border_style()),
    );
    frame.render_widget(header, chunks[0]);

    if state.builder.raw_command_mode {
        let raw_input = Paragraph::new(state.builder.raw_command.as_str()).block(
            Block::default()
                .title(" Raw FFmpeg Command (Edit directly) ")
                .borders(Borders::ALL)
                .border_style(active_border_style()),
        );
        frame.render_widget(raw_input, chunks[1]);
    } else {
        let form_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        let fields = BuilderField::all();
        let field_items: Vec<ListItem> = fields
            .iter()
            .map(|field| {
                let is_selected = state.builder.current_field == *field;
                let value_str = match field {
                    BuilderField::Input => state
                        .builder
                        .input_path
                        .as_ref()
                        .map_or("<None — Press Enter to browse>".to_string(), |p| {
                            p.display().to_string()
                        }),
                    BuilderField::Output => state
                        .builder
                        .output_path
                        .as_ref()
                        .map_or("<None — Press Enter to browse>".to_string(), |p| p.display().to_string()),
                    BuilderField::VideoCodec => state.builder.video_codec.to_string(),
                    BuilderField::AudioCodec => state.builder.audio_codec.to_string(),
                    BuilderField::Format => state.builder.format.to_string(),
                    BuilderField::Quality => format!("CRF {}", state.builder.crf),
                    BuilderField::Preset => state.builder.preset.to_string(),
                    BuilderField::Filters => {
                        let vf_count = state.builder.filters.video_filters().len();
                        let af_count = state.builder.filters.audio_filters().len();
                        let res: String = format!("{vf_count} video, {af_count} audio filter(s)");
                        res
                    }
                };

                let label_style = if is_selected {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let val_style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let prefix = if is_selected { "> " } else { "  " };

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, label_style),
                    Span::styled(format!("{:<18}: ", field.label()), label_style),
                    Span::styled(value_str, val_style),
                ]))
            })
            .collect();

        let form_list = List::new(field_items).block(
            Block::default()
                .title(" Configuration Fields ([Tab]/[Arrows] to navigate, [Left]/[Right] to adjust) ")
                .borders(Borders::ALL)
                .border_style(active_border_style()),
        );

        frame.render_widget(form_list, form_chunks[0]);

        let preview_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(form_chunks[1]);

        let in_p = state
            .builder
            .input_path
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from("input.mp4"));
        let out_p = state
            .builder
            .output_path
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from(format!("output.{}", state.builder.format.extension())));

        let cmd_preview = match crate::domain::CommandBuilder::new()
            .input(in_p)
            .output(out_p)
            .video_codec(state.builder.video_codec)
            .audio_codec(state.builder.audio_codec)
            .format(state.builder.format)
            .crf(state.builder.crf)
            .preset(state.builder.preset)
            .filters(state.builder.filters.clone())
            .build()
        {
            Ok(c) => c.to_command_string(),
            Err(e) => format!("Error: {e}"),
        };

        let cmd_box = Paragraph::new(cmd_preview)
            .wrap(ratatui::widgets::Wrap { trim: false })
            .block(
                Block::default()
                    .title(" Generated Command Preview ")
                    .borders(Borders::ALL)
                    .border_style(inactive_border_style()),
            );
        frame.render_widget(cmd_box, preview_chunks[0]);

        let selected_idx = state.builder.preset_index;
        let preset_items: Vec<ListItem> = state
            .presets
            .iter()
            .take(6)
            .enumerate()
            .map(|(i, p)| {
                let is_selected = i == selected_idx;
                let bullet = if is_selected { "▸" } else { " " };
                let name_style = if is_selected {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Yellow)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} {:<20}", bullet, p.name), name_style),
                    Span::raw(format!(" ({})", p.category.display_name())),
                ]))
            })
            .collect();

        let presets_box = List::new(preset_items).block(
            Block::default()
                .title(" Built-in Presets ([←/→] to select, [p] to apply) ")
                .borders(Borders::ALL)
                .border_style(if state.builder.current_field == BuilderField::Preset {
                    active_border_style()
                } else {
                    inactive_border_style()
                }),
        );
        frame.render_widget(presets_box, preview_chunks[1]);
    }

    let actions_box = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("BUILD JOB: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("Press [b] or [Enter] to queue this job"),
        ]),
        Line::from(vec![
            Span::styled("CONTROLS:  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("[←/→]: Change Option | [Enter]: Browse File / Edit Field / Build Job | [f]: Manage Filters | [r]: Raw Mode"),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(inactive_border_style()),
    );
    frame.render_widget(actions_box, chunks[2]);
}
