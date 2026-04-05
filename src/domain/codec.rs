//! Codec defintions for audio and video

use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]

/// Video codec options
pub enum VideoCodec {
    #[default]
    H264,
    H265,
    Vp9,
    Av1,
    ProRes,
    Mpeg4,
    Copy,
    None,
}

impl VideoCodec {
    /// Returns the codec name
    #[must_use]
    pub const fn ffmpeg_name(&self) -> &'static str {
        match self {
            Self::H264 => "libx264",
            Self::H265 => "libx265",
            Self::Vp9 => "libvpx-vp9",
            Self::Av1 => "libaom-av1",
            Self::ProRes => "prores_ks",
            Self::Mpeg4 => "mpeg4",
            Self::Copy => "copy",
            Self::None => "none",
        }
    }

    /// Return available ones
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::H264,
            Self::H265,
            Self::Vp9,
            Self::Av1,
            Self::ProRes,
            Self::Mpeg4,
            Self::Copy,
        ]
    }

    /// Return whether codec supprots CRF Quality
    #[must_use]
    pub const fn supports_crf(&self) -> bool {
        matches!(self, Self::H264 | Self::H265 | Self::Vp9 | Self::Av1)
    }
}

impl fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::H264 => "H.264 (AVC)",
            Self::H265 => "H.265 (HVEC)",
            Self::Vp9 => "VP9",
            Self::Av1 => "AV1",
            Self::ProRes => "ProRes",
            Self::Mpeg4 => "MPEG-4",
            Self::Copy => "Copy (No Re encode)",
            Self::None => "No Video",
        };
        write!(f, "{name}")
    }
}

/// Audio Codec options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AudioCodec {
    #[default]
    Aac,
    Mp3,
    Opus,
    Vorbis,
    Flac,
    Pcm,
    Ac3,
    Copy,
    None,
}

impl AudioCodec {
    /// Return the codec name
    #[must_use]
    pub const fn ffmpeg_name(&self) -> &'static str {
        match self {
            Self::Aac => "aac",
            Self::Mp3 => "libmp3lame",
            Self::Opus => "libopus",
            Self::Vorbis => "libvorbis",
            Self::Flac => "flac",
            Self::Pcm => "pcm_s16le",
            Self::Ac3 => "ac3",
            Self::Copy => "copy",
            Self::None => "none",
        }
    }

    /// Return all available codecs
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Aac,
            Self::Mp3,
            Self::Opus,
            Self::Vorbis,
            Self::Flac,
            Self::Pcm,
            Self::Ac3,
            Self::Copy,
        ]
    }

    /// return default bitrate for codec in kbps
    #[must_use]
    pub const fn default_bitrate(&self) -> u32 {
        match self {
            Self::Aac | Self::Mp3 => 192,
            Self::Opus => 128,
            Self::Vorbis => 160,
            Self::Ac3 => 384,
            Self::Flac | Self::Pcm | Self::Copy | Self::None => 0,
        }
    }
}

impl fmt::Display for AudioCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Aac => "AAC",
            Self::Mp3 => "MP3",
            Self::Opus => "Opus",
            Self::Vorbis => "Vorbis",
            Self::Flac => "Flac",
            Self::Pcm => "PCM",
            Self::Ac3 => "AC3",
            Self::Copy => "Copy (NO Re encode)",
            Self::None => "No audio",
        };

        write!(f, "{name}")
    }
}

// Simple Tests
#[cfg(test)]
mod tests {
    use crate::domain::codec::{AudioCodec, VideoCodec};

    #[test]
    fn video_codec_ffmpeg_names() {
        assert_eq!(VideoCodec::H264.ffmpeg_name(), "libx264");
        assert_eq!(VideoCodec::H265.ffmpeg_name(), "libx265");
        assert_eq!(VideoCodec::Copy.ffmpeg_name(), "copy")
    }

    #[test]
    fn audio_codec_ffmpeg_names() {
        assert_eq!(AudioCodec::Aac.ffmpeg_name(), "aac");
        assert_eq!(AudioCodec::Mp3.ffmpeg_name(), "libmp3lame");
        assert_eq!(AudioCodec::Copy.ffmpeg_name(), "copy");
    }

    #[test]
    fn crf_support() {
        assert!(VideoCodec::H264.supports_crf());
        assert!(VideoCodec::H265.supports_crf());
        assert!(!VideoCodec::Copy.supports_crf());
        assert!(!VideoCodec::ProRes.supports_crf());
    }
}
