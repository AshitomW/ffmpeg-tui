use crate::app::ApplicationState;
use crate::app::filter_dialog::FilterTab;
use crate::ui::theme::{active_border_style, inactive_border_style};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs};

fn centered_popup(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}

pub fn render_filter_dialog(frame: &mut Frame, area: Rect, state: &ApplicationState) {
    let dialog = match &state.filter_dialog {
        Some(d) => d,
        None => return,
    };

    let popup = centered_popup(area, 75, 80);

    frame.render_widget(Clear, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(popup);

    let tab_titles: Vec<Line> = FilterTab::all()
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();

    let tab_index = FilterTab::all()
        .iter()
        .position(|&t| t == dialog.current_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .title(" Configure FFmpeg Filters ")
                .borders(Borders::ALL)
                .border_style(active_border_style()),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .select(tab_index);

    frame.render_widget(tabs, chunks[0]);

    // ── Presets / Active List ────────────────────────────────────
    match dialog.current_tab {
        FilterTab::Video => {
            render_preset_list(
                frame,
                chunks[1],
                " Available Video Filters ",
                &dialog.video_presets,
                dialog.selected_index,
            );
        }
        FilterTab::Audio => {
            render_preset_list(
                frame,
                chunks[1],
                " Available Audio Filters ",
                &dialog.audio_presets,
                dialog.selected_index,
            );
        }
        FilterTab::Active => {
            render_active_list(frame, chunks[1], state, dialog.selected_index);
        }
    }

    // ── Custom Input Bar ──────────────────────────────────────────
    let (custom_title, custom_style) = if dialog.editing_custom {
        (
            " Custom Filter String (EDITING - Type string & press Enter) ",
            active_border_style(),
        )
    } else {
        (
            " Custom Filter String (Select 'Custom' item & press Enter to edit) ",
            inactive_border_style(),
        )
    };

    let custom_display = if dialog.custom_text.is_empty() {
        if dialog.editing_custom {
            "_"
        } else {
            "<empty — type custom filter e.g. scale=1280:-1,fps=30>"
        }
    } else {
        dialog.custom_text.as_str()
    };

    let custom_p = Paragraph::new(custom_display).block(
        Block::default()
            .title(custom_title)
            .borders(Borders::ALL)
            .border_style(custom_style),
    );
    frame.render_widget(custom_p, chunks[2]);

    // ── Keyboard Legend Bar ────────────────────────────────────────
    let legend = Paragraph::new(Line::from(vec![
        Span::styled("[Tab]/[←/→]: ", Style::default().fg(Color::Yellow)),
        Span::raw("Tabs | "),
        Span::styled("[↑/↓]: ", Style::default().fg(Color::Yellow)),
        Span::raw("Navigate | "),
        Span::styled("[Enter]: ", Style::default().fg(Color::Green)),
        Span::raw("Add/Select | "),
        Span::styled("[d]: ", Style::default().fg(Color::Red)),
        Span::raw("Remove | "),
        Span::styled("[c]: ", Style::default().fg(Color::Red)),
        Span::raw("Clear All | "),
        Span::styled("[Esc]: ", Style::default().fg(Color::Gray)),
        Span::raw("Close"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(inactive_border_style()),
    );
    frame.render_widget(legend, chunks[3]);
}

fn render_preset_list(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: &[crate::app::filter_dialog::FilterPresetItem],
    selected_index: usize,
) {
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == selected_index;
            let prefix = if is_selected { "▸ " } else { "  " };

            let label_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let desc_style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, label_style),
                Span::styled(format!("{:<30}", item.label), label_style),
                Span::styled(format!(" — {}", item.description), desc_style),
            ]))
        })
        .collect();

    let list_widget = List::new(list_items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(inactive_border_style()),
    );
    frame.render_widget(list_widget, area);
}

fn render_active_list(
    frame: &mut Frame,
    area: Rect,
    state: &ApplicationState,
    selected_index: usize,
) {
    let video_filters = state.builder.filters.video_filters();
    let audio_filters = state.builder.filters.audio_filters();

    let mut list_items = Vec::new();
    let mut total_idx = 0;

    for vf in video_filters {
        let is_selected = total_idx == selected_index;
        let prefix = if is_selected { "▸ " } else { "  " };
        let label_style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        list_items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, label_style),
            Span::styled("[VIDEO] ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<20}", vf.name()), label_style),
            Span::styled(format!(" -> FFmpeg: {}", vf.to_ffmpeg_string()), Style::default().fg(Color::Yellow)),
        ])));
        total_idx += 1;
    }

    for af in audio_filters {
        let is_selected = total_idx == selected_index;
        let prefix = if is_selected { "▸ " } else { "  " };
        let label_style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        list_items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, label_style),
            Span::styled("[AUDIO] ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<20}", af.name()), label_style),
            Span::styled(format!(" -> FFmpeg: {}", af.to_ffmpeg_string()), Style::default().fg(Color::Yellow)),
        ])));
        total_idx += 1;
    }

    if list_items.is_empty() {
        list_items.push(ListItem::new(Line::from(vec![Span::styled(
            "  <No active filters applied — select Video or Audio tab to add filters>",
            Style::default().fg(Color::DarkGray),
        )])));
    }

    let title_str = format!(
        " Active Filters ({} video, {} audio) ",
        video_filters.len(),
        audio_filters.len()
    );

    let list_widget = List::new(list_items).block(
        Block::default()
            .title(title_str)
            .borders(Borders::ALL)
            .border_style(inactive_border_style()),
    );
    frame.render_widget(list_widget, area);
}
