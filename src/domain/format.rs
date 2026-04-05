//! Container format defns

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContainerFormat {
    #[default]
    Mp4,
    Mkv,
    Webm,
    Mov,
    Avi,
    Flv,
    Ts,
    Mp3,
    Flac,
    Wav,
    Ogg,
}

impl ContainerFormat {
    /// Return extension for the format
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
            Self::Webm => "webm",
            Self::Mov => "mov",
            Self::Avi => "avi",
            Self::Flv => "flv",
            Self::Ts => "ts",
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
            Self::Wav => "wav",
            Self::Ogg => "ogg",
        }
    }

    /// return format name
    #[must_use]
    pub const fn ffmpeg_name(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "matroska",
            Self::Webm => "webm",
            Self::Mov => "mov",
            Self::Avi => "avi",
            Self::Flv => "flv",
            Self::Ts => "mpegts",
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
            Self::Wav => "wav",
            Self::Ogg => "ogg",
        }
    }

    #[must_use]
    pub const fn video_formats() -> &'static [Self] {
        &[
            Self::Mp4,
            Self::Mkv,
            Self::Webm,
            Self::Mov,
            Self::Avi,
            Self::Flv,
            Self::Ts,
        ]
    }

    #[must_use]
    pub const fn audio_formats() -> &'static [Self] {
        &[Self::Mp3, Self::Flac, Self::Wav, Self::Ogg]
    }

    /// Check if format has video support
    #[must_use]
    pub const fn supports_video(&self) -> bool {
        matches!(
            self,
            Self::Mp4 | Self::Mkv | Self::Webm | Self::Mov | Self::Avi | Self::Flv | Self::Ts
        )
    }

    /// Try to determine format from extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext = ext.to_lowercase();
        match ext.as_str() {
            "mp4" | "m4v" => Some(Self::Mp4),
            "mkv" => Some(Self::Mkv),
            "webm" => Some(Self::Webm),
            "mov" => Some(Self::Mov),
            "avi" => Some(Self::Avi),
            "flv" => Some(Self::Flv),
            "ts" | "mts" => Some(Self::Ts),
            "mp3" => Some(Self::Mp3),
            "flac" => Some(Self::Flac),
            "wav" => Some(Self::Wav),
            "ogg" => Some(Self::Ogg),
            _ => None,
        }
    }
}

impl fmt::Display for ContainerFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Mp4 => "MP4",
            Self::Mp3 => "MP3",
            Self::Mkv => "Matroska (MKV)",
            Self::Webm => "WebM",
            Self::Mov => "Quicktime (MOV)",
            Self::Avi => "AVI",
            Self::Flv => "Flash Video (FLV)",
            Self::Ts => "MPEG-TS",
            Self::Wav => "WAV",
            Self::Ogg => "Ogg",
            Self::Flac => "FLAC",
        };
        write!(f, "{name}")
    }
}

/// Encoding speed preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EncodingPreset {
    UltraFast,
    SuperFast,
    VeryFast,
    Faster,
    Fast,
    #[default]
    Medium,
    Slow,
    Slower,
    VerySlow,
    Placebo,
}

impl EncodingPreset {
    #[must_use]
    pub const fn ffmpeg_name(&self) -> &'static str {
        match self {
            Self::UltraFast => "ultrafast",
            Self::SuperFast => "superfast",
            Self::VeryFast => "veryfast",
            Self::Faster => "faster",
            Self::Fast => "fast",
            Self::Medium => "medium",
            Self::Slow => "slow",
            Self::Slower => "slower",
            Self::VerySlow => "veryslow",
            Self::Placebo => "placebo",
        }
    }

    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::UltraFast,
            Self::SuperFast,
            Self::VeryFast,
            Self::Faster,
            Self::Fast,
            Self::Medium,
            Self::Slow,
            Self::Slower,
            Self::VerySlow,
        ]
    }
}

impl fmt::Display for EncodingPreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ffmpeg_name())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormat {
    pub container: ContainerFormat,
    pub video_bitrate: Option<u32>,
    pub audio_bitrate: Option<u32>,
    pub crf: Option<u8>,
    pub preset: Option<EncodingPreset>,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self {
            container: ContainerFormat::Mp4,
            video_bitrate: None,
            audio_bitrate: None,
            crf: Some(23),
            preset: Some(EncodingPreset::Medium),
        }
    }
}

// Tests

#[cfg(test)]
mod tests {
    use crate::domain::ContainerFormat;

    #[test]
    fn format_extensions() {
        assert_eq!(ContainerFormat::Mp4.extension(), "mp4");
        assert_eq!(ContainerFormat::Mkv.extension(), "mkv");
    }

    #[test]
    fn format_from_extension() {
        assert_eq!(
            ContainerFormat::from_extension("mp4"),
            Some(ContainerFormat::Mp4)
        );
        assert_eq!(
            ContainerFormat::from_extension("MP4"),
            Some(ContainerFormat::Mp4)
        );
        assert_eq!(ContainerFormat::from_extension("xyz"), None);
    }

    #[test]
    fn video_support() {
        assert!(ContainerFormat::Mp4.supports_video());
        assert!(!ContainerFormat::Mp3.supports_video());
    }
}
