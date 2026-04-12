use crate::domain::{FFMpegCommand, JobProgress};
use crate::infra::ProcessError;
use crate::parser::ProgressParser;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct FFMpegExecutor {
    binary_path: String,
}

impl FFMpegExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            binary_path: "ffmpeg".to_string(),
        }
    }

    #[must_use]
    pub fn with_binary(binary_path: impl Into<String>) -> Self {
        Self {
            binary_path: binary_path.into(),
        }
    }

    pub async fn execute<F>(
        &self,
        command: FFMpegCommand,
        mut on_progress: F,
    ) -> Result<u64, ProcessError>
    where
        F: FnMut(JobProgress) + Send + 'static,
    {
        let args = command.to_args();
        debug!("Executing FFMpeg with args: {:?}", args);

        let mut child = Command::new(&self.binary_path)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| ProcessError::FFMpegError("Failed to capture stderr".into()))?;

        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        let mut parser = ProgressParser::new();
        let mut last_error = String::new();

        while let Ok(Some(line)) = lines.next_line().await {
            debug!("FFMpeg: {}", line);
            if let Some(progress) = parser.parse_line(&line) {
                on_progress(progress);
            }

            if line.contains("Error") || line.contains("error") {
                last_error = line;
            }
        }

        let status = child.wait().await?;

        if status.success() {
            let output_size = command
                .output_path()
                .metadata()
                .map(|m| m.len())
                .unwrap_or(0);

            info!(
                "FFMpeg completed successfully, output size: {}",
                output_size
            );
            Ok(output_size)
        } else {
            let code = status.code().unwrap_or(-1);
            error!("FFMpeg failed with code {}: {}", code, last_error);
            Err(ProcessError::NonZeroExit {
                code,
                stderr: last_error,
            })
        }
    }

    pub async fn check_available(&self) -> Result<String, ProcessError> {
        let output = Command::new(&self.binary_path)
            .arg("-version")
            .output()
            .await?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            let first_line = version.lines().next().unwrap_or("unknown");
            Ok(first_line.to_string())
        } else {
            Err(ProcessError::FFMpegError("FFMpeg not available".into()))
        }
    }
}

impl Default for FFMpegExecutor {
    fn default() -> Self {
        Self::new()
    }
}
