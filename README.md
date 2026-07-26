# FFmpeg TUI

A terminal user interface (TUI) application written in Rust for building, running, and managing FFmpeg video and audio processing jobs.

## Features

- Interactive command builder for input/output files, video/audio codecs, container formats, quality, and filter chains.
- Dedicated Filter Dialog for adding and managing preset video, audio, and custom FFmpeg filters.
- Raw command editing mode for custom FFmpeg flags.
- Built-in encoding presets (H.264, H.265, WebM, MP3/FLAC audio extraction, Reels, etc.).
- Job queue management with start, pause, cancel, retry, and reordering controls.
- Real-time encoding progress monitoring (FPS, speed, bitrate, percentage, and ETA).
- Media file inspection using `ffprobe`.
- Dual-mode log viewer for parsed progress breakdown and raw stdout/stderr logs.

## Requirements

- Rust (Edition 2024 / 1.85+)
- `ffmpeg` and `ffprobe` installed and available in system PATH

## Installation & Usage

Build and run the project using `cargo`:

```bash
cargo check
cargo run
```

## Keybindings

### Navigation
- `1` - `5`: Switch screens (Dashboard, Builder, Queue, Logs, Inspector)
- `?`: Toggle Help overlay
- `q` / `Esc`: Quit application

### Builder Screen
- `Tab` / `Shift+Tab`: Navigate fields
- `←` / `→`: Cycle option values
- `f` / `Enter` (on Filters field): Open Filter Dialog
- `p`: Load preset
- `r`: Toggle raw command mode
- `b` / `Enter`: Build and queue job

### Filter Dialog Overlay
- `Tab` / `←` / `→`: Switch tabs (Video, Audio, Active Filters)
- `↑` / `↓`: Navigate list items
- `Enter`: Add preset or edit custom filter
- `d` / `Delete`: Remove selected active filter
- `c`: Clear all active filters
- `Esc`: Close Filter Dialog

### Queue Screen
- `s`: Start queue
- `p`: Pause queue
- `c`: Cancel selected job
- `r`: Retry failed job
- `K` / `J`: Move job up/down in queue
- `x`: Clear completed jobs

### Logs Screen
- `t`: Toggle raw stdout/stderr logs vs clean progress
- `a`: Toggle autoscroll

## Architecture

- `domain`: Domain models for codecs, formats, filters, commands, and jobs.
- `app`: State management, file browser, filter dialog, and action handlers.
- `infra`: Asynchronous FFmpeg and FFprobe execution engines.
- `parser`: Real-time stderr output parsing for progress tracking.
- `preset`: Built-in presets and configuration loaders.
- `ui`: Ratatui components, modal overlays, and rendering pipeline.
