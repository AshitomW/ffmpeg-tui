use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Failed to spawn process: {0}")]
    SpawnError(#[from] std::io::Error),

    #[error("Process exited with non-zero status: {code}")]
    NonZeroExit { code: i32, stderr: String },

    #[error("Process was killed by signal")]
    Killed,

    #[error("Failed to parse output: {0}")]
    ParseError(String),

    #[error("Process timed out")]
    Timeout,

    #[error("FFmpeg error: {0}")]
    FFMpegError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
