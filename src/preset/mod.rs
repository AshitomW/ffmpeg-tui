mod builtin;

pub use builtin::{Preset, PresetCategory, builtin_presets};

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PresetError {
    #[error("Failed to read preset file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse preset: {0}")]
    ParseError(String),

    #[error("Preset not found: {0}")]
    NotFound(String),
}

pub fn load_user_presets(dir: &Path) -> Result<Vec<Preset>, PresetError> {
    let mut presets = Vec::new();

    if !dir.exists() {
        return Ok(presets);
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "toml") {
            let content = std::fs::read_to_string(&path)?;
            let preset: Preset =
                toml::from_str(&content).map_err(|e| PresetError::ParseError(e.to_string()))?;
            presets.push(preset);
        }
    }
    Ok(presets)
}

pub fn save_preset(preset: &Preset, path: &Path) -> Result<(), PresetError> {
    let content =
        toml::to_string_pretty(preset).map_err(|e| PresetError::ParseError(e.to_string()))?;

    std::fs::write(path, content)?;
    Ok(())
}
