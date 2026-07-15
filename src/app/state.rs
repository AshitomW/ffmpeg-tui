use crate::app::file_browser::FileBrowserState;
use crate::app::filter_dialog::FilterDialogState;
use crate::domain::{Job, JobConfig, JobId, MediaInfo};
use crate::infra::{FFMpegExecutor, FFProbeExecutor};
use crate::preset::Preset;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, mpsc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Dashboard,
    Builder,
    Queue,
    Logs,
    Inspector,
    Help,
}

impl Screen {
    #[must_use]
    pub const fn title(&self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Builder => "Builder",
            Self::Queue => "Job Queue",
            Self::Logs => "Logs",
            Self::Inspector => "File Inspector",
            Self::Help => "Help",
        }
    }

    #[must_use]
    pub const fn key(&self) -> char {
        match self {
            Self::Dashboard => '1',
            Self::Builder => '2',
            Self::Queue => '3',
            Self::Logs => '4',
            Self::Inspector => '5',
            Self::Help => '?',
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuilderState {
    pub input_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub selected_preset: Option<String>,
    pub input_info: Option<MediaInfo>,
    pub video_codec: crate::domain::VideoCodec,
    pub audio_codec: crate::domain::AudioCodec,
    pub format: crate::domain::ContainerFormat,
    pub crf: u8,
    pub preset: crate::domain::EncodingPreset,
    pub filters: crate::domain::FilterChain,
    pub preset_index: usize,
    pub current_field: BuilderField,
    pub raw_command_mode: bool,
    pub raw_command: String,
}

impl Default for BuilderState {
    fn default() -> Self {
        Self {
            input_path: None,
            output_path: None,
            selected_preset: None,
            input_info: None,
            video_codec: crate::domain::VideoCodec::default(),
            audio_codec: crate::domain::AudioCodec::default(),
            format: crate::domain::ContainerFormat::default(),
            crf: 23,
            preset: crate::domain::EncodingPreset::default(),
            filters: crate::domain::FilterChain::default(),
            preset_index: 0,
            current_field: BuilderField::default(),
            raw_command_mode: false,
            raw_command: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuilderField {
    #[default]
    Input,
    Output,
    VideoCodec,
    AudioCodec,
    Format,
    Quality,
    Preset,
    Filters,
}

impl BuilderField {
    pub const fn all() -> &'static [Self] {
        &[
            Self::Input,
            Self::Output,
            Self::VideoCodec,
            Self::AudioCodec,
            Self::Format,
            Self::Quality,
            Self::Preset,
            Self::Filters,
        ]
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::Input => "Input File",
            Self::Output => "Output File",
            Self::VideoCodec => "Video Codec",
            Self::AudioCodec => "Audio Codec",
            Self::Format => "Container Format",
            Self::Quality => "Quality (CRF)",
            Self::Preset => "Encoding Preset",
            Self::Filters => "Filters",
        }
    }

    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::Input => Self::Output,
            Self::Output => Self::VideoCodec,
            Self::VideoCodec => Self::AudioCodec,
            Self::AudioCodec => Self::Format,
            Self::Format => Self::Quality,
            Self::Quality => Self::Preset,
            Self::Preset => Self::Filters,
            Self::Filters => Self::Input,
        }
    }

    #[must_use]
    pub const fn prev(&self) -> Self {
        match self {
            Self::Input => Self::Filters,
            Self::Output => Self::Input,
            Self::VideoCodec => Self::Output,
            Self::AudioCodec => Self::VideoCodec,
            Self::Format => Self::AudioCodec,
            Self::Quality => Self::Format,
            Self::Preset => Self::Quality,
            Self::Filters => Self::Preset,
        }
    }
}

#[derive(Debug, Default)]
pub struct QueueState {
    pub selected_index: usize,
    pub show_completed: bool,
    pub is_paused: bool,
}

#[derive(Debug, Default)]
pub struct LogsState {
    pub selected_job: Option<JobId>,
    pub show_raw_output: bool,
    pub scroll_position: usize,
    pub auto_scroll: bool,
}

#[derive(Debug, Default)]
pub struct InspectorState {
    pub current_path: Option<PathBuf>,
    pub media_info: Option<MediaInfo>,
    pub loading: bool,
    pub error: Option<String>,
}

pub struct ApplicationState {
    pub screen: Screen,
    pub running: bool,
    pub status_message: Option<String>,

    pub builder: BuilderState,
    pub queue_state: QueueState,
    pub logs_state: LogsState,
    pub inspector: InspectorState,
    pub file_browser: Option<FileBrowserState>,
    pub filter_dialog: Option<FilterDialogState>,

    jobs: Arc<RwLock<HashMap<JobId, Job>>>,
    job_queue: Arc<Mutex<VecDeque<JobId>>>,
    active_jobs: Arc<RwLock<Vec<JobId>>>,
    running_tasks: Arc<Mutex<HashMap<JobId, tokio::task::JoinHandle<()>>>>,

    pub presets: Vec<Preset>,

    ffmpeg: Arc<FFMpegExecutor>,
    ffprobe: Arc<FFProbeExecutor>,

    pub action_tx: mpsc::UnboundedSender<crate::app::Action>,
    action_rx: Option<mpsc::UnboundedReceiver<crate::app::Action>>,

    pub max_concurrent_jobs: usize,
}

impl ApplicationState {
    pub fn new(ffmpeg: FFMpegExecutor, ffprobe: FFProbeExecutor) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        Self {
            screen: Screen::Dashboard,
            running: true,
            status_message: None,
            builder: BuilderState::default(),
            queue_state: QueueState::default(),
            logs_state: LogsState {
                auto_scroll: true,
                ..Default::default()
            },
            inspector: InspectorState::default(),
            file_browser: None,
            filter_dialog: None,
            jobs: Arc::new(RwLock::new(HashMap::new())),
            job_queue: Arc::new(Mutex::new(VecDeque::new())),
            active_jobs: Arc::new(RwLock::new(Vec::new())),
            running_tasks: Arc::new(Mutex::new(HashMap::new())),
            presets: crate::preset::builtin_presets(),
            ffmpeg: Arc::new(ffmpeg),
            ffprobe: Arc::new(ffprobe),
            action_tx,
            action_rx: Some(action_rx),
            max_concurrent_jobs: 2,
        }
    }

    pub fn take_action_rx(&mut self) -> Option<mpsc::UnboundedReceiver<crate::app::Action>> {
        self.action_rx.take()
    }

    pub fn ffmpeg(&self) -> &Arc<FFMpegExecutor> {
        &self.ffmpeg
    }

    pub fn ffprobe(&self) -> &Arc<FFProbeExecutor> {
        &self.ffprobe
    }

    pub fn jobs(&self) -> &Arc<RwLock<HashMap<JobId, Job>>> {
        &self.jobs
    }

    pub fn job_queue(&self) -> &Arc<Mutex<VecDeque<JobId>>> {
        &self.job_queue
    }

    pub fn active_jobs(&self) -> &Arc<RwLock<Vec<JobId>>> {
        &self.active_jobs
    }

    pub async fn register_task(&self, id: JobId, handle: tokio::task::JoinHandle<()>) {
        let mut tasks = self.running_tasks.lock().await;
        tasks.insert(id, handle);
    }

    pub async fn cancel_task(&self, id: JobId) {
        let mut tasks = self.running_tasks.lock().await;
        if let Some(handle) = tasks.remove(&id) {
            handle.abort();
        }
    }

    pub async fn remove_task(&self, id: JobId) {
        let mut tasks = self.running_tasks.lock().await;
        tasks.remove(&id);
    }

    pub async fn add_job(&self, config: JobConfig) -> JobId {
        let mut job = Job::new(config);
        let id = job.id();
        job.queue();

        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(id, job);
        }

        {
            let mut queue = self.job_queue.lock().await;
            queue.push_back(id);
        }

        id
    }

    pub async fn get_jobs(&self) -> Vec<Job> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }

    pub async fn get_job(&self, id: JobId) -> Option<Job> {
        let jobs = self.jobs.read().await;
        jobs.get(&id).cloned()
    }

    pub async fn get_sorted_job_ids(&self) -> Vec<JobId> {
        let jobs = self.jobs.read().await;
        let mut entries: Vec<_> = jobs.values().collect();
        entries.sort_by_key(|j| std::cmp::Reverse(j.created_at()));
        entries.iter().map(|j| j.id()).collect()
    }

    pub async fn move_job_up(&self, id: JobId) {
        let mut queue = self.job_queue.lock().await;
        if let Some(idx) = queue.iter().position(|&j| j == id).filter(|&i| i > 0) {
            queue.swap(idx, idx - 1);
        }
    }

    pub async fn move_job_down(&self, id: JobId) {
        let mut queue = self.job_queue.lock().await;
        if let Some(idx) = queue.iter().position(|&j| j == id).filter(|&i| i + 1 < queue.len()) {
            queue.swap(idx, idx + 1);
        }
    }

    pub async fn pending_job_count(&self) -> usize {
        let queue = self.job_queue.lock().await;
        queue.len()
    }

    pub async fn active_job_count(&self) -> usize {
        let active = self.active_jobs.read().await;
        active.len()
    }

    pub async fn completed_job_count(&self) -> usize {
        let jobs = self.jobs.read().await;
        jobs.values().filter(|j| j.status().is_terminal()).count()
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn switch_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}

