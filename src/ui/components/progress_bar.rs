use crate::domain::JobProgress;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Gauge;

pub fn render_progress_bar(
    frame: &mut Frame,
    area: Rect,
    progress: &JobProgress,
    source_duration: Option<f64>,
) {
    let label = if let Some(duration) = source_duration {
        let eta = progress.eta_string(duration);
        format!(
            "{:.1}% | FPS: {:.1} | Speed: {:.2}x | ETA: {}",
            progress.percentage, progress.fps, progress.speed, eta
        )
    } else {
        format!(
            "{:.1}% | FPS: {:.1} | Speed: {:.2}x",
            progress.percentage, progress.fps, progress.speed
        )
    };

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .percent(progress.percentage.clamp(0.0, 100.0) as u16)
        .label(label);

    frame.render_widget(gauge, area);
}
