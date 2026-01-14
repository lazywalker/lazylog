use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::{RotationPeriod, RotationTrigger};

/// State of the current log file.
#[derive(Debug)]
pub struct FileState {
    /// The open file handle.
    pub file: File,
    /// Current size of the file in bytes.
    pub size: u64,
    /// Time suffix for the current file (empty for size-only rotation).
    pub time_suffix: String,
}

/// A writer that rotates log files based on size and/or time.
pub struct RotatingWriter {
    /// Base path for log files.
    base_path: PathBuf,
    /// Rotation trigger configuration.
    trigger: RotationTrigger,
    /// Current file state, protected by mutex.
    state: Arc<Mutex<Option<FileState>>>,
}

impl RotatingWriter {
    /// Create a new rotating writer.
    pub fn new(base_path: &std::path::Path, trigger: RotationTrigger) -> io::Result<Self> {
        let writer = Self {
            base_path: base_path.to_path_buf(),
            trigger,
            state: Arc::new(Mutex::new(None)),
        };

        // Ensure parent directory exists (create if necessary). This makes
        // file logging robust when users specify a path containing directories
        // that don't yet exist (e.g., `logs/lazydns.log`).
        if let Some(parent) = writer.base_path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }

        // Initialize with a file
        writer.get_or_rotate(0)?;

        Ok(writer)
    }

    /// Get the current time suffix based on the rotation period.
    fn current_time_suffix(&self) -> String {
        match &self.trigger {
            RotationTrigger::Never => String::new(),
            RotationTrigger::Time { period } => period.get_suffix(),
            RotationTrigger::Size { .. } => String::new(),
            RotationTrigger::Both { period, .. } => period.get_suffix(),
        }
    }

    /// Determine the actual file path for the current rotation.
    fn current_file_path(&self) -> PathBuf {
        let suffix = self.current_time_suffix();
        if suffix.is_empty() {
            self.base_path.clone()
        } else {
            // Time-based: append suffix like .2026-01-09
            let path_str = self.base_path.to_string_lossy();
            PathBuf::from(format!("{}.{}", path_str, suffix))
        }
    }

    /// Check if rotation is needed based on current state and buffer size.
    fn needs_rotation(&self, state: &FileState, buf_len: usize) -> bool {
        match &self.trigger {
            RotationTrigger::Never => false,
            RotationTrigger::Time { period } => {
                if *period == RotationPeriod::Never {
                    return false;
                }
                let current_suffix = period.get_suffix();
                current_suffix != state.time_suffix
            }
            RotationTrigger::Size { max_size, .. } => state.size + buf_len as u64 > *max_size,
            RotationTrigger::Both {
                period, max_size, ..
            } => {
                let time_trigger = if *period != RotationPeriod::Never {
                    period.get_suffix() != state.time_suffix
                } else {
                    false
                };
                let size_trigger = state.size + buf_len as u64 > *max_size;
                time_trigger || size_trigger
            }
        }
    }

    /// Check if the base file exists and is within size limits
    fn should_use_existing_file(&self) -> io::Result<bool> {
        if !self.base_path.exists() {
            return Ok(false);
        }

        match &self.trigger {
            RotationTrigger::Size { max_size, .. } | RotationTrigger::Both { max_size, .. } => {
                let metadata = self.base_path.metadata()?;
                let file_size = metadata.len();
                Ok(file_size <= *max_size)
            }
            _ => Ok(false), // For time-only or never rotation, don't use existing file
        }
    }

    /// Perform size-based rotation by copying content instead of renaming.
    ///
    /// Copies content: base.log -> base.log.1, then truncates base.log to 0
    /// This preserves the main log file for continuous monitoring (e.g., tail -f)
    fn rotate_by_size(&self) -> io::Result<()> {
        // Rotate the *current* file (which may include a time suffix) rather than
        // the base path. This ensures hybrid (Both) rotation behaves sensibly —
        // size-based rotations will operate on the active file (e.g. `base.2026-01-15`)
        // instead of an unrelated `base` path.
        let max_files = self.trigger.max_files().unwrap_or(5);
        let current = self.current_file_path();

        // Delete the oldest file if it exists (current.<max_files>)
        let oldest = PathBuf::from(format!("{}.{}", current.display(), max_files));
        if oldest.exists() {
            std::fs::remove_file(&oldest)?;
        }

        // Shift files: current.(N-1) -> current.N, ..., current.1 -> current.2
        for i in (1..max_files).rev() {
            let from = PathBuf::from(format!("{}.{}", current.display(), i));
            let to = PathBuf::from(format!("{}.{}", current.display(), i + 1));
            if from.exists() {
                std::fs::rename(&from, &to)?;
            }
        }

        // Copy current file content to current.1 and truncate the current file
        if current.exists() {
            let first = PathBuf::from(format!("{}.1", current.display()));
            std::fs::copy(&current, &first)?;

            // Truncate the original current file to 0 bytes
            let file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&current)?;
            file.set_len(0)?;
        }

        Ok(())
    }

    /// Perform rotation and create a new file.
    fn rotate(&self) -> io::Result<FileState> {
        // For size-based rotation, perform the rename chain
        if self.trigger.has_size_rotation() {
            // Use base_path for size rotation (not time-suffixed path)
            self.rotate_by_size()?;
        }

        // Open/create the new file
        let file_path = self.current_file_path();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;

        let size = file.metadata().map(|m| m.len()).unwrap_or(0);

        Ok(FileState {
            file,
            size,
            time_suffix: self.current_time_suffix(),
        })
    }

    /// Get or create the current file, rotating if necessary.
    fn get_or_rotate(&self, buf_len: usize) -> io::Result<Arc<Mutex<Option<FileState>>>> {
        let mut guard = self.state.lock().unwrap();

        let needs_rotation = match &*guard {
            None => {
                // First time initialization - check if we can use existing file
                !self.should_use_existing_file()?
            }
            Some(state) => self.needs_rotation(state, buf_len),
        };

        if needs_rotation {
            // Close current file (drop it)
            *guard = None;

            // Perform rotation and create new file
            let new_state = self.rotate()?;
            *guard = Some(new_state);
        } else if guard.is_none() {
            // No rotation needed and no current state - open existing file
            let file_path = self.current_file_path();
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)?;

            let size = file.metadata().map(|m| m.len()).unwrap_or(0);
            let time_suffix = self.current_time_suffix();

            *guard = Some(FileState {
                file,
                size,
                time_suffix,
            });
        }

        Ok(Arc::clone(&self.state))
    }
}

