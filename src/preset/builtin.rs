use crate::domain::{AudioCodec, ContainerFormat, EncodingPreset, FilterChain, VideoCodec};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PresetCategory {
    Compression,
    Conversion,
    AudioExtraction,
    Streaming,
    Social,
    Archive,
    Custom,
}
impl PresetCategory {
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Compression => "Compression",
            Self::Conversion => "Conversion",
            Self::AudioExtraction => "Audio Extraction",
            Self::Streaming => "Streaming",
            Self::Social => "Social Media",
            Self::Archive => "Archive",
            Self::Custom => "Custom",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub description: String,
    pub category: PresetCategory,
    pub video_codec: VideoCodec,
    pub audio_codec: AudioCodec,
    pub container: ContainerFormat,
    pub crf: Option<u8>,
    pub encoding_preset: Option<EncodingPreset>,
    pub video_bitrate: Option<u32>,
    pub audio_bitrate: Option<u32>,
    pub filters: FilterChain,
    pub extra_args: Vec<String>,
}

impl Preset {
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            category: PresetCategory::Custom,
            video_codec: VideoCodec::default(),
            audio_codec: AudioCodec::default(),
            container: ContainerFormat::default(),
            crf: Some(23),
            encoding_preset: Some(EncodingPreset::Medium),
            video_bitrate: None,
            audio_bitrate: Some(192),
            filters: FilterChain::new(),
            extra_args: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_category(mut self, category: PresetCategory) -> Self {
        self.category = category;
        self
    }

    #[must_use]
    pub fn with_video_codec(mut self, codec: VideoCodec) -> Self {
        self.video_codec = codec;
        self
    }

    #[must_use]
    pub fn with_audio_codec(mut self, codec: AudioCodec) -> Self {
        self.audio_codec = codec;
        self
    }

    #[must_use]
    pub fn with_container(mut self, container: ContainerFormat) -> Self {
        self.container = container;
        self
    }

    #[must_use]
    pub fn with_crf(mut self, crf: u8) -> Self {
        self.crf = Some(crf);
        self.video_bitrate = None;
        self
    }

    #[must_use]
    pub fn with_encoding_preset(mut self, preset: EncodingPreset) -> Self {
        self.encoding_preset = Some(preset);
        self
    }

    #[must_use]
    pub fn with_audio_bitrate(mut self, bitrate: u32) -> Self {
        self.audio_bitrate = Some(bitrate);
        self
    }
}

#[must_use]
pub fn builtin_presets() -> Vec<Preset> {
    // These presets are what i commonly use, so i added them directly in code.
    vec![
        Preset::new(
            "High quality compress",
            "High quality H.265 compression good with file size reduction",
        )
        .with_category(PresetCategory::Compression)
        .with_video_codec(VideoCodec::H265)
        .with_audio_codec(AudioCodec::Aac)
        .with_container(ContainerFormat::Mp4)
        .with_crf(23)
        .with_encoding_preset(EncodingPreset::Slow)
        .with_audio_bitrate(192),
        Preset::new("Fast compress", "Quick compression with H.264")
            .with_category(PresetCategory::Compression)
            .with_video_codec(VideoCodec::H264)
            .with_audio_codec(AudioCodec::Aac)
            .with_container(ContainerFormat::Mp4)
            .with_crf(26)
            .with_encoding_preset(EncodingPreset::Fast)
            .with_audio_bitrate(128),
        Preset::new("Max Compression", "Agressive compression with H.265")
            .with_category(PresetCategory::Compression)
            .with_video_codec(VideoCodec::H265)
            .with_audio_codec(AudioCodec::Opus)
            .with_container(ContainerFormat::Mkv)
            .with_crf(30)
            .with_encoding_preset(EncodingPreset::Slow)
            .with_audio_bitrate(96),
        // Conversion presets
        Preset::new("WebM VP9", "Convert to WebM format with VP9 and Opus")
            .with_category(PresetCategory::Conversion)
            .with_video_codec(VideoCodec::Vp9)
            .with_audio_codec(AudioCodec::Opus)
            .with_container(ContainerFormat::Webm)
            .with_crf(31)
            .with_audio_bitrate(128),
        Preset::new("ProRes 422", "Convert to ProRes for editing")
            .with_category(PresetCategory::Conversion)
            .with_video_codec(VideoCodec::ProRes)
            .with_audio_codec(AudioCodec::Pcm)
            .with_container(ContainerFormat::Mov),
        // Audio Extraction
        Preset::new("Extract MP3", "Extract audio to MP3 Format")
            .with_category(PresetCategory::AudioExtraction)
            .with_video_codec(VideoCodec::None)
            .with_audio_codec(AudioCodec::Mp3)
            .with_container(ContainerFormat::Mp3)
            .with_audio_bitrate(320),
        Preset::new("Extract FLAC", "Extract audio to FLAC")
            .with_category(PresetCategory::AudioExtraction)
            .with_video_codec(VideoCodec::None)
            .with_audio_codec(AudioCodec::Flac)
            .with_container(ContainerFormat::Flac),
        Preset::new("Extract Opus", "Extract audio to Opus format")
            .with_category(PresetCategory::AudioExtraction)
            .with_video_codec(VideoCodec::None)
            .with_audio_codec(AudioCodec::Opus)
            .with_container(ContainerFormat::Ogg)
            .with_audio_bitrate(192),
        // Generic presets , that i dont use , just added becuase i could
        Preset::new("Youtube Upload", "For Youtube Uploads")
            .with_category(PresetCategory::Streaming)
            .with_video_codec(VideoCodec::H264)
            .with_audio_codec(AudioCodec::Aac)
            .with_container(ContainerFormat::Mp4)
            .with_crf(18)
            .with_encoding_preset(EncodingPreset::Slow)
            .with_audio_bitrate(320),
        Preset::new("Twitch Stream", "For Twitch Streaming")
            .with_category(PresetCategory::Social)
            .with_video_codec(VideoCodec::H264)
            .with_audio_codec(AudioCodec::Aac)
            .with_container(ContainerFormat::Mp4)
            .with_crf(23)
            .with_encoding_preset(EncodingPreset::Medium)
            .with_audio_bitrate(128),
        Preset::new("Reels", "For Reels")
            .with_category(PresetCategory::Social)
            .with_video_codec(VideoCodec::H264)
            .with_audio_codec(AudioCodec::Aac)
            .with_container(ContainerFormat::Mp4)
            .with_crf(20)
            .with_encoding_preset(EncodingPreset::Medium)
            .with_audio_bitrate(192),
        // Archives
        Preset::new("Kind of lossless archive", "Lossless copy for archival")
            .with_category(PresetCategory::Archive)
            .with_video_codec(VideoCodec::Copy)
            .with_audio_codec(AudioCodec::Copy)
            .with_container(ContainerFormat::Mkv),
        Preset::new(
            "High Quality Archive",
            "Very HQ H.265 for long term storage",
        )
        .with_category(PresetCategory::Archive)
        .with_video_codec(VideoCodec::H265)
        .with_audio_codec(AudioCodec::Flac)
        .with_container(ContainerFormat::Mkv)
        .with_crf(16)
        .with_encoding_preset(EncodingPreset::VerySlow),
    ]
}
