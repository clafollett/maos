//! Session-based logging with automatic rotation and compression

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use flate2::Compression;
use flate2::write::GzEncoder;
use parking_lot::RwLock;
use tracing::debug;

use super::config::RollingLogConfig;
use crate::{Result, SessionId};

/// Session logger that writes to rotating log files
pub struct SessionLogger {
    session_id: SessionId,
    log_dir: PathBuf,
    config: RollingLogConfig,
    current_file: Option<BufWriter<File>>,
    current_file_size: usize,
}

/// Thread-safe wrapper around SessionLogger
pub struct ThreadSafeSessionLogger {
    inner: Arc<RwLock<SessionLogger>>,
}

impl SessionLogger {
    /// Create a new session logger
    pub fn new(session_id: SessionId, log_dir: PathBuf, config: RollingLogConfig) -> Result<Self> {
        // Create log directory if it doesn't exist
        if !log_dir.exists() {
            fs::create_dir_all(&log_dir)?;
        }

        let mut logger = Self {
            session_id,
            log_dir,
            config,
            current_file: None,
            current_file_size: 0,
        };

        // Open the initial log file
        logger.open_current_file()?;

        Ok(logger)
    }

    /// Get the session ID
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Write a log entry
    pub fn write(&mut self, entry: &str) -> Result<()> {
        let entry_with_newline = format!("{}\n", entry);
        let entry_bytes = entry_with_newline.len();

        // Check if we need to rotate
        if self.current_file_size + entry_bytes > self.config.max_file_size_bytes {
            self.rotate_log_file()?;
        }

        // Write to current file
        if let Some(ref mut file) = self.current_file {
            file.write_all(entry_with_newline.as_bytes())?;
            file.flush()?;
            self.current_file_size += entry_bytes;
        }

        Ok(())
    }

    /// Convert to thread-safe variant
    pub fn into_thread_safe(self) -> ThreadSafeSessionLogger {
        ThreadSafeSessionLogger {
            inner: Arc::new(RwLock::new(self)),
        }
    }

    /// Open the current log file
    fn open_current_file(&mut self) -> Result<()> {
        let log_path = self.current_log_path();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        // Get current file size
        self.current_file_size = file.metadata()?.len() as usize;

        self.current_file = Some(BufWriter::new(file));

        Ok(())
    }

    /// Get the path to the current log file
    fn current_log_path(&self) -> PathBuf {
        let filename = self
            .config
            .file_pattern
            .replace("{session_id}", self.session_id.as_str());
        self.log_dir.join(filename)
    }

    /// Rotate the current log file atomically to prevent data loss
    fn rotate_log_file(&mut self) -> Result<()> {
        // Close current file
        if let Some(mut file) = self.current_file.take() {
            file.flush()?;
        }

        let current_path = self.current_log_path();

        // Shift existing rotated files
        self.shift_rotated_files()?;

        // Move current file to .1 atomically
        if self.config.compress_on_roll {
            let rotated_path = current_path.with_extension("log.1.gz");
            let temp_path = current_path.with_extension("log.1.gz.tmp");

            // Compress to temporary file first
            self.compress_file(&current_path, &temp_path)?;

            // Atomic rename of compressed file
            fs::rename(&temp_path, &rotated_path).or_else(|_| -> Result<()> {
                // Fallback: copy and remove if rename fails (cross-filesystem)
                fs::copy(&temp_path, &rotated_path)?;
                fs::remove_file(&temp_path)?;
                Ok(())
            })?;

            // Remove original only after successful compression
            fs::remove_file(&current_path)?;
        } else {
            let rotated_path = current_path.with_extension("log.1");
            fs::rename(&current_path, &rotated_path)?;
        };

        // Clean up old files beyond max limit
        self.cleanup_old_files()?;

        // Reset current file size and open new file
        self.current_file_size = 0;
        self.open_current_file()?;

        Ok(())
    }

    /// Shift existing rotated files (rename .1 to .2, .2 to .3, etc.)
    fn shift_rotated_files(&self) -> Result<()> {
        for i in (1..self.config.max_files_per_session).rev() {
            let current_num = i;
            let next_num = i + 1;

            let extensions = if self.config.compress_on_roll {
                [
                    format!("log.{}.gz", current_num),
                    format!("log.{}.gz", next_num),
                ]
            } else {
                [format!("log.{}", current_num), format!("log.{}", next_num)]
            };

            let current_path = self.current_log_path().with_extension(&extensions[0]);
            let next_path = self.current_log_path().with_extension(&extensions[1]);

            if current_path.exists() {
                fs::rename(current_path, next_path)?;
            }
        }

        Ok(())
    }

    /// Compress a file using gzip
    fn compress_file(&self, source: &Path, dest: &Path) -> Result<()> {
        let input_data = fs::read(source)?;
        let output_file = File::create(dest)?;
        let mut encoder = GzEncoder::new(output_file, Compression::default());
        encoder.write_all(&input_data)?;
        encoder.finish()?;
        Ok(())
    }

    /// Clean up files beyond the maximum limit
    fn cleanup_old_files(&self) -> Result<()> {
        let max_rotated_file_num = self.config.max_files_per_session;

        // Try to remove files beyond the limit
        for i in (max_rotated_file_num + 1)..=(max_rotated_file_num * 2) {
            let extensions = if self.config.compress_on_roll {
                format!("log.{}.gz", i)
            } else {
                format!("log.{}", i)
            };

            let old_file = self.current_log_path().with_extension(&extensions);

            if old_file.exists()
                && let Err(e) = fs::remove_file(&old_file)
            {
                debug!("Failed to remove old log file {old_file:?}: {e}");
            }
        }

        Ok(())
    }
}

impl ThreadSafeSessionLogger {
    /// Write a log entry (thread-safe)
    pub fn write(&self, entry: &str) -> Result<()> {
        let mut logger = self.inner.write();
        logger.write(entry)
    }

    /// Get the session ID (thread-safe)
    pub fn session_id(&self) -> SessionId {
        let logger = self.inner.read();
        logger.session_id.clone()
    }
}