impl Write for RotatingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let state_arc = self.get_or_rotate(buf.len())?;
        let mut guard = state_arc.lock().unwrap();

        if let Some(state) = guard.as_mut() {
            let written = state.file.write(buf)?;
            state.size += written as u64;
            Ok(written)
        } else {
            Err(io::Error::other("Failed to open log file"))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let guard = self.state.lock().unwrap();
        if let Some(state) = guard.as_ref() {
            // We need interior mutability for flush, so we use a trick:
            // File::flush takes &mut self, but we can sync_all() on &File
            state.file.sync_all()
        } else {
            Ok(())
        }
    }
}

// Implement Send for use with non_blocking
unsafe impl Send for RotatingWriter {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let unique = format!(
            "{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        std::env::temp_dir().join(format!("lazydns_log_test_{}_{}", prefix, unique))
    }

    fn cleanup_dir(dir: &PathBuf) {
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_rotating_writer_creates_file() {
        let dir = unique_test_dir("create");
        std::fs::create_dir_all(&dir).unwrap();

        let log_path = dir.join("test.log");
        let mut writer =
            RotatingWriter::new(&log_path, RotationTrigger::Never).expect("create writer");

        writer.write_all(b"hello world\n").unwrap();
        writer.flush().unwrap();

        assert!(log_path.exists());
        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("hello world"));

        cleanup_dir(&dir);
    }

    #[test]
    fn test_rotating_writer_creates_parent_dir() {
        // Don't pre-create nested dirs; writer should create them automatically
        let dir = unique_test_dir("parent_create");
        let nested = dir.join("nested/inner");
        let log_path = nested.join("test.log");

        // Parent does not exist yet
        assert!(!nested.exists());

        // Creating writer should create parent directories
        let mut writer =
            RotatingWriter::new(&log_path, RotationTrigger::Never).expect("create writer");

        writer.write_all(b"hello parent\n").unwrap();
        writer.flush().unwrap();

        assert!(log_path.exists(), "Log file should have been created");
        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("hello parent"));

