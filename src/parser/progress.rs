use crate::domain::JobProgress;
use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct ProgressParser {
    duration: Option<f64>,
}

impl ProgressParser {
    #[must_use]
    pub fn new() -> Self {
        Self { duration: None }
    }

    #[must_use]
    pub fn with_duration(duration: f64) -> Self {
        Self {
            duration: Some(duration),
        }
    }

    pub fn set_duration(&mut self, duration: f64) {
        self.duration = Some(duration);
    }

    fn extract_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
        let start = line.find(key)? + key.len();
        let rest = &line[start..];

        let end = rest
            .find(|c: char| c.is_whitespace() && c != ' ')
            .or_else(|| rest.find("  "))
            .unwrap_or(rest.len());

        Some(rest[..end].trim())
    }

    fn parse_duration_line(line: &str) -> Option<f64> {
        static DURATION_RE: OnceLock<Regex> = OnceLock::new();
        let re = DURATION_RE
            .get_or_init(|| Regex::new(r"Duration:\s*(\d{2}):(\d{2}):(\d{2})\.(\d{2})").unwrap());

        re.captures(line).map(|caps| {
            let hours: f64 = caps[1].parse().unwrap_or(0.0);
            let minutes: f64 = caps[2].parse().unwrap_or(0.0);
            let seconds: f64 = caps[3].parse().unwrap_or(0.0);
            let centis: f64 = caps[4].parse().unwrap_or(0.0);

            hours * 3600.0 + minutes * 60.0 + seconds + centis / 100.0
        })
    }

    fn parse_time(time_str: &str) -> Option<f64> {
        if time_str.starts_with("-") || time_str == "N/A" {
            return None;
        }

        let parts: Vec<&str> = time_str.split(":").collect();
        if parts.len() != 3 {
            return None;
        }

        let hours: f64 = parts[0].parse().ok()?;
        let minutes: f64 = parts[1].parse().ok()?;
        let seconds: f64 = parts[2].parse().ok()?;

        Some(hours * 3600.0 + minutes * 60.0 + seconds)
    }

    fn parse_size(size_str: &str) -> Option<u64> {
        let size_str = size_str.trim();
        if size_str == "N/A" {
            return None;
        }

        if let Some(kb_str) = size_str.strip_suffix("kB") {
            let kb: f64 = kb_str.trim().parse().ok()?;
            return Some((kb * 1024.0) as u64);
        }

        if let Some(mb_str) = size_str.strip_suffix("MB") {
            let mb: f64 = mb_str.trim().parse().ok()?;
            return Some((mb * 1024.0 * 1024.0) as u64);
        }

        size_str.parse().ok()
    }

    #[must_use]
    pub fn parse_line(&mut self, line: &str) -> Option<JobProgress> {
        if let Some(duration) = Self::parse_duration_line(line) {
            self.duration = Some(duration);
            return None;
        }

        if !line.contains("frame=") && !line.contains("time=") {
            return None;
        }

        let frame = Self::extract_value(line, "frame=")
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);

        let fps = Self::extract_value(line, "fps=")
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0.0);

        let bitrate = Self::extract_value(line, "bitrate=").and_then(|s| {
            let s = s.trim().trim_end_matches("kbits/s");
            s.parse().ok()
        });

        let size = Self::extract_value(line, "size=")
            .and_then(|s| Self::parse_size(s.trim()))
            .unwrap_or(0);

        let time_encoded = Self::extract_value(line, "time=")
            .and_then(|s| Self::parse_time(s.trim()))
            .unwrap_or(0.0);

        let speed = Self::extract_value(line, "speed=")
            .and_then(|s| {
                let s = s.trim().trim_end_matches('x');
                s.parse().ok()
            })
            .unwrap_or(0.0);

        let percentage = if let Some(duration) = self.duration {
            if duration > 0.0 {
                (time_encoded / duration * 100.0).min(100.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        Some(JobProgress {
            frame,
            fps,
            bitrate,
            total_size: size,
            time_encoded,
            speed,
            percentage,
        })
    }
}

impl Default for ProgressParser {
    fn default() -> Self {
        Self::new()
    }
}
