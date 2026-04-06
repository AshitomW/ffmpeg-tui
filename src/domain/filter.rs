///! Filter defintions for video and audio processing
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VideoFilter {
    Scale {
        width: ScaleDimension,
        height: ScaleDimension,
    },

    Crop {
        width: u32,
        height: u32,
        x: u32,
        y: u32,
    },

    Fps {
        rate: f64,
    },
    Rotate {
        angle: RotationAngle,
    },
    Overlay {
        x: String,
        y: String,
    },
    Pad {
        width: u32,
        height: u32,
        x: u32,
        y: u32,
        color: String,
    },
    Trim {
        start: Option<f64>,
        end: Option<f64>,
    },
    Deinterlace,
    Denoise {
        strength: DenoiseStrength,
    },
    Sharpen {
        amount: f64,
    },
    Brightness {
        value: f64,
    },
    Contrast {
        value: f64,
    },
    Saturation {
        value: f64,
    },
    Hue {
        degrees: f64,
    },
    Custom {
        filter_string: String,
    },
}

impl VideoFilter {
    #[must_use]
    pub fn to_ffmpeg_string(&self) -> String {
        match self {
            Self::Scale { width, height } => {
                format!("scale={}:{}", width.to_ffmpeg(), height.to_ffmpeg())
            }
            Self::Crop {
                width,
                height,
                x,
                y,
            } => {
                format!("crop={width}:{height}:{x}:{y}")
            }
            Self::Fps { rate } => format!("fps={rate}"),
            Self::Rotate { angle } => angle.to_ffmpeg_filter(),
            Self::Overlay { x, y } => format!("overlay={x}:{y}"),
            Self::Pad {
                width,
                height,
                x,
                y,
                color,
            } => {
                format!("pad={width}:{height}:{x}:{y}:{color}")
            }
            Self::Trim { start, end } => {
                let mut parts = Vec::new();
                if let Some(s) = start {
                    parts.push(format!("start={s}"));
                }
                if let Some(e) = end {
                    parts.push(format!("end={e}"));
                }
                format!("trim={}", parts.join(":"))
            }
            Self::Deinterlace => "yadif".to_string(),
            Self::Denoise { strength } => format!("hqdn3d={}", strength.value()),
            Self::Sharpen { amount } => format!("unsharp=5:5:{amount}:5:5:{amount}"),
            Self::Brightness { value } => format!("eq=brightness={value}"),
            Self::Contrast { value } => format!("eq=contrast={value}"),
            Self::Saturation { value } => format!("eq=saturation={value}"),
            Self::Hue { degrees } => format!("hue=h={degrees}"),
            Self::Custom { filter_string } => filter_string.clone(),
        }
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Scale { .. } => "Scale",
            Self::Crop { .. } => "Crop",
            Self::Fps { .. } => "FPS",
            Self::Rotate { .. } => "Rotate",
            Self::Overlay { .. } => "Overlay",
            Self::Pad { .. } => "Pad",
            Self::Trim { .. } => "Trim",
            Self::Deinterlace => "Deinterlace",
            Self::Denoise { .. } => "Denoise",
            Self::Sharpen { .. } => "Sharpen",
            Self::Brightness { .. } => "Brightness",
            Self::Contrast { .. } => "Contrast",
            Self::Saturation { .. } => "Saturation",
            Self::Hue { .. } => "Hue",
            Self::Custom { .. } => "Custom",
        }
    }
}

///Dimension specification for scaling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleDimension {
    Exact(u32),
    Keep,
    KeepAspect,
}

impl ScaleDimension {
    fn to_ffmpeg(&self) -> String {
        match self {
            Self::Exact(v) => v.to_string(),
            Self::Keep => "-1".to_string(),
            Self::KeepAspect => "-2".to_string(),
        }
    }
}

/// Rotation Angle Options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RotationAngle {
    Clockwise90,
    CounterClockwise90,
    Rotate180,
    FlipHorizontal,
    FlipVertical,
}

impl RotationAngle {
    fn to_ffmpeg_filter(&self) -> String {
        match self {
            Self::Clockwise90 => "transpose=1".to_string(),
            Self::CounterClockwise90 => "transpose=2".to_string(),
            Self::Rotate180 => "transpose=1,transpose=1".to_string(),
            Self::FlipHorizontal => "hflip".to_string(),
            Self::FlipVertical => "vflip".to_string(),
        }
    }
}

/// Denoise Strength Levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DenoiseStrength {
    Light,
    #[default]
    Medium,
    Strong,
}

impl DenoiseStrength {
    const fn value(&self) -> &'static str {
        match self {
            Self::Light => "2:1:2:3",
            Self::Medium => "4:3:6:4.5",
            Self::Strong => "6:4:8:6",
        }
    }
}

