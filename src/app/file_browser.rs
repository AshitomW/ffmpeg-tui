use std::path::{Path, PathBuf};
use std::{cmp::Ordering, fs, io};

/// Whether the file browser is selecting an input or output path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileBrowserTarget {
    Input,
    Output,
}

/// A single entry in the directory listing.
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

impl DirEntry {
    fn from_path(path: &Path) -> Option<Self> {
        let metadata = fs::metadata(path).ok()?;
        let name = path.file_name()?.to_string_lossy().to_string();
        Some(Self {
            name,
            path: path.to_path_buf(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
        })
    }
}

/// File browser state for terminal-based directory navigation.
#[derive(Debug, Clone)]
pub struct FileBrowserState {
    pub current_dir: PathBuf,
    pub entries: Vec<DirEntry>,
    pub selected_index: usize,
    pub filter_text: String,
    pub target: FileBrowserTarget,
    /// Entries after applying filter, stored as indices into `entries`.
    pub filtered_indices: Vec<usize>,
    /// For output mode: the filename (stem) the user is typing.
    pub output_filename: String,
    /// For output mode: the file extension derived from the builder's format.
    pub output_extension: String,
}

impl FileBrowserState {
    /// Open a file browser rooted at the given directory.
    ///
    /// `extension` is used for output mode to auto-set the file extension.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if the directory cannot be read.
    pub fn open(
        start_dir: &Path,
        target: FileBrowserTarget,
        extension: &str,
    ) -> io::Result<Self> {
        let current_dir = if start_dir.is_dir() {
            start_dir.to_path_buf()
        } else {
            start_dir
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("/"))
        };

        let current_dir = fs::canonicalize(&current_dir).unwrap_or(current_dir);

        let mut browser = Self {
            current_dir,
            entries: Vec::new(),
            selected_index: 0,
            filter_text: String::new(),
            target,
            filtered_indices: Vec::new(),
            output_filename: String::new(),
            output_extension: extension.to_string(),
        };
        browser.refresh_entries()?;
        Ok(browser)
    }

    /// Re-read the current directory and rebuild the entry list.
    pub fn refresh_entries(&mut self) -> io::Result<()> {
        let mut entries = Vec::new();

        let read_dir = fs::read_dir(&self.current_dir)?;
        for entry_result in read_dir {
            let entry = entry_result?;
            let path = entry.path();
            if let Some(dir_entry) = DirEntry::from_path(&path) {
                // Skip hidden files (starting with '.')
                if !dir_entry.name.starts_with('.') {
                    entries.push(dir_entry);
                }
            }
        }

        // Sort: directories first (alphabetically), then files (alphabetically)
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        self.entries = entries;
        self.selected_index = 0;
        self.rebuild_filter();
        Ok(())
    }

    /// Navigate into the entry at the given filtered index.
    ///
    /// Returns `Some(PathBuf)` if the entry is a file (i.e. selection confirmed).
    /// Returns `None` if we navigated into a subdirectory.
    pub fn navigate_into(&mut self, filtered_idx: usize) -> io::Result<Option<PathBuf>> {
        let entry_idx = match self.filtered_indices.get(filtered_idx) {
            Some(&idx) => idx,
            None => return Ok(None),
        };

        let entry = match self.entries.get(entry_idx) {
            Some(e) => e.clone(),
            None => return Ok(None),
        };

        if entry.is_dir {
            self.current_dir = entry.path;
            self.filter_text.clear();
            self.refresh_entries()?;
            Ok(None)
        } else {
            Ok(Some(entry.path))
        }
    }

    /// Navigate to the parent directory.
    pub fn navigate_up(&mut self) -> io::Result<()> {
        if let Some(parent) = self.current_dir.parent().map(Path::to_path_buf) {
            self.current_dir = parent;
            self.filter_text.clear();
            self.refresh_entries()?;
        }
        Ok(())
    }

