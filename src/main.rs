use ffmpeg_tui::app::ApplicationState;
use ffmpeg_tui::infra::{FFMpegExecutor, FFProbeExecutor};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(log_path) = std::env::var("FFMPEG_TUI_LOG") {
        let file = std::fs::File::create(log_path)?;
        tracing_subscriber::fmt()
            .with_writer(file)
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }

    let ffmpeg = FFMpegExecutor::new();
    let ffprobe = FFProbeExecutor::new();

    let mut state = ApplicationState::new(ffmpeg, ffprobe);

    match state.ffmpeg().check_available().await {
        Ok(ver) => state.set_status(format!("FFmpeg ready: {ver}")),
        Err(_) => state.set_status("Warning: FFmpeg binary not found in PATH!"),
    }

    ffmpeg_tui::ui::run_app(state).await?;

    Ok(())
}
