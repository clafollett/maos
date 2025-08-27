use super::config::RollingLogConfig;
use crate::{MaosError, Result, SessionId};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Logger for session-specific log files
pub struct SessionLogger {
    session_id: SessionId,
    log_file: File,
    log_path: PathBuf,
    current_size: u64,
    config: RollingLogConfig,
    rotation_count: usize,
}

impl SessionLogger {
    /// Create a new session logger
    pub fn new(session_id: SessionId, log_dir: PathBuf, config: RollingLogConfig) -> Result<Self> {
        // Ensure log directory exists
        fs::create_dir_all(&log_dir).map_err(|e| {
            MaosError::Io(std::io::Error::other(format!(
                "Failed to create log directory {}: {}",
                log_dir.display(),
                e
            )))
        })?;

        // Create log file path
        let file_name = config
            .file_pattern
            .replace("{session_id}", session_id.as_str());
        let log_path = log_dir.join(file_name);

        // Open or create the log file
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| {
                MaosError::Io(std::io::Error::other(format!(
                    "Failed to open log file {}: {}",
                    log_path.display(),
                    e
                )))
            })?;

        // Get initial file size
        let current_size = log_file.metadata().map(|m| m.len()).unwrap_or(0);

        Ok(Self {
            session_id,
            log_file,
            log_path,
            current_size,
            config,
            rotation_count: 0,
        })
    }

    /// Write a log entry
    pub fn write(&mut self, message: &str) -> Result<()> {
        let entry = format!("{message}\n");
        let entry_bytes = entry.as_bytes();

        // Check if rotation is needed
        if self.current_size + entry_bytes.len() as u64 > self.config.max_file_size_bytes as u64 {
            self.rotate()?;
        }

        // Write to file
        self.log_file.write_all(entry_bytes).map_err(|e| {
            MaosError::Io(std::io::Error::other(format!(
                "Failed to write to log file {}: {}",
                self.log_path.display(),
                e
            )))
        })?;

        self.log_file.flush().map_err(|e| {
            MaosError::Io(std::io::Error::other(format!(
                "Failed to flush log file {}: {}",
                self.log_path.display(),
                e
            )))
        })?;

        self.current_size += entry_bytes.len() as u64;
        Ok(())
    }

    /// Rotate the log file
    fn rotate(&mut self) -> Result<()> {
        self.rotation_count += 1;

        // Close current file
        self.log_file.sync_all().ok();

        // Rotate existing files
        for i in (1..self.config.max_files_per_session).rev() {
            let old_path = self.rotated_path(i);
            let new_path = self.rotated_path(i + 1);
            if old_path.exists() {
                if i < self.config.max_files_per_session {
                    let _ = fs::rename(&old_path, &new_path);
                } else {
                    let _ = fs::remove_file(&old_path);
                }
            }
        }

        // Rotate current file to .1
        if self.config.compress_on_roll {
            // When compression is enabled, rotate to uncompressed path first
            let uncompressed_path = self.log_path.with_extension("log.1");
            let _ = fs::rename(&self.log_path, &uncompressed_path);

            // Compress the rotated file (creates .gz and removes original)
            let _ = self.compress_file(&uncompressed_path);
        } else {
            // Direct rotation without compression
            let rotated_path = self.rotated_path(1);
            let _ = fs::rename(&self.log_path, &rotated_path);
        }

        // Create new log file
        self.log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| {
                MaosError::Io(std::io::Error::other(format!(
                    "Failed to create new log file after rotation {}: {}",
                    self.log_path.display(),
                    e
                )))
            })?;

        self.current_size = 0;
        Ok(())
    }

    /// Get the path for a rotated file
    fn rotated_path(&self, index: usize) -> PathBuf {
        let mut path = self.log_path.clone();
        let extension = if self.config.compress_on_roll {
            format!("{index}.gz")
        } else {
            format!("{index}")
        };
        path.set_extension(format!("log.{extension}"));
        path
    }

    /// Compress a file using gzip
    fn compress_file(&self, path: &Path) -> Result<()> {
        use flate2::Compression;
        use flate2::write::GzEncoder;

        let input = fs::read(path).map_err(|e| {
            MaosError::Io(std::io::Error::other(format!(
                "Failed to read file for compression: {e}"
            )))
        })?;

        // Add .gz extension without replacing existing extension
        let compressed_path = PathBuf::from(format!("{}.gz", path.display()));
        let output = File::create(&compressed_path).map_err(|e| {
            MaosError::Io(std::io::Error::other(format!(
                "Failed to create compressed file: {e}"
            )))
        })?;

        let mut encoder = GzEncoder::new(output, Compression::default());
        encoder.write_all(&input).map_err(|e| {
            MaosError::Io(std::io::Error::other(format!(
                "Failed to compress file: {e}"
            )))
        })?;

        encoder.finish().map_err(|e| {
            MaosError::Io(std::io::Error::other(format!(
                "Failed to finish compression: {e}"
            )))
        })?;

        // Remove original file
        let _ = fs::remove_file(path);

        Ok(())
    }

    /// Get the session ID
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Convert to thread-safe logger
    pub fn into_thread_safe(self) -> ThreadSafeSessionLogger {
        ThreadSafeSessionLogger {
            inner: Arc::new(Mutex::new(self)),
        }
    }
}

/// Thread-safe wrapper around SessionLogger
pub struct ThreadSafeSessionLogger {
    inner: Arc<Mutex<SessionLogger>>,
}

impl ThreadSafeSessionLogger {
    /// Write a log entry
    pub fn write(&self, message: &str) -> Result<()> {
        let mut logger = self.inner.lock().map_err(|e| MaosError::Context {
            message: format!("Failed to lock session logger: {e}"),
            source: Box::new(std::io::Error::other("mutex poisoned")),
        })?;
        logger.write(message)
    }

    /// Get the session ID
    pub fn session_id(&self) -> SessionId {
        let logger = self.inner.lock().unwrap();
        logger.session_id.clone()
    }
}

#[cfg(test)]
mod tests {
    // NOTE: All filesystem-dependent tests have been moved to
    // crates/maos-core/tests/integration/logging_integration_tests.rs
    // Unit tests should NEVER touch the filesystem - that's for integration tests

    // TODO: Add proper unit tests that don't use filesystem
    // Examples: test configuration validation, test data structures, mock filesystem, etc.
}