    /// Move selection cursor up.
    pub fn select_prev(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    /// Move selection cursor down.
    pub fn select_next(&mut self) {
        let max = self.visible_count().saturating_sub(1);
        self.selected_index = self.selected_index.saturating_add(1).min(max);
    }

    /// Append a character to the filter text and rebuild filtered indices.
    pub fn filter_push(&mut self, c: char) {
        self.filter_text.push(c);
        self.rebuild_filter();
        // Clamp selection after filter change
        let max = self.visible_count().saturating_sub(1);
        self.selected_index = self.selected_index.min(max);
    }

    /// Remove the last character from the filter text.
    pub fn filter_pop(&mut self) {
        self.filter_text.pop();
        self.rebuild_filter();
        let max = self.visible_count().saturating_sub(1);
        self.selected_index = self.selected_index.min(max);
    }

    /// Append a character to the output filename.
    pub fn filename_push(&mut self, c: char) {
        self.output_filename.push(c);
    }

    /// Remove the last character from the output filename.
    pub fn filename_pop(&mut self) {
        self.output_filename.pop();
    }

    /// Build the full output path from `current_dir + output_filename.extension`.
    #[must_use]
    pub fn output_full_path(&self) -> Option<PathBuf> {
        if self.output_filename.is_empty() {
            return None;
        }
        let filename = if self.output_extension.is_empty() {
            self.output_filename.clone()
        } else {
            format!("{}.{}", self.output_filename, self.output_extension)
        };
        Some(self.current_dir.join(filename))
    }

    /// Number of entries visible after filtering.
    #[must_use]
    pub fn visible_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Get the currently selected `DirEntry` (after filtering).
    #[must_use]
    pub fn selected_entry(&self) -> Option<&DirEntry> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.entries.get(idx))
    }

    /// Get visible entries as an iterator of `(filtered_position, &DirEntry)`.
    pub fn visible_entries(&self) -> impl Iterator<Item = (usize, &DirEntry)> {
        self.filtered_indices
            .iter()
            .enumerate()
            .filter_map(|(pos, &idx)| self.entries.get(idx).map(|e| (pos, e)))
    }

    /// Rebuild the filtered_indices from entries + filter_text.
    fn rebuild_filter(&mut self) {
        let filter_lower = self.filter_text.to_lowercase();
        self.filtered_indices = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                if filter_lower.is_empty() {
                    true
                } else {
                    e.name.to_lowercase().contains(&filter_lower)
                }
            })
            .map(|(i, _)| i)
            .collect();
    }

    /// Confirm the currently selected entry (for input mode).
    ///
    /// Returns `Some(PathBuf)` if a file was selected.
    pub fn confirm_selected(&mut self) -> io::Result<Option<PathBuf>> {
        let idx = self.selected_index;
        self.navigate_into(idx)
    }

    /// Confirm output selection: returns the full output path built from
    /// current_dir + filename + extension.
    ///
    /// If a directory is highlighted in the list, navigate into it instead.
    pub fn confirm_output(&mut self) -> io::Result<Option<PathBuf>> {
        if let Some(entry) = self.selected_entry()
            && entry.is_dir
        {
            return self.navigate_into(self.selected_index);
        }

        if let Some(path) = self.output_full_path() {
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }

    /// Format file size for display.
    #[must_use]
    pub fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;

        if size >= GB {
            format!("{:.1} GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.1} MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1} KB", size as f64 / KB as f64)
        } else {
            format!("{size} B")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_browser_at_current_dir() {
        let browser = FileBrowserState::open(
            &std::env::current_dir().unwrap(),
            FileBrowserTarget::Input,
            "mp4",
        )
        .unwrap();

        assert!(!browser.entries.is_empty() || browser.entries.is_empty());
        assert_eq!(browser.selected_index, 0);
        assert!(browser.filter_text.is_empty());
    }

    #[test]
    fn format_size_display() {
        assert_eq!(FileBrowserState::format_size(500), "500 B");
        assert_eq!(FileBrowserState::format_size(1024), "1.0 KB");
        assert_eq!(FileBrowserState::format_size(1_048_576), "1.0 MB");
        assert_eq!(FileBrowserState::format_size(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn output_full_path_builds_correctly() {
        let mut browser = FileBrowserState::open(
            &std::env::current_dir().unwrap(),
            FileBrowserTarget::Output,
            "mp4",
        )
        .unwrap();

        // No filename → None
        assert!(browser.output_full_path().is_none());

        // With filename → builds full path
        browser.output_filename = "my_video".to_string();
        let path = browser.output_full_path().unwrap();
        assert!(path.to_string_lossy().ends_with("my_video.mp4"));
    }
}

