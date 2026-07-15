use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

use crate::domain::{AudioCodec, ContainerFormat, FilterChain, VideoCodec, format::EncodingPreset};

#[derive(Debug, Error)]
pub enum CommandBuildError {
    #[error("No input file specified")]
    NoInput,
    #[error("No output file specified")]
    NoOutput,
    #[error("Invalid input path: {0}")]
    InvalidInputPath(String),
    #[error("Invalid output path: {0}")]
    InvalidOutputPath(String),
    #[error("Incompatible codec {codec} with format {format}")]
    IncompatibleCodec { codec: String, format: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    path: PathBuf,
    seek_start: Option<f64>,
    duration: Option<f64>,
    stream_loop: Option<i32>,
    input_options: Vec<String>,
}
impl InputConfig {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(seek) = self.seek_start {
            args.push("-ss".to_string());
            args.push(format!("{seek}"));
        }

        if let Some(duration) = self.duration {
            args.push("-t".to_string());
            args.push(format!("{duration}"));
        }

        if let Some(loops) = self.stream_loop {
            args.push("-stream_loop".to_string());
            args.push(loops.to_string());
        }

        args.extend(self.input_options.clone());
        args.push("-i".to_string());
        args.push(self.path.to_string_lossy().to_string());

        args
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    path: PathBuf,
    format: ContainerFormat,
    video_codec: VideoCodec,
    audio_codec: AudioCodec,
    video_bitrate: Option<u32>,
    audio_bitrate: Option<u32>,
    crf: Option<u8>,
    preset: Option<EncodingPreset>,
    output_options: Vec<String>,
}

impl OutputConfig {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        match self.video_codec {
            VideoCodec::None => {
                args.push("-vh".to_string());
            }
            _ => {
                args.push("-c:v".to_string());
                args.push(self.video_codec.ffmpeg_name().to_string());
            }
        }

        match self.audio_codec {
            AudioCodec::None => {
                args.push("-an".to_string());
            }
            _ => {
                args.push("-c:a".to_string());
                args.push(self.audio_codec.ffmpeg_name().to_string());
            }
        }

        if let Some(crf) = self.crf.filter(|_| self.video_codec.supports_crf()) {
            args.push("-crf".to_string());
            args.push(crf.to_string());
        }

        if let Some(preset) = self
            .preset
            .as_ref()
            .filter(|_| matches!(self.video_codec, VideoCodec::H264 | VideoCodec::H265))
        {
            args.push("-preset".to_string());
            args.push(preset.ffmpeg_name().to_string());
        }

        if let Some(vb) = self.video_bitrate {
            args.push("-b:v".to_string());
            args.push(format!("{vb}k"));
        }

        if let Some(ab) = self.audio_bitrate {
            args.push("-b:a".to_string());
            args.push(format!("{ab}k"));
        }

        args.push("-f".to_string());
        args.push(self.format.ffmpeg_name().to_string());

        args.extend(self.output_options.clone());

        args.push("-y".to_string());
        args.push(self.path.to_string_lossy().to_string());

