use crate::app::{
    Action, ActionHandler, ApplicationState, BuilderField, NavigationTarget, Screen,
};
use crate::app::file_browser::FileBrowserTarget;
use crate::domain::{AudioCodec, ContainerFormat, EncodingPreset, VideoCodec};
use crate::ui::components::{
    render_file_browser, render_filter_dialog, render_nav_bar, render_status_bar,
};
use crate::ui::screens::{
    render_builder, render_dashboard, render_help, render_inspector, render_logs, render_queue,
};
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use std::io::stdout;

pub async fn run_app(mut state: ApplicationState) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut event_stream = EventStream::new();
    let mut action_rx = state
        .take_action_rx()
        .ok_or("Failed to acquire action receiver")?;

    let mut ticker = tokio::time::interval(std::time::Duration::from_millis(100));

    while state.running {
        let jobs_snapshot = state.get_jobs().await;

        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(1),
                ])
                .split(frame.area());

            render_nav_bar(frame, chunks[0], state.screen);

            match state.screen {
                Screen::Dashboard => render_dashboard(frame, chunks[1], &jobs_snapshot),
                Screen::Builder => render_builder(frame, chunks[1], &state),
                Screen::Queue => render_queue(frame, chunks[1], &state, &jobs_snapshot),
                Screen::Logs => render_logs(frame, chunks[1], &state, &jobs_snapshot),
                Screen::Inspector => render_inspector(frame, chunks[1], &state),
                Screen::Help => render_help(frame, chunks[1]),
            }

            render_status_bar(frame, chunks[2], &state);

            // Render file browser popup overlay when open
            if let Some(browser) = &state.file_browser {
                render_file_browser(frame, frame.area(), browser);
            }

            // Render filter dialog popup overlay when open
            if state.filter_dialog.is_some() {
                render_filter_dialog(frame, frame.area(), &state);
            }
        })?;

        tokio::select! {
            _ = ticker.tick() => {
                ActionHandler::handle(&mut state, Action::Tick).await;
            }
            maybe_event = event_stream.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_event {
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        state.quit();
                        break;
                    }

                    if let Some(act) = handle_key_event(key.code, &mut state, &jobs_snapshot) {
                        ActionHandler::handle(&mut state, act).await;
                    }
                }
            }
            maybe_action = action_rx.recv() => {
                if let Some(action) = maybe_action {
                    ActionHandler::handle(&mut state, action).await;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_key_event(
    code: KeyCode,
    state: &mut ApplicationState,
    jobs: &[crate::domain::Job],
) -> Option<Action> {
    // ── Priority 1: Filter dialog modal captures input ──────────────
    if let Some(dlg) = &state.filter_dialog {
        return handle_filter_dialog_keys(code, dlg);
    }

    // ── Priority 2: File browser modal captures ALL input ──────────
    if let Some(browser) = &state.file_browser {
        return handle_file_browser_keys(code, browser.target);
    }

    // ── Priority 2: Raw command mode captures character input ──────
    if state.screen == Screen::Builder && state.builder.raw_command_mode {
        return handle_raw_command_keys(code);
    }

    // ── Priority 3: Global navigation (only when not in raw/browser) ──
    match code {
        KeyCode::Char('1') => return Some(Action::Navigate(NavigationTarget::Dashboard)),
        KeyCode::Char('2') => return Some(Action::Navigate(NavigationTarget::Builder)),
        KeyCode::Char('3') => return Some(Action::Navigate(NavigationTarget::Queue)),
        KeyCode::Char('4') => return Some(Action::Navigate(NavigationTarget::Logs)),
        KeyCode::Char('5') => return Some(Action::Navigate(NavigationTarget::Inspector)),
        KeyCode::Char('?') => return Some(Action::Navigate(NavigationTarget::Help)),
        KeyCode::Char('q') | KeyCode::Esc => return Some(Action::Quit),
        _ => {}
    }

    // ── Priority 4: Screen-specific keys ──────────────────────────
    match state.screen {
        Screen::Builder => handle_builder_keys(code, state),
        Screen::Queue => handle_queue_keys(code, state, jobs),
        Screen::Logs => handle_logs_keys(code),
        Screen::Inspector => handle_inspector_keys(code, state),
        _ => None,
    }
}

/// Handle keys when the file browser popup is open (modal).
fn handle_file_browser_keys(code: KeyCode, target: FileBrowserTarget) -> Option<Action> {
    match target {
        FileBrowserTarget::Input => match code {
            KeyCode::Up => Some(Action::FileBrowserSelectPrev),
            KeyCode::Down => Some(Action::FileBrowserSelectNext),
            KeyCode::Enter => Some(Action::FileBrowserConfirm),
            KeyCode::Esc => Some(Action::FileBrowserCancel),
            KeyCode::Backspace => Some(Action::FileBrowserFilterBackspace),
            KeyCode::Tab => Some(Action::FileBrowserNavigateUp),
            KeyCode::Char(c) => Some(Action::FileBrowserFilterAppend(c)),
            _ => None,
        },
        FileBrowserTarget::Output => match code {
            KeyCode::Up => Some(Action::FileBrowserSelectPrev),
            KeyCode::Down => Some(Action::FileBrowserSelectNext),
            KeyCode::Enter => Some(Action::FileBrowserConfirm),
            KeyCode::Esc => Some(Action::FileBrowserCancel),
            KeyCode::Backspace => Some(Action::FileBrowserFilenameBackspace),
            KeyCode::Tab => Some(Action::FileBrowserNavigateUp),
            KeyCode::Char(c) => Some(Action::FileBrowserFilenameAppend(c)),
            _ => None,
        },
    }
}

/// Handle keys when the Filter Dialog popup is open (modal).
fn handle_filter_dialog_keys(
    code: KeyCode,
    dlg: &crate::app::FilterDialogState,
) -> Option<Action> {
    if dlg.editing_custom {
        match code {
            KeyCode::Enter => Some(Action::FilterDialogConfirm),
            KeyCode::Esc => Some(Action::CloseFilterDialog),
            KeyCode::Backspace => Some(Action::FilterDialogCustomBackspace),
            KeyCode::Char(c) => Some(Action::FilterDialogCustomAppend(c)),
            _ => None,
        }
    } else {
        match code {
            KeyCode::Tab | KeyCode::Right => Some(Action::FilterDialogNextTab),
            KeyCode::BackTab | KeyCode::Left => Some(Action::FilterDialogPrevTab),
            KeyCode::Up => Some(Action::FilterDialogSelectPrev),
            KeyCode::Down => Some(Action::FilterDialogSelectNext),
            KeyCode::Enter => Some(Action::FilterDialogConfirm),
            KeyCode::Char('d') | KeyCode::Delete => Some(Action::FilterDialogRemoveSelected),
            KeyCode::Char('c') => Some(Action::FilterDialogClearAll),
            KeyCode::Esc => Some(Action::CloseFilterDialog),
            _ => None,
        }
    }
}

/// Handle keys when raw command mode is active in the builder.
/// Only Escape and 'r' (with Ctrl) escape raw mode; everything else is text input.
fn handle_raw_command_keys(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Esc => Some(Action::ToggleRawCommandMode),
        KeyCode::Backspace => Some(Action::RawCommandBackspace),
        KeyCode::Enter => Some(Action::BuildJob),
        KeyCode::Char(c) => Some(Action::RawCommandAppend(c)),
        _ => None,
    }
}

fn handle_builder_keys(code: KeyCode, state: &ApplicationState) -> Option<Action> {
    match code {
        KeyCode::Tab | KeyCode::Down => Some(Action::NextField),
        KeyCode::BackTab | KeyCode::Up => Some(Action::PrevField),
        KeyCode::Char('r') => Some(Action::ToggleRawCommandMode),
        KeyCode::Char('f') => Some(Action::OpenFilterDialog),
        // Enter: open file browser for Input/Output; open filter dialog for Filters; build job for other fields
        KeyCode::Enter => match state.builder.current_field {
            BuilderField::Input => Some(Action::OpenFileBrowser(FileBrowserTarget::Input)),
            BuilderField::Output => Some(Action::OpenFileBrowser(FileBrowserTarget::Output)),
            BuilderField::Filters => Some(Action::OpenFilterDialog),
            _ => Some(Action::BuildJob),
        },
        KeyCode::Char('b') => Some(Action::BuildJob),
        KeyCode::Char('p') => {
            let idx = state.builder.preset_index.min(state.presets.len().saturating_sub(1));
            let name = state.presets.get(idx)?.name.clone();
            Some(Action::LoadPreset(name))
        }
        KeyCode::Right => match state.builder.current_field {
            BuilderField::Filters => Some(Action::OpenFilterDialog),
            BuilderField::VideoCodec => {
                let all = VideoCodec::all();
                let idx = all.iter().position(|&c| c == state.builder.video_codec).unwrap_or(0);
                let next = all[(idx + 1) % all.len()];
                Some(Action::SetVideoCodec(next))
            }
            BuilderField::AudioCodec => {
                let all = AudioCodec::all();
                let idx = all.iter().position(|&c| c == state.builder.audio_codec).unwrap_or(0);
                let next = all[(idx + 1) % all.len()];
                Some(Action::SetAudioCodec(next))
            }
            BuilderField::Format => {
                let all = ContainerFormat::video_formats();
                let idx = all.iter().position(|&f| f == state.builder.format).unwrap_or(0);
                let next = all[(idx + 1) % all.len()];
                Some(Action::SetFormat(next))
            }
            BuilderField::Quality => {
                let crf = state.builder.crf.saturating_add(1).min(51);
                Some(Action::SetCrf(crf))
            }
            BuilderField::Preset => {
                let all = crate::domain::EncodingPreset::all();
                let idx = all.iter().position(|&p| p == state.builder.preset).unwrap_or(0);
                let next = all[(idx + 1) % all.len()];
                Some(Action::SetPreset(next))
            }
            _ => None,
        },
        KeyCode::Left => match state.builder.current_field {
            BuilderField::Filters => Some(Action::OpenFilterDialog),
            BuilderField::VideoCodec => {
                let all = VideoCodec::all();
                let idx = all.iter().position(|&c| c == state.builder.video_codec).unwrap_or(0);
                let prev = all[(idx + all.len() - 1) % all.len()];
                Some(Action::SetVideoCodec(prev))
            }
            BuilderField::AudioCodec => {
                let all = AudioCodec::all();
                let idx = all.iter().position(|&c| c == state.builder.audio_codec).unwrap_or(0);
                let prev = all[(idx + all.len() - 1) % all.len()];
                Some(Action::SetAudioCodec(prev))
            }
            BuilderField::Format => {
                let all = ContainerFormat::video_formats();
                let idx = all.iter().position(|&f| f == state.builder.format).unwrap_or(0);
                let prev = all[(idx + all.len() - 1) % all.len()];
                Some(Action::SetFormat(prev))
            }
            BuilderField::Quality => {
                let crf = state.builder.crf.saturating_sub(1);
                Some(Action::SetCrf(crf))
            }
            BuilderField::Preset => {
                let all = EncodingPreset::all();
                let idx = all.iter().position(|&p| p == state.builder.preset).unwrap_or(0);
                let prev = all[(idx + all.len() - 1) % all.len()];
                Some(Action::SetPreset(prev))
            }
            _ => None,
        },
        KeyCode::Char('i') => state.builder.input_path.as_ref().map(|path| Action::InspectFile(path.clone())),
        _ => None,
    }
}

fn handle_queue_keys(code: KeyCode, state: &mut ApplicationState, jobs: &[crate::domain::Job]) -> Option<Action> {
    if jobs.is_empty() {
        return None;
    }

    let selected_idx = state.queue_state.selected_index.min(jobs.len().saturating_sub(1));
    let selected_job = jobs.get(selected_idx);

    match code {
        KeyCode::Up => {
            state.queue_state.selected_index = selected_idx.saturating_sub(1);
            None
        }
        KeyCode::Down => {
            state.queue_state.selected_index = (selected_idx + 1).min(jobs.len().saturating_sub(1));
            None
        }
        KeyCode::Char('s') => Some(Action::StartQueue),
        KeyCode::Char('p') => Some(Action::PauseQueue),
        KeyCode::Char('c') => selected_job.map(|j| Action::CancelJob(j.id())),
        KeyCode::Char('r') => selected_job.map(|j| Action::RetryJob(j.id())),
        KeyCode::Char('K') => selected_job.map(|j| Action::MoveJobUp(j.id())),
        KeyCode::Char('J') => selected_job.map(|j| Action::MoveJobDown(j.id())),
        KeyCode::Char('x') => Some(Action::ClearCompleted),
        _ => None,
    }
}

fn handle_logs_keys(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char('t') => Some(Action::ToggleRawLogs),
        KeyCode::Char('a') => Some(Action::ToggleAutoScroll),
        KeyCode::Up => Some(Action::ScrollUp),
        KeyCode::Down => Some(Action::ScrollDown),
        _ => None,
    }
}

fn handle_inspector_keys(code: KeyCode, state: &ApplicationState) -> Option<Action> {
    match code {
        KeyCode::Char('i') => state.builder.input_path.as_ref().map(|path| Action::InspectFile(path.clone())),
        _ => None,
    }
}

