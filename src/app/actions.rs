use std::path::PathBuf;

use crate::app::file_browser::FileBrowserTarget;
use crate::domain::{
    AudioCodec, ContainerFormat, EncodingPreset, Filter, JobId, JobProgress, VideoCodec,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationTarget {
    Dashboard,
    Builder,
    Queue,
    Logs,
    Inspector,
    Help,
}

#[derive(Debug, Clone)]
pub enum Action {
    Navigate(NavigationTarget),
    NavigateBack,

    Quit,
    Tick,

    SetInput(PathBuf),
    SetOutput(PathBuf),
    SetVideoCodec(VideoCodec),
    SetAudioCodec(AudioCodec),
    SetFormat(ContainerFormat),
    SetCrf(u8),
    SetPreset(EncodingPreset),
    AddFilter(Filter),
    RemoveFilter(usize),
    LoadPreset(String),
    BuildJob,
    ToggleRawCommandMode,
    SetRawCommand(String),
    RawCommandAppend(char),
    RawCommandBackspace,

    // File browser actions
    OpenFileBrowser(FileBrowserTarget),
    FileBrowserNavigateUp,
    FileBrowserConfirm,
    FileBrowserCancel,
    FileBrowserSelectPrev,
    FileBrowserSelectNext,
    FileBrowserFilterAppend(char),
    FileBrowserFilterBackspace,
    FileBrowserFilenameAppend(char),
    FileBrowserFilenameBackspace,

    // Filter dialog actions
    OpenFilterDialog,
    CloseFilterDialog,
    FilterDialogNextTab,
    FilterDialogPrevTab,
    FilterDialogSelectNext,
    FilterDialogSelectPrev,
    FilterDialogConfirm,
    FilterDialogRemoveSelected,
    FilterDialogClearAll,
    FilterDialogCustomAppend(char),
    FilterDialogCustomBackspace,

    StartQueue,
    PauseQueue,
    ClearCompleted,
    CancelJob(JobId),
    RetryJob(JobId),
    SelectJob(JobId),
    MoveJobUp(JobId),
    MoveJobDown(JobId),

    ToggleRawLogs,
    ToggleAutoScroll,
    ScrollUp,
    ScrollDown,
    ScrollToTop,
    ScrollToBottom,

    InspectFile(PathBuf),

    JobStarted(JobId),
    JobProgress(JobId, JobProgress),
    JobCompleted(JobId, u64),
    JobFailed(JobId, String),
    JobLogLine(JobId, String),

    NextField,
    PrevField,
    ConfirmSelection,
    CancelSelection,

    ShowStatus(String),
    ClearStatus,
}