        // Parent directories should exist now
        assert!(
            nested.exists(),
            "Parent directories should have been created"
        );

        cleanup_dir(&dir);
    }

    #[test]
    fn test_rotating_writer_size_rotation() {
        let dir = unique_test_dir("size");
        std::fs::create_dir_all(&dir).unwrap();

        let log_path = dir.join("test.log");
        // Very small max_size to trigger rotation quickly
        let mut writer =
            RotatingWriter::new(&log_path, RotationTrigger::size(50, 3)).expect("create writer");

        // Write enough to trigger rotation
        for i in 0..5 {
            writer
                .write_all(format!("line {} - some padding text here\n", i).as_bytes())
                .unwrap();
        }
        writer.flush().unwrap();

        // Check that rotated files exist
        assert!(log_path.exists(), "base log file should exist");
        let rotated_1 = dir.join("test.log.1");
        assert!(rotated_1.exists(), "test.log.1 should exist");

        cleanup_dir(&dir);
    }

    #[cfg(feature = "time")]
    #[test]
    fn test_rotating_writer_time_suffix() {
        let dir = unique_test_dir("time");
        std::fs::create_dir_all(&dir).unwrap();

        let log_path = dir.join("test.log");
        let mut writer = RotatingWriter::new(
            &log_path,
            RotationTrigger::Time {
                period: RotationPeriod::Daily,
            },
        )
        .expect("create writer");

        writer.write_all(b"hello\n").unwrap();
        writer.flush().unwrap();

        // Find the time-suffixed file
        let entries: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert!(!entries.is_empty(), "should have created a log file");

        // Check that filename has date suffix
        let first = &entries[0];
        let name = first.file_name().to_string_lossy().to_string();
        assert!(
            name.starts_with("test.log."),
            "filename should have date suffix"
        );
        assert!(name.contains('-'), "filename should contain date separator");

        cleanup_dir(&dir);
    }

    #[test]
    fn test_rotating_writer_reuse_existing_file() {
        let dir = unique_test_dir("reuse");
        std::fs::create_dir_all(&dir).unwrap();

        let log_path = dir.join("test.log");

        // Create a file with some content that's under the size limit
        {
            let mut file = std::fs::File::create(&log_path).unwrap();
            file.write_all(b"existing content\n").unwrap();
        }

        // Create writer with size limit larger than existing content
        let mut writer =
            RotatingWriter::new(&log_path, RotationTrigger::size(100, 5)).expect("create writer");

        // Write additional content
        writer.write_all(b"new content\n").unwrap();
        writer.flush().unwrap();

        // Check that the file still contains both old and new content
        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("existing content"));
        assert!(content.contains("new content"));

        // Check that no rotated files were created
        let rotated_1 = dir.join("test.log.1");
        assert!(!rotated_1.exists(), "Should not have created rotated file");

        cleanup_dir(&dir);
    }

    #[cfg(feature = "time")]
    #[test]
    fn test_rotating_writer_hybrid() {
        let dir = unique_test_dir("hybrid");
        std::fs::create_dir_all(&dir).unwrap();

        let log_path = dir.join("test.log");
        let mut writer = RotatingWriter::new(
            &log_path,
            RotationTrigger::both(RotationPeriod::Daily, 50, 3),
        )
        .expect("create writer");

        // Write enough to trigger size rotation
        for i in 0..5 {
            writer
                .write_all(format!("line {} padding\n", i).as_bytes())
                .unwrap();
        }
        writer.flush().unwrap();

        // Should have created files
        let entries: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert!(!entries.is_empty(), "should have created log files");

        // Find the active time-suffixed file (e.g., test.log.2026-01-15)
        // Choose the suffixed name whose last segment contains a '-' (date),
        // which distinguishes it from rotated numeric suffixes like `.1`.
        let active = entries
            .iter()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .find(|name| {
                name.starts_with("test.log.")
                    && name
                        .rsplit('.')
                        .next()
                        .map(|s| s.contains('-'))
                        .unwrap_or(false)
            })
            .expect("active time-suffixed file should exist");

        let rotated_name = format!("{}.1", active);
        let rotated_path = dir.join(&rotated_name);
        assert!(
            rotated_path.exists(),
            "rotated file {} should exist",
            rotated_name
        );

        cleanup_dir(&dir);
    }
}
