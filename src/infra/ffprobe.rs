use crate::domain::{
    AudioStream, DataStream, FormatInfo, MediaInfo, StreamInfo, SubtitleStream, VideoStream,
};
use crate::infra::ProcessError;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use tokio::process::Command;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct FFProbeExecutor {
    binary_path: String,
}
impl FFProbeExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            binary_path: "ffprobe".to_string(),
        }
    }

    #[must_use]
    pub fn with_binary(binary_path: impl Into<String>) -> Self {
        Self {
            binary_path: binary_path.into(),
        }
    }

    pub async fn inspect(&self, path: &Path) -> Result<MediaInfo, ProcessError> {
        let output = Command::new(&self.binary_path)
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProcessError::FFMpegError(format!(
                "FFprobe failed: {}",
                stderr
            )));
        }

        let json_output = String::from_utf8_lossy(&output.stdout);
        debug!("FFprobe output: {}", json_output);

        let raw: RawProbeOutput = serde_json::from_str(&json_output)
            .map_err(|e| ProcessError::ParseError(format!("Failed to parse JSON: {e}")))?;

        Ok(self.convert_probe_output(path.to_path_buf(), raw))
    }

    fn convert_probe_output(&self, path: std::path::PathBuf, raw: RawProbeOutput) -> MediaInfo {
        let format = FormatInfo {
            format_name: raw.format.format_name,
            format_long_name: raw.format.format_long_name,
            duration: raw.format.duration.and_then(|s| s.parse().ok()),
            size: raw.format.size.and_then(|s| s.parse().ok()),
            bit_rate: raw.format.bit_rate.and_then(|s| s.parse().ok()),
            tags: raw.format.tags.unwrap_or_default(),
        };

        let streams = raw
            .streams
            .into_iter()
            .map(|s| self.convert_stream(s))
            .collect();

        MediaInfo {
            path,
            format,
            streams,
        }
    }

    fn convert_stream(&self, raw: RawStream) -> StreamInfo {
        match raw.codec_type.as_str() {
            "video" => StreamInfo::Video(VideoStream {
                index: raw.index,
                codec_name: raw.codec_name,
                codec_long_name: raw.codec_long_name,
                profile: raw.profile,
                width: raw.width.unwrap_or(0),
                height: raw.height.unwrap_or(0),
                coded_width: raw.coded_width,
                coded_height: raw.coded_height,
                display_aspect_ratio: raw.display_aspect_ratio,
                sample_aspect_ratio: raw.sample_aspect_ratio,
                pix_fmt: raw.pix_fmt,
                frame_rate: parse_frame_rate(&raw.r_frame_rate),
                avg_frame_rate: parse_frame_rate(&raw.avg_frame_rate),
                bit_rate: raw.bit_rate.and_then(|s| s.parse().ok()),
                bits_per_raw_sample: raw.bits_per_raw_sample.and_then(|s| s.parse().ok()),
                color_space: raw.color_space,
                color_range: raw.color_range,
                color_transfer: raw.color_transfer,
                color_primaries: raw.color_primaries,
                tags: raw.tags.unwrap_or_default(),
            }),
            "audio" => StreamInfo::Audio(AudioStream {
                index: raw.index,
                codec_name: raw.codec_name,
                codec_long_name: raw.codec_long_name,
                profile: raw.profile,
                sample_rate: raw.sample_rate.and_then(|s| s.parse().ok()),
                channels: raw.channels,
                channel_layout: raw.channel_layout,
                sample_fmt: raw.sample_fmt,
                bit_rate: raw.bit_rate.and_then(|s| s.parse().ok()),
                bits_per_sample: raw.bits_per_sample,
                tags: raw.tags.unwrap_or_default(),
            }),
            "subtitle" => StreamInfo::Subtitle(SubtitleStream {
                index: raw.index,
                codec_name: raw.codec_name,
                codec_long_name: raw.codec_long_name,
                language: raw.tags.as_ref().and_then(|t| t.get("language").cloned()),
                tags: raw.tags.unwrap_or_default(),
            }),
            _ => StreamInfo::Data(DataStream {
                index: raw.index,
                codec_name: raw.codec_name,
                tags: raw.tags.unwrap_or_default(),
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawProbeOutput {
    format: RawFormat,
    streams: Vec<RawStream>,
}

#[derive(Debug, Deserialize)]
struct RawFormat {
    format_name: String,
    format_long_name: Option<String>,
    duration: Option<String>,
    size: Option<String>,
    bit_rate: Option<String>,
    tags: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct RawStream {
    index: u32,
    codec_name: String,
    codec_long_name: Option<String>,
    codec_type: String,
    profile: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    coded_width: Option<u32>,
    coded_height: Option<u32>,
    display_aspect_ratio: Option<String>,
    sample_aspect_ratio: Option<String>,
    pix_fmt: Option<String>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    bit_rate: Option<String>,
    bits_per_raw_sample: Option<String>,
    color_space: Option<String>,
    color_range: Option<String>,
    color_transfer: Option<String>,
    color_primaries: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
    channel_layout: Option<String>,
    sample_fmt: Option<String>,
    bits_per_sample: Option<u32>,
    tags: Option<HashMap<String, String>>,
}

fn parse_frame_rate(rate_str: &Option<String>) -> Option<f64> {
    rate_str.as_ref().and_then(|s| {
        if let Some((num, den)) = s.split_once('/') {
            let num: f64 = num.parse().ok()?;
            let den: f64 = den.parse().ok()?;
            if den > 0.0 { Some(num / den) } else { None }
        } else {
            s.parse().ok()
        }
    })
}

impl Default for FFProbeExecutor {
    fn default() -> Self {
        Self::new()
    }
}
