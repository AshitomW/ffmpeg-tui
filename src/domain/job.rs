use crate::domain::FFMpegCommand;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(Uuid);

impl JobId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    #[must_use]
    pub const fn inner(&self) -> Uuid {
        self.0
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.to_string()[..8])
    }
}

/// For Job execution priority

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub enum JobPriority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

impl std::fmt::Display for JobPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Low => "Low",
            Self::Normal => "Normal",
            Self::High => "High",
            Self::Critical => "Critical",
        };
        write!(f, "{name}")
    }
}

/// Job Progress
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JobProgress {
    pub frame: u64,
    pub fps: f64,
    pub bitrate: Option<f64>,
    pub total_size: u64,
    pub time_encoded: f64,
    pub speed: f64,
    pub percentage: f64,
}

impl JobProgress {
    /// Estimate time
    #[must_use]
    pub fn eta_in_seconds(&self, total_duration: f64) -> Option<f64> {
        if self.speed > 0.0 && self.percentage < 100.0 {
            let remaining = total_duration - self.time_encoded;
            Some(remaining / self.speed)
        } else {
            None
        }
    }

    /// Human readable formatting
    #[must_use]
    pub fn eta_string(&self, total_duration: f64) -> String {
        match self.eta_in_seconds(total_duration) {
            Some(seconds) if seconds.is_finite() && seconds >= 0.0 => {
                format_duration(seconds as u64)
            }
            _ => "N/A".to_string(),
        }
    }
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{hours}h {minutes:02}m {secs:02}s")
    } else if minutes > 0 {
        format!("{minutes}m {secs:02}s")
    } else {
        format!("{secs}s")
    }
}

/// Status of the job
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Queued,
    Running {
        started_at: DateTime<Utc>,
        progress: JobProgress,
    },
    Paused {
        progress: JobProgress,
    },
    Completed {
        started_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
        output_size: u64,
    },
    Failed {
        error: String,
        retries: u32,
    },
    Cancelled,
}

impl JobStatus {
    /// true if job is in terminal state
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed { .. } | Self::Failed { .. } | Self::Cancelled
        )
    }

    /// true if job is retryable
    #[must_use]
    pub const fn can_retry(&self) -> bool {
        matches!(self, Self::Failed { .. } | Self::Cancelled)
    }

    /// rtrue if the job is running
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    /// status name
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Queued => "Queued",
            Self::Running { .. } => "Running",
            Self::Paused { .. } => "Paused",
            Self::Completed { .. } => "Completed",
            Self::Failed { .. } => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }
}

impl Default for JobStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Configuration for the job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub command: FFMpegCommand,
    pub priority: JobPriority,
    pub max_retries: u32,
}

impl JobConfig {
    #[must_use]
    pub fn new(input_path: PathBuf, output_path: PathBuf, command: FFMpegCommand) -> Self {
        Self {
            input_path,
            output_path,
            command,
            priority: JobPriority::Normal,
            max_retries: 3,
        }
    }

    #[must_use]
    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    #[must_use]
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub output_path: PathBuf,
    pub output_size: u64,
    pub duration_seconds: f64,
    pub average_fps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    id: JobId,
    config: JobConfig,
    status: JobStatus,
    created_at: DateTime<Utc>,
    log: Vec<String>,
    source_duration: Option<f64>,
}

impl Job {
    #[must_use]
    pub fn new(config: JobConfig) -> Self {
        Self {
            id: JobId::new(),
            config,
            status: JobStatus::Pending,
            created_at: Utc::now(),
            log: Vec::new(),
            source_duration: None,
        }
    }

    #[must_use]
    pub const fn id(&self) -> JobId {
        self.id
    }

    #[must_use]
    pub const fn config(&self) -> &JobConfig {
        &self.config
    }

    #[must_use]
    pub const fn status(&self) -> &JobStatus {
        &self.status
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub fn log(&self) -> &[String] {
        &self.log
    }

    #[must_use]
    pub const fn source_duration(&self) -> Option<f64> {
        self.source_duration
    }

    pub fn set_source_duration(&mut self, duration: f64) {
        self.source_duration = Some(duration)
    }

    pub fn add_log(&mut self, line: String) {
        self.log.push(line);
    }

    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    pub fn queue(&mut self) {
        if matches!(self.status, JobStatus::Pending) {
            self.status = JobStatus::Queued
        }
    }

    pub fn start(&mut self) {
        if matches!(self.status, JobStatus::Queued | JobStatus::Pending) {
            self.status = JobStatus::Running {
                started_at: Utc::now(),
                progress: JobProgress::default(),
            }
        }
    }

    pub fn update_progress(&mut self, progress: JobProgress) {
        if let JobStatus::Running { started_at, .. } = self.status {
            self.status = JobStatus::Running {
                started_at,
                progress,
            }
        }
    }

    pub fn complete(&mut self, output_size: u64) {
        if let JobStatus::Running { started_at, .. } = self.status {
            self.status = JobStatus::Completed {
                started_at,
                completed_at: Utc::now(),
                output_size,
            }
        }
    }

    pub fn fail(&mut self, error: String) {
        let retries = match &self.status {
            JobStatus::Failed { retries, .. } => *retries + 1,
            _ => 0,
        };
        self.status = JobStatus::Failed { error, retries }
    }

    pub fn cancel(&mut self) {
        if !self.status.is_terminal() {
            self.status = JobStatus::Cancelled;
        }
    }

    pub fn requeue(&mut self) -> bool {
        if self.status.can_retry() {
            if let JobStatus::Failed { retries, .. } = &self.status {
                if *retries < self.config.max_retries {
                    self.status = JobStatus::Queued;
                    return true;
                }
            } else {
                self.status = JobStatus::Queued;
                return true;
            }
        }
        false
    }

    #[must_use]
    pub const fn can_cancel(&self) -> bool {
        !self.status.is_terminal()
    }

    #[must_use]
    pub fn input_name(&self) -> &str {
        self.config
            .input_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    #[must_use]
    pub fn output_name(&self) -> &str {
        self.config
            .output_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    #[must_use]
    pub fn progress(&self) -> Option<&JobProgress> {
        match &self.status {
            JobStatus::Running { progress, .. } | JobStatus::Paused { progress } => Some(progress),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::CommandBuilder;

    fn create_test_job() -> Job {
        let command = CommandBuilder::new()
            .input(PathBuf::from("input.mp4"))
            .output(PathBuf::from("output.mp4"))
            .build()
            .unwrap();

        let config = JobConfig::new(
            PathBuf::from("input.mp4"),
            PathBuf::from("output.mp4"),
            command,
        );

        Job::new(config)
    }

    #[test]
    fn job_state_transition() {
        let mut job = create_test_job();
        assert!(matches!(job.status(), JobStatus::Pending));

        job.queue();
        assert!(matches!(job.status(), JobStatus::Queued));

        job.start();
        assert!(job.status().is_running());

        job.complete(1000);
        assert!(job.status().is_terminal());
    }

    #[test]
    fn job_failure_and_retry() {
        let mut job = create_test_job();
        job.queue();
        job.start();
        job.fail("Test error".to_string());

        assert!(matches!(job.status(), JobStatus::Failed { .. }));
        assert!(job.requeue());
        assert!(matches!(job.status(), JobStatus::Queued));
    }

    #[test]
    fn job_progress_eta() {
        let progress = JobProgress {
            percentage: 50.0,
            speed: 2.0,
            time_encoded: 30.0,
            ..Default::default()
        };

        let eta = progress.eta_in_seconds(60.0);
        assert!(eta.is_some());
        assert!((eta.unwrap() - 15.0).abs() < 0.01)
    }
}
