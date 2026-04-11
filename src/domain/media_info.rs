use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStream {
    pub index: u32,
    pub codec_name: String,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleStream {
    pub index: u32,
    pub codec_name: String,
    pub codec_long_name: Option<String>,
    pub language: Option<String>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStream {
    pub index: u32,
    pub codec_name: String,
    pub codec_long_name: Option<String>,
    pub profile: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub channel_layout: Option<String>,
    pub sample_fmt: Option<String>,
    pub bit_rate: Option<u64>,
    pub bits_per_sample: Option<u32>,
    pub tags: HashMap<String, String>,
}

impl AudioStream {
    #[must_use]
    pub fn channel_description(&self) -> String {
        match self.channel_layout.as_deref() {
            Some("mono") => "Mono".to_string(),
            Some("stereo") => "Stereo".to_string(),
            Some("5.1") | Some("5.1(side)") => "5.1 Surround".to_string(),
            Some("7.1") => "7.1 Surround".to_string(),
            Some(layout) => layout.to_string(),
            None => self
                .channels
                .map(|c| format!("{c} channel(s)"))
                .unwrap_or_else(|| "Unknown".to_string()),
        }
    }

    #[must_use]
    pub fn sample_rate_khz(&self) -> Option<f64> {
        self.sample_rate.map(|r| f64::from(r) / 1000.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStream {
    pub index: u32,
    pub codec_name: String,
    pub codec_long_name: Option<String>,
    pub profile: Option<String>,
    pub width: u32,
    pub height: u32,
    pub coded_width: Option<u32>,
    pub coded_height: Option<u32>,
    pub display_aspect_ratio: Option<String>,
    pub sample_aspect_ratio: Option<String>,
    pub pix_fmt: Option<String>,
    pub frame_rate: Option<f64>,
    pub avg_frame_rate: Option<f64>,
    pub bit_rate: Option<u64>,
    pub bits_per_raw_sample: Option<u32>,
    pub color_space: Option<String>,
    pub color_range: Option<String>,
    pub color_transfer: Option<String>,
    pub color_primaries: Option<String>,
    pub tags: HashMap<String, String>,
}

impl VideoStream {
    #[must_use]
    pub fn aspect_ratio(&self) -> f64 {
        if self.height > 0 {
            f64::from(self.width) / f64::from(self.height)
        } else {
            0.0
        }
    }

    #[must_use]
    pub fn is_hdr(&self) -> bool {
        matches!(
            self.color_transfer.as_deref(),
            Some("smpte2084") | Some("arib-std-b67")
        )
    }

    #[must_use]
    pub fn resolution_class(&self) -> &'static str {
        match (self.width, self.height) {
            (w, h) if w >= 7680 || h >= 4320 => "8K UHD",
            (w, h) if w >= 3840 || h >= 2160 => "4K UHD",
            (w, h) if w >= 2560 || h >= 1440 => "1440p",
            (w, h) if w >= 1920 || h >= 1080 => "1080p",
            (w, h) if w >= 1280 || h >= 720 => "720p",
            (w, h) if w >= 854 || h >= 480 => "480p",
            _ => "SD",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamInfo {
    Video(VideoStream),
    Audio(AudioStream),
    Subtitle(SubtitleStream),
    Data(DataStream),
}

impl StreamInfo {
    #[must_use]
    pub fn index(&self) -> u32 {
        match self {
            Self::Video(s) => s.index,
            Self::Audio(s) => s.index,
            Self::Subtitle(s) => s.index,
            Self::Data(s) => s.index,
        }
    }

    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Video(_) => "Video",
            Self::Audio(_) => "Audio",
            Self::Subtitle(_) => "Subtitle",
            Self::Data(_) => "Data",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub format_name: String,
    pub format_long_name: Option<String>,
    pub duration: Option<f64>,
    pub size: Option<u64>,
    pub bit_rate: Option<u64>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub path: PathBuf,
    pub format: FormatInfo,
    pub streams: Vec<StreamInfo>,
}

impl MediaInfo {
    #[must_use]
    pub fn video_stream(&self) -> Option<&VideoStream> {
        self.streams.iter().find_map(|s| match s {
            StreamInfo::Video(v) => Some(v),
            _ => None,
        })
    }

    #[must_use]
    pub fn audio_stream(&self) -> Option<&AudioStream> {
        self.streams.iter().find_map(|s| match s {
            StreamInfo::Audio(a) => Some(a),
            _ => None,
        })
    }

    #[must_use]
    pub fn video_streams(&self) -> Vec<&VideoStream> {
        self.streams
            .iter()
            .filter_map(|s| match s {
                StreamInfo::Video(v) => Some(v),
                _ => None,
            })
            .collect()
    }

    #[must_use]
    pub fn audio_streams(&self) -> Vec<&AudioStream> {
        self.streams
            .iter()
            .filter_map(|s| match s {
                StreamInfo::Audio(a) => Some(a),
                _ => None,
            })
            .collect()
    }

    #[must_use]
    pub fn duration_seconds(&self) -> Option<f64> {
        self.format.duration
    }

    #[must_use]
    pub fn file_size(&self) -> Option<u64> {
        self.format.size
    }

    #[must_use]
    pub fn duration_string(&self) -> String {
        match self.duration_seconds() {
            Some(secs) => {
                let duration = Duration::from_secs_f64(secs);
                let hours = duration.as_secs() / 3600;
                let minutes = (duration.as_secs() % 3600) / 60;
                let seconds = duration.as_secs() % 60;
                let millis = duration.subsec_millis();

                if hours > 0 {
                    format!("{hours}:{minutes:02}:{seconds:02}.{millis:03}")
                } else {
                    format!("{minutes:02}:{seconds:02}.{millis:03}")
                }
            }
            None => "N/A".to_string(),
        }
    }

    #[must_use]
    pub fn resolution_string(&self) -> String {
        self.video_stream()
            .map(|v| format!("{}x{}", v.width, v.height))
            .unwrap_or_else(|| "N/A".to_string())
    }
}
