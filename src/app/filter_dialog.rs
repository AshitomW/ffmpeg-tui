use crate::domain::{
    AudioFilter, DenoiseStrength, FadeType, Filter, RotationAngle, ScaleDimension, VideoFilter,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterTab {
    #[default]
    Video,
    Audio,
    Active,
}

impl FilterTab {
    pub const fn all() -> &'static [Self] {
        &[Self::Video, Self::Audio, Self::Active]
    }

    pub const fn title(&self) -> &'static str {
        match self {
            Self::Video => "Video Filters",
            Self::Audio => "Audio Filters",
            Self::Active => "Active Filters",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilterPresetItem {
    pub label: &'static str,
    pub description: &'static str,
    pub filter: Filter,
}

#[derive(Debug, Clone)]
pub struct FilterDialogState {
    pub current_tab: FilterTab,
    pub selected_index: usize,
    pub custom_text: String,
    pub editing_custom: bool,
    pub video_presets: Vec<FilterPresetItem>,
    pub audio_presets: Vec<FilterPresetItem>,
}

impl Default for FilterDialogState {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterDialogState {
    #[must_use]
    pub fn new() -> Self {
        let video_presets = vec![
            FilterPresetItem {
                label: "Scale 1080p",
                description: "Resize video to 1920x1080 resolution",
                filter: Filter::Video(VideoFilter::Scale {
                    width: ScaleDimension::Exact(1920),
                    height: ScaleDimension::Exact(1080),
                }),
            },
            FilterPresetItem {
                label: "Scale 720p",
                description: "Resize video to 1280x720 resolution",
                filter: Filter::Video(VideoFilter::Scale {
                    width: ScaleDimension::Exact(1280),
                    height: ScaleDimension::Exact(720),
                }),
            },
            FilterPresetItem {
                label: "Scale 4K",
                description: "Resize video to 3840x2160 ultra HD",
                filter: Filter::Video(VideoFilter::Scale {
                    width: ScaleDimension::Exact(3840),
                    height: ScaleDimension::Exact(2160),
                }),
            },
            FilterPresetItem {
                label: "Scale Auto Height (1080p)",
                description: "Maintain aspect ratio with height 1080",
                filter: Filter::Video(VideoFilter::Scale {
                    width: ScaleDimension::KeepAspect,
                    height: ScaleDimension::Exact(1080),
                }),
            },
            FilterPresetItem {
                label: "FPS 30",
                description: "Convert video frame rate to 30 fps",
                filter: Filter::Video(VideoFilter::Fps { rate: 30.0 }),
            },
            FilterPresetItem {
                label: "FPS 60",
                description: "Convert video frame rate to 60 fps",
                filter: Filter::Video(VideoFilter::Fps { rate: 60.0 }),
            },
            FilterPresetItem {
                label: "Rotate 90° Clockwise",
                description: "Rotate video 90 degrees CW (transpose=1)",
                filter: Filter::Video(VideoFilter::Rotate {
                    angle: RotationAngle::Clockwise90,
                }),
            },
            FilterPresetItem {
                label: "Rotate 90° Counter-Clockwise",
                description: "Rotate video 90 degrees CCW (transpose=2)",
                filter: Filter::Video(VideoFilter::Rotate {
                    angle: RotationAngle::CounterClockwise90,
                }),
            },
            FilterPresetItem {
                label: "Rotate 180°",
                description: "Rotate video upside down (180 degrees)",
                filter: Filter::Video(VideoFilter::Rotate {
                    angle: RotationAngle::Rotate180,
                }),
            },
            FilterPresetItem {
                label: "Flip Horizontal",
                description: "Mirror video horizontally (hflip)",
                filter: Filter::Video(VideoFilter::Rotate {
                    angle: RotationAngle::FlipHorizontal,
                }),
            },
            FilterPresetItem {
                label: "Flip Vertical",
                description: "Flip video vertically (vflip)",
                filter: Filter::Video(VideoFilter::Rotate {
                    angle: RotationAngle::FlipVertical,
                }),
            },
            FilterPresetItem {
                label: "Deinterlace",
                description: "Deinterlace interlaced video using yadif",
                filter: Filter::Video(VideoFilter::Deinterlace),
            },
            FilterPresetItem {
                label: "Denoise (Medium)",
                description: "Reduce video noise using hqdn3d filter",
                filter: Filter::Video(VideoFilter::Denoise {
                    strength: DenoiseStrength::Medium,
                }),
            },
            FilterPresetItem {
                label: "Sharpen",
                description: "Sharpen video details using unsharp filter",
                filter: Filter::Video(VideoFilter::Sharpen { amount: 1.0 }),
            },
            FilterPresetItem {
                label: "Custom Video Filter",
                description: "Type an arbitrary FFmpeg video filter string (-vf)",
                filter: Filter::Video(VideoFilter::Custom {
                    filter_string: String::new(),
                }),
            },
        ];

        let audio_presets = vec![
            FilterPresetItem {
                label: "Normalize Volume (EBU R128)",
                description: "Normalize audio loudness using loudnorm",
                filter: Filter::Audio(AudioFilter::Normalize),
            },
            FilterPresetItem {
                label: "Volume +50%",
                description: "Increase volume level to 1.5",
                filter: Filter::Audio(AudioFilter::Volume { level: 1.5 }),
            },
            FilterPresetItem {
                label: "Volume -50%",
                description: "Decrease volume level to 0.5",
                filter: Filter::Audio(AudioFilter::Volume { level: 0.5 }),
            },
            FilterPresetItem {
                label: "Fade In (2 sec)",
                description: "Apply audio fade in over 2 seconds",
                filter: Filter::Audio(AudioFilter::Fade {
                    fade_type: FadeType::In,
                    start: 0.0,
                    duration: 2.0,
                }),
            },
            FilterPresetItem {
                label: "Fade Out (3 sec)",
                description: "Apply audio fade out over 3 seconds",
                filter: Filter::Audio(AudioFilter::Fade {
                    fade_type: FadeType::Out,
                    start: 0.0,
                    duration: 3.0,
                }),
            },
            FilterPresetItem {
                label: "High Pass Filter (100Hz)",
                description: "Cut off low frequencies below 100Hz",
                filter: Filter::Audio(AudioFilter::HighPass { frequency: 100.0 }),
            },
            FilterPresetItem {
                label: "Low Pass Filter (12kHz)",
                description: "Cut off high frequencies above 12kHz",
                filter: Filter::Audio(AudioFilter::LowPass {
                    frequency: 12000.0,
                }),
            },
            FilterPresetItem {
                label: "Resample 48kHz",
                description: "Resample audio sample rate to 48000Hz",
                filter: Filter::Audio(AudioFilter::Resample { rate: 48000 }),
            },
            FilterPresetItem {
                label: "Resample 44.1kHz",
                description: "Resample audio sample rate to 44100Hz",
                filter: Filter::Audio(AudioFilter::Resample { rate: 44100 }),
            },
            FilterPresetItem {
                label: "Channels: Stereo (2ch)",
                description: "Convert audio channel layout to stereo",
                filter: Filter::Audio(AudioFilter::Channels { count: 2 }),
            },
            FilterPresetItem {
                label: "Channels: Mono (1ch)",
                description: "Convert audio channel layout to mono",
                filter: Filter::Audio(AudioFilter::Channels { count: 1 }),
            },
            FilterPresetItem {
                label: "Tempo 1.25x",
                description: "Adjust audio tempo speed to 1.25x without pitch shift",
                filter: Filter::Audio(AudioFilter::Tempo { rate: 1.25 }),
            },
            FilterPresetItem {
                label: "Custom Audio Filter",
                description: "Type an arbitrary FFmpeg audio filter string (-af)",
                filter: Filter::Audio(AudioFilter::Custom {
                    filter_string: String::new(),
                }),
            },
        ];

        Self {
            current_tab: FilterTab::Video,
            selected_index: 0,
            custom_text: String::new(),
            editing_custom: false,
            video_presets,
            audio_presets,
        }
    }

    pub fn next_tab(&mut self) {
        let tabs = FilterTab::all();
        let idx = tabs.iter().position(|&t| t == self.current_tab).unwrap_or(0);
        self.current_tab = tabs[(idx + 1) % tabs.len()];
        self.selected_index = 0;
        self.editing_custom = false;
    }

    pub fn prev_tab(&mut self) {
        let tabs = FilterTab::all();
        let idx = tabs.iter().position(|&t| t == self.current_tab).unwrap_or(0);
        self.current_tab = tabs[(idx + tabs.len() - 1) % tabs.len()];
        self.selected_index = 0;
        self.editing_custom = false;
    }

    pub fn select_next(&mut self, active_count: usize) {
        if self.editing_custom {
            return;
        }
        let max = match self.current_tab {
            FilterTab::Video => self.video_presets.len(),
            FilterTab::Audio => self.audio_presets.len(),
            FilterTab::Active => active_count,
        };
        if max > 0 {
            self.selected_index = (self.selected_index + 1) % max;
        }
    }

    pub fn select_prev(&mut self, active_count: usize) {
        if self.editing_custom {
            return;
        }
        let max = match self.current_tab {
            FilterTab::Video => self.video_presets.len(),
            FilterTab::Audio => self.audio_presets.len(),
            FilterTab::Active => active_count,
        };
        if max > 0 {
            self.selected_index = (self.selected_index + max - 1) % max;
        }
    }

    pub fn custom_append(&mut self, c: char) {
        if self.editing_custom {
            self.custom_text.push(c);
        }
    }

    pub fn custom_backspace(&mut self) {
        if self.editing_custom {
            self.custom_text.pop();
        }
    }

    #[must_use]
    pub fn active_count(&self, filters: &crate::domain::FilterChain) -> usize {
        filters.video_filters().len() + filters.audio_filters().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_dialog_tab_navigation() {
        let mut dlg = FilterDialogState::new();
        assert_eq!(dlg.current_tab, FilterTab::Video);

        dlg.next_tab();
        assert_eq!(dlg.current_tab, FilterTab::Audio);

        dlg.next_tab();
        assert_eq!(dlg.current_tab, FilterTab::Active);

        dlg.next_tab();
        assert_eq!(dlg.current_tab, FilterTab::Video);

        dlg.prev_tab();
        assert_eq!(dlg.current_tab, FilterTab::Active);
    }

    #[test]
    fn test_filter_dialog_selection() {
        let mut dlg = FilterDialogState::new();
        assert_eq!(dlg.selected_index, 0);

        dlg.select_next(10);
        assert_eq!(dlg.selected_index, 1);

        dlg.select_prev(10);
        assert_eq!(dlg.selected_index, 0);

        dlg.select_prev(10);
        assert_eq!(dlg.selected_index, 9);
    }

    #[test]
    fn test_filter_dialog_custom_text() {
        let mut dlg = FilterDialogState::new();
        dlg.editing_custom = true;
        dlg.custom_append('v');
        dlg.custom_append('f');
        assert_eq!(dlg.custom_text, "vf");

        dlg.custom_backspace();
        assert_eq!(dlg.custom_text, "v");
    }
}