        args
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HardwareAccel {
    Auto,
    Cuda,
    Vaapi,
    Qsv,
    VideoToolbox,
}

impl HardwareAccel {
    const fn ffmpeg_name(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Cuda => "cuda",
            Self::Vaapi => "vaapi",
            Self::Qsv => "qsv",
            Self::VideoToolbox => "videotoolbox",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LogLevel {
    Quiet,
    Panic,
    Fatal,
    Error,
    Warning,
    #[default]
    Info,
    Verbose,
    Debug,
}

impl LogLevel {
    const fn ffmpeg_name(&self) -> &'static str {
        match self {
            Self::Quiet => "quiet",
            Self::Panic => "panic",
            Self::Fatal => "fatal",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Verbose => "verbose",
            Self::Debug => "debug",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalOptions {
    hide_banner: bool,
    overwrite: bool,
    threads: Option<u32>,
    hwaccel: Option<HardwareAccel>,
    loglevel: Option<LogLevel>,
}

impl GlobalOptions {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.hide_banner {
            args.push("-hide_banner".to_string());
        }

        if let Some(threads) = self.threads {
            args.push("-threads".to_string());
            args.push(threads.to_string());
        }

        if let Some(hwaccel) = &self.hwaccel {
            args.push("-hwaccel".to_string());
            args.push(hwaccel.ffmpeg_name().to_string());
        }

        if let Some(loglevel) = &self.loglevel {
            args.push("-loglevel".to_string());
            args.push(loglevel.ffmpeg_name().to_string());
        }

        args
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFMpegCommand {
    inputs: Vec<InputConfig>,
    output: OutputConfig,
    global_options: GlobalOptions,
    filters: FilterChain,
    raw_args: Vec<String>,
}

impl FFMpegCommand {
    #[must_use]
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        args.extend(self.global_options.to_args());

        for input in &self.inputs {
            args.extend(input.to_args());
        }

        if let Some(vf) = self.filters.video_filter_string() {
            args.push("-vf".to_string());
            args.push(vf);
        }

        if let Some(af) = self.filters.audio_filter_string() {
            args.push("-af".to_string());
            args.push(af);
        }

        args.extend(self.output.to_args());
        args.extend(self.raw_args.clone());

        args
    }

    #[must_use]
    pub fn to_command_string(&self) -> String {
        let args = self.to_args();
        let formatted: Vec<String> = args
            .into_iter()
            .map(|arg| {
                if arg.contains(' ') || arg.contains(';') {
                    format!("\"{arg}\"")
                } else {
                    arg
                }
            })
            .collect();
        format!("ffmpeg {}", formatted.join(" "))
    }

    #[must_use]
    pub fn input_path(&self) -> Option<&PathBuf> {
        self.inputs.first().map(|i| &i.path)
    }

    #[must_use]
    pub fn output_path(&self) -> &PathBuf {
        &self.output.path
    }

    #[must_use]
    pub fn filters(&self) -> &FilterChain {
        &self.filters
    }

    pub fn filters_mut(&mut self) -> &mut FilterChain {
        &mut self.filters
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommandBuilder {
    inputs: Vec<InputConfig>,
    output: Option<OutputConfig>,
    global_options: GlobalOptions,
    filters: FilterChain,
    raw_args: Vec<String>,
}

impl CommandBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            global_options: GlobalOptions {
                hide_banner: true,
                overwrite: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[must_use]
    pub fn input(mut self, path: PathBuf) -> Self {
        self.inputs.push(InputConfig {
            path,
            seek_start: None,
            duration: None,
            stream_loop: None,
            input_options: Vec::new(),
        });
        self
    }

    #[must_use]
    pub fn input_with_range(
        mut self,
        path: PathBuf,
        seek: Option<f64>,
        duration: Option<f64>,
    ) -> Self {
        self.inputs.push(InputConfig {
            path,
            seek_start: seek,
            duration,
            stream_loop: None,
            input_options: Vec::new(),
        });

        self
    }

    #[must_use]
    pub fn output(mut self, path: PathBuf) -> Self {
        let format = path
            .extension()
            .and_then(|e| e.to_str())
            .and_then(ContainerFormat::from_extension)
            .unwrap_or_default();

        self.output = Some(OutputConfig {
            path,
            format,
            video_codec: VideoCodec::default(),
            audio_codec: AudioCodec::default(),
            video_bitrate: None,
            audio_bitrate: Some(192),
            crf: Some(23),
            preset: Some(EncodingPreset::Medium),
            output_options: Vec::new(),
        });

        self
    }

    #[must_use]
    pub fn video_codec(mut self, codec: VideoCodec) -> Self {
        if let Some(output) = &mut self.output {
            output.video_codec = codec;
        }
        self
    }
    #[must_use]
    pub fn audio_codec(mut self, codec: AudioCodec) -> Self {
        if let Some(output) = &mut self.output {
            output.audio_codec = codec;
        }
        self
    }

    #[must_use]
    pub fn format(mut self, format: ContainerFormat) -> Self {
        if let Some(output) = &mut self.output {
            output.format = format;
        }
        self
    }

    #[must_use]
    pub fn crf(mut self, crf: u8) -> Self {
        if let Some(output) = &mut self.output {
            output.crf = Some(crf);
        }

        self
    }

    #[must_use]
    pub fn preset(mut self, preset: EncodingPreset) -> Self {
        if let Some(output) = &mut self.output {
            output.preset = Some(preset);
        }
        self
    }

    #[must_use]
    pub fn video_bitrate(mut self, bitrate: u32) -> Self {
        if let Some(output) = &mut self.output {
            output.video_bitrate = Some(bitrate);
            output.crf = None;
        }
        self
    }

    #[must_use]
    pub fn filters(mut self, filters: FilterChain) -> Self {
        self.filters = filters;
        self
    }
    #[must_use]
    pub fn threads(mut self, threads: u32) -> Self {
        self.global_options.threads = Some(threads);
        self
    }

    #[must_use]
    pub fn hardware_accel(mut self, accel: HardwareAccel) -> Self {
        self.global_options.hwaccel = Some(accel);
        self
    }

    #[must_use]
    pub fn raw_arg(mut self, arg: String) -> Self {
        self.raw_args.push(arg);
        self
    }

    pub fn build(self) -> Result<FFMpegCommand, CommandBuildError> {
        if self.inputs.is_empty() {
            return Err(CommandBuildError::NoInput);
        }

        let output = self.output.ok_or(CommandBuildError::NoOutput)?;

        Ok(FFMpegCommand {
            inputs: self.inputs,
            output,
            global_options: self.global_options,
            filters: self.filters,
            raw_args: self.raw_args,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_basic_command() {
        let cmd = CommandBuilder::new()
            .input(PathBuf::from("input.mp4"))
            .output(PathBuf::from("output.mp4"))
            .build()
            .unwrap();

        let args = cmd.to_args();

        println!("{:?}", args);

        assert!(args.contains(&"-i".to_string()));
        assert!(args.contains(&"input.mp4".to_string()));
        assert!(args.contains(&"output.mp4".to_string()))
    }

    #[test]
    fn build_with_codecs() {
        let cmd = CommandBuilder::new()
            .input(PathBuf::from("input.mp4"))
            .output(PathBuf::from("output.mp4"))
            .video_codec(VideoCodec::H265)
            .audio_codec(AudioCodec::Opus)
            .build()
            .unwrap();

        let args = cmd.to_args();
        println!("{:?}", args);
        assert!(args.contains(&"libx265".to_string()));
        assert!(args.contains(&"libopus".to_string()));
    }

    #[test]
    fn build_with_filters() {
        use crate::domain::{ScaleDimension, VideoFilter};

        let mut filters = FilterChain::new();
        filters.add_video_filter(VideoFilter::Scale {
            width: ScaleDimension::Exact(1920),
            height: ScaleDimension::Exact(1080),
        });

        let cmd = CommandBuilder::new()
            .input(PathBuf::from("input.mp4"))
            .output(PathBuf::from("output.mp4"))
            .filters(filters)
            .build()
            .unwrap();

        let args = cmd.to_args();
        println!("{:?}", args);
        assert!(args.contains(&"-vf".to_string()));

        assert!(args.contains(&"scale=1920:1080".to_string()));
    }
    #[test]
    fn error_on_missing_input() {
        let result = CommandBuilder::new()
            .output(PathBuf::from("output.mp4"))
            .build();

        assert!(matches!(result, Err(CommandBuildError::NoInput)));
    }

    #[test]
    fn error_on_missing_output() {
        let result = CommandBuilder::new()
            .input(PathBuf::from("input.mp4"))
            .build();

        assert!(matches!(result, Err(CommandBuildError::NoOutput)));
    }
}
