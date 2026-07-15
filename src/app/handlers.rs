use crate::app::file_browser::{FileBrowserState, FileBrowserTarget};
use crate::app::{Action, ApplicationState, NavigationTarget, Screen};
use crate::domain::{CommandBuilder, Filter, JobConfig};
use std::path::PathBuf;
use tracing::{error, info, warn};

pub struct ActionHandler;

impl ActionHandler {
    pub async fn handle(state: &mut ApplicationState, action: Action) {
        match action {
            Action::Navigate(target) => {
                state.screen = match target {
                    NavigationTarget::Dashboard => Screen::Dashboard,
                    NavigationTarget::Builder => Screen::Builder,
                    NavigationTarget::Queue => Screen::Queue,
                    NavigationTarget::Logs => Screen::Logs,
                    NavigationTarget::Inspector => Screen::Inspector,
                    NavigationTarget::Help => Screen::Help,
                };
            }
            Action::NavigateBack => {
                state.screen = Screen::Dashboard;
            }
            Action::Quit => {
                state.quit();
            }
            Action::SetInput(path) => {
                Self::handle_set_input(state, path).await;
            }
            Action::SetOutput(path) => {
                state.builder.output_path = Some(path);
            }
            Action::SetVideoCodec(codec) => {
                state.builder.video_codec = codec;
                info!("Set video codec: {:?}", codec);
            }
            Action::SetAudioCodec(codec) => {
                state.builder.audio_codec = codec;
                info!("Set audio codec: {:?}", codec);
            }
            Action::SetFormat(format) => {
                state.builder.format = format;
                // Auto-update the output path extension when format changes
                if let Some(out_path) = &state.builder.output_path {
                    let mut updated = out_path.clone();
                    updated.set_extension(format.extension());
                    state.builder.output_path = Some(updated);
                }
                info!("Set format: {:?}", format);
            }
            Action::SetCrf(crf) => {
                state.builder.crf = crf;
                info!("Set CRF: {}", crf);
            }
            Action::SetPreset(preset) => {
                state.builder.preset = preset;
                info!("Set encoding preset: {:?}", preset);
            }
            Action::AddFilter(filter) => match filter {
                Filter::Video(vf) => {
                    state.builder.filters.add_video_filter(vf);
                    info!("Added video filter");
                }
                Filter::Audio(af) => {
                    state.builder.filters.add_audio_filter(af);
                    info!("Added audio filter");
                }
            },
            Action::RemoveFilter(index) => {
                if state.builder.filters.remove_video_filter(index).is_none() {
                    state.builder.filters.remove_audio_filter(index);
                }
                info!("Removed filter at index: {}", index);
            }
            Action::LoadPreset(name) => {
                if let Some(preset) = state.presets.iter().find(|p| p.name == name).cloned() {
                    state.builder.selected_preset = Some(preset.name.clone());
                    state.builder.video_codec = preset.video_codec;
                    state.builder.audio_codec = preset.audio_codec;
                    state.builder.format = preset.container;
                    if let Some(crf) = preset.crf {
                        state.builder.crf = crf;
                    }
                    if let Some(p) = preset.encoding_preset {
                        state.builder.preset = p;
                    }
                    state.builder.filters = preset.filters;
                    state.builder.preset_index = (state.builder.preset_index + 1) % state.presets.len();
                    state.set_status(format!("Loaded preset: {}", preset.name));
                    info!("Loaded preset: {}", name);
                }
            }

            Action::BuildJob => {
                Self::handle_build_job(state).await;
            }
            Action::ToggleRawCommandMode => {
                state.builder.raw_command_mode = !state.builder.raw_command_mode;
            }
            Action::SetRawCommand(cmd) => state.builder.raw_command = cmd,
            Action::RawCommandAppend(c) => {
                state.builder.raw_command.push(c);
            }
            Action::RawCommandBackspace => {
                state.builder.raw_command.pop();
            }

            // ── File browser actions ──────────────────────────────────
            Action::OpenFileBrowser(target) => {
                Self::handle_open_file_browser(state, target);
            }
            Action::FileBrowserNavigateUp => {
                if let Some(browser) = &mut state.file_browser
                    && let Err(e) = browser.navigate_up()
                {
                    warn!("Failed to navigate up: {}", e);
                    state.set_status(format!("Navigation error: {e}"));
                }
            }
            Action::FileBrowserConfirm => {
                Self::handle_file_browser_confirm(state).await;
            }
            Action::FileBrowserCancel => {
                state.file_browser = None;
            }
            Action::FileBrowserSelectPrev => {
                if let Some(browser) = &mut state.file_browser {
                    browser.select_prev();
                }
            }
            Action::FileBrowserSelectNext => {
                if let Some(browser) = &mut state.file_browser {
                    browser.select_next();
                }
            }
            Action::FileBrowserFilterAppend(c) => {
                if let Some(browser) = &mut state.file_browser {
                    browser.filter_push(c);
                }
            }
            Action::FileBrowserFilterBackspace => {
                if let Some(browser) = &mut state.file_browser {
                    if browser.filter_text.is_empty() {
                        // When filter is empty, backspace navigates up
                        if let Err(e) = browser.navigate_up() {
                            warn!("Failed to navigate up: {}", e);
                        }
                    } else {
                        browser.filter_pop();
                    }
                }
            }
            Action::FileBrowserFilenameAppend(c) => {
                if let Some(browser) = &mut state.file_browser {
                    browser.filename_push(c);
                }
            }
            Action::FileBrowserFilenameBackspace => {
                if let Some(browser) = &mut state.file_browser {
                    browser.filename_pop();
                }
            }

            // ── Filter dialog actions ──────────────────────────────────
            Action::OpenFilterDialog => {
                state.filter_dialog = Some(crate::app::FilterDialogState::new());
                info!("Opened Filter Dialog");
            }
            Action::CloseFilterDialog => {
                state.filter_dialog = None;
                info!("Closed Filter Dialog");
            }
            Action::FilterDialogNextTab => {
                if let Some(dlg) = &mut state.filter_dialog {
                    dlg.next_tab();
                }
            }
            Action::FilterDialogPrevTab => {
                if let Some(dlg) = &mut state.filter_dialog {
                    dlg.prev_tab();
                }
            }
            Action::FilterDialogSelectNext => {
                if let Some(dlg) = &mut state.filter_dialog {
                    let count = dlg.active_count(&state.builder.filters);
                    dlg.select_next(count);
                }
            }
            Action::FilterDialogSelectPrev => {
                if let Some(dlg) = &mut state.filter_dialog {
                    let count = dlg.active_count(&state.builder.filters);
                    dlg.select_prev(count);
                }
            }
            Action::FilterDialogConfirm => {
                Self::handle_filter_dialog_confirm(state);
            }
            Action::FilterDialogRemoveSelected => {
                Self::handle_filter_dialog_remove(state);
            }
            Action::FilterDialogClearAll => {
                state.builder.filters.clear();
                state.set_status("Cleared all filters");
            }
            Action::FilterDialogCustomAppend(c) => {
                if let Some(dlg) = &mut state.filter_dialog {
                    dlg.custom_append(c);
                }
            }
            Action::FilterDialogCustomBackspace => {
                if let Some(dlg) = &mut state.filter_dialog {
                    dlg.custom_backspace();
                }
            }

            Action::StartQueue => {
                state.queue_state.is_paused = false;
                state.set_status("Queue started");
                info!("Starting job queue processing");
                Self::process_queue(state).await;
            }

            Action::PauseQueue => {
                state.queue_state.is_paused = true;
                state.set_status("Queue paused");
                info!("Pausing job queue");
            }
            Action::ClearCompleted => {
                Self::clear_completed_jobs(state).await;
            }
            Action::CancelJob(id) => {
                Self::cancel_job(state, id).await;
            }
            Action::RetryJob(id) => {
                Self::retry_job(state, id).await;
            }

            Action::SelectJob(id) => {
                state.logs_state.selected_job = Some(id);
            }

            Action::MoveJobUp(id) => {
                state.move_job_up(id).await;
            }
            Action::MoveJobDown(id) => {
                state.move_job_down(id).await;
            }

            Action::ToggleRawLogs => {
                state.logs_state.show_raw_output = !state.logs_state.show_raw_output;
            }

            Action::ToggleAutoScroll => {
                state.logs_state.auto_scroll = !state.logs_state.auto_scroll;
            }

            Action::ScrollUp => {
                state.logs_state.scroll_position =
                    state.logs_state.scroll_position.saturating_sub(1);
            }
            Action::ScrollDown => {
                state.logs_state.scroll_position += 1;
            }
            Action::ScrollToTop => {
                state.logs_state.scroll_position = 0;
            }
            Action::ScrollToBottom => state.logs_state.auto_scroll = true,

            Action::InspectFile(path) => {
                Self::handle_inspect_file(state, path).await;
            }

            Action::JobStarted(id) => {
                let mut jobs = state.jobs().write().await;
                if let Some(job) = jobs.get_mut(&id) {
                    job.start();
                }
            }
            Action::JobProgress(id, progress) => {
                let mut jobs = state.jobs().write().await;
                if let Some(job) = jobs.get_mut(&id) {
                    job.update_progress(progress);
                }
            }
            Action::JobCompleted(id, size) => {
                {
                    let mut jobs = state.jobs().write().await;
                    if let Some(job) = jobs.get_mut(&id) {
                        job.complete(size);
                    }
                }
                {
                    let mut active = state.active_jobs().write().await;
                    active.retain(|&j| j != id);
                }
                state.remove_task(id).await;
                state.set_status(format!("Job {id} completed successfully"));
                Self::process_queue(state).await;
            }
            Action::JobFailed(id, err) => {
                {
                    let mut jobs = state.jobs().write().await;
                    if let Some(job) = jobs.get_mut(&id) {
                        job.fail(err.clone());
                    }
                }
                {
                    let mut active = state.active_jobs().write().await;
                    active.retain(|&j| j != id);
                }
                state.remove_task(id).await;
                state.set_status(format!("Job {id} failed: {err}"));
                Self::process_queue(state).await;
            }
            Action::JobLogLine(id, line) => {
                let mut jobs = state.jobs().write().await;
                if let Some(job) = jobs.get_mut(&id) {
                    job.add_log(line);
                }
            }

            Action::NextField => {
                state.builder.current_field = state.builder.current_field.next();
            }
            Action::PrevField => {
                state.builder.current_field = state.builder.current_field.prev();
            }
            Action::ConfirmSelection | Action::CancelSelection | Action::Tick => {}
            Action::ShowStatus(msg) => {
                state.set_status(msg);
            }
            Action::ClearStatus => {
                state.clear_status();
            }
        }
    }

