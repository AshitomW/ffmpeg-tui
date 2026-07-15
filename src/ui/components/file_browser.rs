use crate::app::file_browser::{FileBrowserState, FileBrowserTarget};
use crate::ui::theme::{active_border_style, inactive_border_style};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

/// Render a centered popup area within `area`, occupying `percent_x`% width
/// and `percent_y`% height.
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

/// Render the file browser as a modal popup overlay.
pub fn render_file_browser(frame: &mut Frame, area: Rect, browser: &FileBrowserState) {
    let popup = centered_popup(area, 75, 80);

    // Clear the background behind the popup
    frame.render_widget(Clear, popup);

    let is_output = browser.target == FileBrowserTarget::Output;

    let target_label = match browser.target {
        FileBrowserTarget::Input => " SELECT INPUT FILE ",
        FileBrowserTarget::Output => " SELECT OUTPUT FILE & LOCATION ",
    };

    let bottom_height = if is_output { 5 } else { 3 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),         // path bar
            Constraint::Min(5),           // file list
            Constraint::Length(bottom_height), // filter / filename bar
        ])
        .split(popup);

    // ── Path bar ──────────────────────────────────────────────────
    let path_display = browser.current_dir.display().to_string();
    let path_bar = Paragraph::new(Line::from(vec![
        Span::styled(
            target_label,
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Directory: ", Style::default().fg(Color::Gray)),
        Span::styled(
            path_display,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(active_border_style())
            .title(" File Browser "),
    );
    frame.render_widget(path_bar, chunks[0]);

    // ── File list ─────────────────────────────────────────────────
    let visible: Vec<ListItem> = browser
        .visible_entries()
        .map(|(pos, entry)| {
            let is_selected = pos == browser.selected_index;
            let prefix = if is_selected { "▸ " } else { "  " };
            let icon = if entry.is_dir { "📁 " } else { "📄 " };

            let name_style = if is_selected {
                if entry.is_dir {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                }
            } else if entry.is_dir {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };

            let size_str = if entry.is_dir {
                "<DIR>".to_string()
            } else {
                FileBrowserState::format_size(entry.size)
            };

            let size_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, name_style),
                Span::raw(icon),
                Span::styled(format!("{:<40}", entry.name), name_style),
                Span::styled(format!("{:>10}", size_str), size_style),
            ]))
        })
        .collect();

    let count_label = if is_output {
        format!(" {} item(s) (↑↓ to select subfolder) ", browser.visible_count())
    } else {
        format!(" {} item(s) ", browser.visible_count())
    };

    let list = List::new(visible).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(inactive_border_style())
            .title(count_label),
    );
    frame.render_widget(list, chunks[1]);

    // ── Bottom section (Input vs Output) ──────────────────────────
    if is_output {
        let full_destination = browser
            .output_full_path()
            .map_or_else(|| "<enter filename>".to_string(), |p| p.display().to_string());

        let filename_style = if browser.output_filename.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        };

        let bottom_content = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(" Output Filename: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(
                    if browser.output_filename.is_empty() { "output" } else { &browser.output_filename },
                    filename_style,
                ),
                Span::styled("█", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!(".{}", browser.output_extension),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Save Destination: ", Style::default().fg(Color::Gray)),
                Span::styled(full_destination, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(
                    " [Enter] Confirm Save / Enter Dir  [Tab] Parent Dir  [Esc] Cancel  [Type] Change Name ",
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(active_border_style())
                .title(" Output File Settings "),
        );
        frame.render_widget(bottom_content, chunks[2]);
    } else {
        let filter_display = if browser.filter_text.is_empty() {
            "Type to filter...".to_string()
        } else {
            browser.filter_text.clone()
        };

        let filter_style = if browser.filter_text.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        };

        let hints = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(" Filter: ", Style::default().fg(Color::Cyan)),
                Span::styled(format!("{filter_display}█"), filter_style),
            ]),
            Line::from(vec![
                Span::styled(
                    " [Enter] Open/Select  [Tab] Parent Dir  [Esc] Cancel  [↑↓] Navigate ",
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(inactive_border_style()),
        );
        frame.render_widget(hints, chunks[2]);
    }
}