/// Audio filter with typed Params
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioFilter {
    Volume {
        level: f64,
    },
    Normalize,
    Fade {
        fade_type: FadeType,
        start: f64,
        duration: f64,
    },
    HighPass {
        frequency: f64,
    },
    LowPass {
        frequency: f64,
    },
    Tempo {
        rate: f64,
    },
    Resample {
        rate: u32,
    },
    Channels {
        count: u8,
    },
    Custom {
        filter_string: String,
    },
}

impl AudioFilter {
    #[must_use]
    pub fn to_ffmpeg_string(&self) -> String {
        match self {
            Self::Volume { level } => format!("volume={level}"),
            Self::Normalize => "loudnorm=I=-16:TP=-1.5:LRA=11".to_string(),
            Self::Fade {
                fade_type,
                start,
                duration,
            } => {
                format!("a{}=st={start}:d={duration}", fade_type.name())
            }
            Self::HighPass { frequency } => format!("highpass=f={frequency}"),
            Self::LowPass { frequency } => format!("lowpass=f={frequency}"),
            Self::Tempo { rate } => format!("atempo={rate}"),
            Self::Resample { rate } => format!("aresample={rate}"),
            Self::Channels { count } => format!("pan={}c", count),
            Self::Custom { filter_string } => filter_string.clone(),
        }
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Volume { .. } => "Volume",
            Self::Normalize => "Normalize",
            Self::Fade { .. } => "Fade",
            Self::HighPass { .. } => "High Pass",
            Self::LowPass { .. } => "Low Pass",
            Self::Resample { .. } => "Resample",
            Self::Channels { .. } => "Channels",
            Self::Custom { .. } => "Custom",
            Self::Tempo { .. } => "Tempo",
        }
    }
}

impl fmt::Display for AudioFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_ffmpeg_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FadeType {
    In,
    Out,
}

impl FadeType {
    const fn name(&self) -> &'static str {
        match self {
            Self::In => "fadein",
            Self::Out => "fadeout",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Filter {
    Video(VideoFilter),
    Audio(AudioFilter),
}

impl Filter {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Video(f) => f.name(),
            Self::Audio(f) => f.name(),
        }
    }

    #[must_use]
    pub const fn is_video(&self) -> bool {
        matches!(self, Self::Video(_))
    }
}

// Chain of filter to be applied in order
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FilterChain {
    video_filters: Vec<VideoFilter>,
    audio_filters: Vec<AudioFilter>,
}

impl FilterChain {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_video_filter(&mut self, filter: VideoFilter) {
        self.video_filters.push(filter);
    }

    pub fn add_audio_filter(&mut self, filter: AudioFilter) {
        self.audio_filters.push(filter);
    }

    pub fn remove_video_filter(&mut self, index: usize) -> Option<VideoFilter> {
        if index < self.video_filters.len() {
            Some(self.video_filters.remove(index))
        } else {
            None
        }
    }
    pub fn remove_audio_filter(&mut self, index: usize) -> Option<AudioFilter> {
        if index < self.audio_filters.len() {
            Some(self.audio_filters.remove(index))
        } else {
            None
        }
    }

    pub fn move_video_filter_up(&mut self, index: usize) -> bool {
        if index > 0 && index < self.video_filters.len() {
            self.video_filters.swap(index, index - 1);
            true
        } else {
            false
        }
    }

    pub fn move_video_filter_down(&mut self, index: usize) -> bool {
        if index + 1 < self.video_filters.len() {
            self.video_filters.swap(index, index + 1);
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn video_filter_string(&self) -> Option<String> {
        if self.video_filters.is_empty() {
            None
        } else {
            Some(
                self.video_filters
                    .iter()
                    .map(VideoFilter::to_ffmpeg_string)
                    .collect::<Vec<_>>()
                    .join(","),
            )
        }
    }

    #[must_use]
    pub fn audio_filter_string(&self) -> Option<String> {
        if self.audio_filters.is_empty() {
            None
        } else {
            Some(
                self.audio_filters
                    .iter()
                    .map(AudioFilter::to_ffmpeg_string)
                    .collect::<Vec<_>>()
                    .join(","),
            )
        }
    }

    #[must_use]
    pub fn video_filters(&self) -> &[VideoFilter] {
        &self.video_filters
    }

    #[must_use]
    pub fn audio_filters(&self) -> &[AudioFilter] {
        &self.audio_filters
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.video_filters.is_empty() && self.audio_filters.is_empty()
    }

    pub fn clear(&mut self) {
        self.video_filters.clear();
        self.audio_filters.clear();
    }
}