    fn handle_open_file_browser(state: &mut ApplicationState, target: FileBrowserTarget) {
        let start_dir = match target {
            FileBrowserTarget::Input => state
                .builder
                .input_path
                .as_ref()
                .and_then(|p| p.parent().map(|pp| pp.to_path_buf()))
                .unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
                }),
            FileBrowserTarget::Output => state
                .builder
                .output_path
                .as_ref()
                .and_then(|p| p.parent().map(|pp| pp.to_path_buf()))
                .or_else(|| {
                    state
                        .builder
                        .input_path
                        .as_ref()
                        .and_then(|p| p.parent().map(|pp| pp.to_path_buf()))
                })
                .unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
                }),
        };

        let extension = state.builder.format.extension();

        match FileBrowserState::open(&start_dir, target, extension) {
            Ok(mut browser) => {
                if target == FileBrowserTarget::Output {
                    let default_stem = if let Some(out_p) = &state.builder.output_path {
                        out_p
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("output")
                            .to_string()
                    } else if let Some(in_p) = &state.builder.input_path {
                        let stem = in_p
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("output");
                        format!("{stem}_output")
                    } else {
                        "output".to_string()
                    };
                    browser.output_filename = default_stem;
                }
                state.file_browser = Some(browser);
            }
            Err(e) => {
                error!("Failed to open file browser at {}: {}", start_dir.display(), e);
                state.set_status(format!("Cannot open directory: {e}"));
            }
        }
    }

    async fn handle_file_browser_confirm(state: &mut ApplicationState) {
        let target = state
            .file_browser
            .as_ref()
            .map(|b| b.target)
            .unwrap_or(FileBrowserTarget::Input);

        let result = {
            if let Some(browser) = &mut state.file_browser {
                match target {
                    FileBrowserTarget::Input => browser.confirm_selected(),
                    FileBrowserTarget::Output => browser.confirm_output(),
                }
            } else {
                return;
            }
        };

        match result {
            Ok(Some(path)) => {
                // A file/path was selected
                state.file_browser = None;

                match target {
                    FileBrowserTarget::Input => {
                        Self::handle_set_input(state, path).await;
                    }
                    FileBrowserTarget::Output => {
                        state.builder.output_path = Some(path.clone());
                        state.set_status(format!(
                            "Output set: {}",
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                        ));
                    }
                }
            }
            Ok(None) => {
                // Navigated into a subdirectory — browser stays open
            }
            Err(e) => {
                warn!("File browser navigation error: {}", e);
                state.set_status(format!("Error: {e}"));
            }
        }
    }

    async fn handle_set_input(state: &mut ApplicationState, path: PathBuf) {
        state.builder.input_path = Some(path.clone());
        if state.builder.output_path.is_none() {
            let file_stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let ext = state.builder.format.extension();
            let mut out_path = path.clone();
            out_path.set_file_name(format!("{file_stem}_output.{ext}"));
            state.builder.output_path = Some(out_path);
        }

        let ffprobe = state.ffprobe().clone();
        match ffprobe.inspect(&path).await {
            Ok(media_info) => {
                state.builder.input_info = Some(media_info.clone());
                state.set_status(format!(
                    "Loaded input: {}",
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                ));
            }
            Err(e) => {
                error!("Failed to inspect file {}: {}", path.display(), e);
                state.set_status(format!("Input file set (probe failed: {e})"));
            }
        }
    }

    async fn handle_build_job(state: &mut ApplicationState) {
        let input_path = match &state.builder.input_path {
            Some(p) => p.clone(),
            None => {
                state.set_status("Cannot build job: Input path missing");
                return;
            }
        };

        let output_path = match &state.builder.output_path {
            Some(p) => p.clone(),
            None => {
                state.set_status("Cannot build job: Output path missing");
                return;
            }
        };

        let command_res = if state.builder.raw_command_mode && !state.builder.raw_command.is_empty()
        {
            let mut builder = CommandBuilder::new()
                .input(input_path.clone())
                .output(output_path.clone());
            for arg in shellwords::split(&state.builder.raw_command).unwrap_or_default() {
                builder = builder.raw_arg(arg);
            }
            builder.build()
        } else {
            CommandBuilder::new()
                .input(input_path.clone())
                .output(output_path.clone())
                .video_codec(state.builder.video_codec)
                .audio_codec(state.builder.audio_codec)
                .format(state.builder.format)
                .crf(state.builder.crf)
                .preset(state.builder.preset)
                .filters(state.builder.filters.clone())
                .build()
        };

        match command_res {
            Ok(command) => {
                let mut config = JobConfig::new(input_path, output_path, command);
                if state.builder.input_info.as_ref().and_then(|i| i.duration_seconds()).is_some() {
                    config = config.with_priority(crate::domain::JobPriority::Normal);
                }

                let job_id = state.add_job(config).await;
                if let Some(dur) = state.builder.input_info.as_ref().and_then(|i| i.duration_seconds()) {
                    let mut jobs = state.jobs().write().await;
                    if let Some(job) = jobs.get_mut(&job_id) {
                        job.set_source_duration(dur);
                    }
                }

                state.set_status(format!("Job queued: {job_id}"));
                state.screen = Screen::Queue;
                Self::process_queue(state).await;
            }
            Err(e) => {
                error!("Failed to build command: {}", e);
                state.set_status(format!("Error building command: {e}"));
            }
        }
    }

    async fn handle_inspect_file(state: &mut ApplicationState, path: PathBuf) {
        state.inspector.loading = true;
        state.inspector.current_path = Some(path.clone());
        state.inspector.error = None;
        state.screen = Screen::Inspector;

        let ffprobe = state.ffprobe().clone();
        match ffprobe.inspect(&path).await {
            Ok(info) => {
                state.inspector.media_info = Some(info);
                state.inspector.loading = false;
                state.set_status("File inspected successfully");
            }
            Err(e) => {
                state.inspector.error = Some(e.to_string());
                state.inspector.loading = false;
                state.set_status("Inspection failed");
            }
        }
    }

    pub async fn process_queue(state: &mut ApplicationState) {
        if state.queue_state.is_paused {
            return;
        }

        while state.active_job_count().await < state.max_concurrent_jobs {
            let next_id = {
                let mut queue = state.job_queue().lock().await;
                queue.pop_front()
            };

            let job_id = match next_id {
                Some(id) => id,
                None => break,
            };

            let job = match state.get_job(job_id).await {
                Some(j) => j,
                None => continue,
            };

            if job.status().is_terminal() {
                continue;
            }

            {
                let mut active = state.active_jobs().write().await;
                active.push(job_id);
            }

            let ffmpeg = state.ffmpeg().clone();
            let action_tx = state.action_tx.clone();
            let command = job.config().command.clone();

            let task_id = job_id;
            let handle = tokio::spawn(async move {
                let _ = action_tx.send(Action::JobStarted(task_id));

                let tx_progress = action_tx.clone();
                let on_progress = move |progress| {
                    let _ = tx_progress.send(Action::JobProgress(task_id, progress));
                };

                match ffmpeg.execute(command, on_progress).await {
                    Ok(size) => {
                        let _ = action_tx.send(Action::JobCompleted(task_id, size));
                    }
                    Err(e) => {
                        let _ = action_tx.send(Action::JobFailed(task_id, e.to_string()));
                    }
                }
            });

            state.register_task(job_id, handle).await;
        }
    }

    fn handle_filter_dialog_confirm(state: &mut ApplicationState) {
        let (tab, idx, custom_text, is_editing) = match &state.filter_dialog {
            Some(dlg) => (
                dlg.current_tab,
                dlg.selected_index,
                dlg.custom_text.clone(),
                dlg.editing_custom,
            ),
            None => return,
        };

        use crate::app::filter_dialog::FilterTab;
        use crate::domain::{AudioFilter, Filter, VideoFilter};

        match tab {
            FilterTab::Video => {
                let preset_filter = state
                    .filter_dialog
                    .as_ref()
                    .and_then(|dlg| dlg.video_presets.get(idx))
                    .map(|p| (p.label, p.filter.clone()));

                if let Some((label, filter)) = preset_filter {
                    match filter {
                        Filter::Video(VideoFilter::Custom { .. }) => {
                            if !is_editing && custom_text.trim().is_empty() {
                                if let Some(dlg_mut) = &mut state.filter_dialog {
                                    dlg_mut.editing_custom = true;
                                }
                                state.set_status("Type custom video filter and press Enter");
                                return;
                            }
                            if !custom_text.trim().is_empty() {
                                state.builder.filters.add_video_filter(VideoFilter::Custom {
                                    filter_string: custom_text.clone(),
                                });
                                if let Some(dlg_mut) = &mut state.filter_dialog {
                                    dlg_mut.custom_text.clear();
                                    dlg_mut.editing_custom = false;
                                }
                                state.set_status(format!("Added custom video filter: {custom_text}"));
                            }
                        }
                        Filter::Video(vf) => {
                            state.builder.filters.add_video_filter(vf);
                            state.set_status(format!("Added video filter: {label}"));
                        }
                        _ => {}
                    }
                }
            }
            FilterTab::Audio => {
                let preset_filter = state
                    .filter_dialog
                    .as_ref()
                    .and_then(|dlg| dlg.audio_presets.get(idx))
                    .map(|p| (p.label, p.filter.clone()));

                if let Some((label, filter)) = preset_filter {
                    match filter {
                        Filter::Audio(AudioFilter::Custom { .. }) => {
                            if !is_editing && custom_text.trim().is_empty() {
                                if let Some(dlg_mut) = &mut state.filter_dialog {
                                    dlg_mut.editing_custom = true;
                                }
                                state.set_status("Type custom audio filter and press Enter");
                                return;
                            }
                            if !custom_text.trim().is_empty() {
                                state.builder.filters.add_audio_filter(AudioFilter::Custom {
                                    filter_string: custom_text.clone(),
                                });
                                if let Some(dlg_mut) = &mut state.filter_dialog {
                                    dlg_mut.custom_text.clear();
                                    dlg_mut.editing_custom = false;
                                }
                                state.set_status(format!("Added custom audio filter: {custom_text}"));
                            }
                        }
                        Filter::Audio(af) => {
                            state.builder.filters.add_audio_filter(af);
                            state.set_status(format!("Added audio filter: {label}"));
                        }
                        _ => {}
                    }
                }
            }
            FilterTab::Active => {
                Self::handle_filter_dialog_remove(state);
            }
        }
    }

    fn handle_filter_dialog_remove(state: &mut ApplicationState) {
        let idx = match &state.filter_dialog {
            Some(dlg) => dlg.selected_index,
            None => return,
        };

        if state.builder.filters.remove_video_filter(idx).is_some() {
            state.set_status("Removed video filter");
        } else {
            let vf_len = state.builder.filters.video_filters().len();
            if idx >= vf_len
                && state
                    .builder
                    .filters
                    .remove_audio_filter(idx - vf_len)
                    .is_some()
            {
                state.set_status("Removed audio filter");
            }
        }

        let count = state.builder.filters.video_filters().len()
            + state.builder.filters.audio_filters().len();
        if let Some(dlg) = &mut state.filter_dialog {
            if count == 0 {
                dlg.selected_index = 0;
            } else if dlg.selected_index >= count {
                dlg.selected_index = count - 1;
            }
        }
    }

    async fn clear_completed_jobs(state: &mut ApplicationState) {
        {
            let mut jobs = state.jobs().write().await;
            jobs.retain(|_, job| !job.status().is_terminal());
        }
        state.set_status("Cleared completed jobs");
    }

    async fn cancel_job(state: &mut ApplicationState, id: crate::domain::JobId) {
        state.cancel_task(id).await;
        {
            let mut jobs = state.jobs().write().await;
            if let Some(job) = jobs.get_mut(&id) {
                job.cancel();
            }
        }
        {
            let mut active = state.active_jobs().write().await;
            active.retain(|&j| j != id);
        }
        {
            let mut queue = state.job_queue().lock().await;
            queue.retain(|&j| j != id);
        }
        state.set_status(format!("Job {id} cancelled"));
        Self::process_queue(state).await;
    }

    async fn retry_job(state: &mut ApplicationState, id: crate::domain::JobId) {
        let mut requeued = false;
        {
            let mut jobs = state.jobs().write().await;
            if let Some(job) = jobs.get_mut(&id) {
                requeued = job.requeue();
            }
        }

        if requeued {
            {
                let mut queue = state.job_queue().lock().await;
                queue.push_back(id);
            }
            state.set_status(format!("Job {id} requeued for retry"));
            Self::process_queue(state).await;
        } else {
            state.set_status(format!("Cannot retry job {id}"));
        }
    }
}

