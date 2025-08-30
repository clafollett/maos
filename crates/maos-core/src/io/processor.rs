//! High-performance stdin processor for Claude Code hooks

use crate::{MaosError, Result, config::HookConfig, constants::sizes::DEFAULT_BUFFER_SIZE};
use bytes::BytesMut;
use serde::de::DeserializeOwned;
use std::time::Duration;
use tokio::io::{AsyncReadExt, stdin};
use tracing;

/// High-performance JSON input processor for stdin
///
/// Provides async stdin reading with timeout protection, buffer reuse,
/// and security hardening for Claude Code hook inputs.
///
/// # Example
///
/// ```ignore
/// use maos_core::io::StdinProcessor;
/// use maos_core::io::HookInput;
/// use maos_core::config::HookConfig;
///
/// #[tokio::main]
/// async fn main() {
///     let config = HookConfig::default();
///     let mut processor = StdinProcessor::new(config);
///     match processor.read_json::<HookInput>().await {
///         Ok(input) => println!("Received hook: {}", input.hook_event_name),
///         Err(e) => eprintln!("Error reading input: {}", e),
///     }
/// }
/// ```
pub struct StdinProcessor {
    buffer: BytesMut,
    /// 🔥 EFFICIENCY FIX: Reusable read buffer to avoid repeated allocations
    read_buffer: Vec<u8>,
    config: HookConfig,
}

impl StdinProcessor {
    /// Create a new processor with hook configuration
    pub fn new(config: HookConfig) -> Self {
        Self {
            buffer: BytesMut::with_capacity(DEFAULT_BUFFER_SIZE),
            read_buffer: vec![0u8; DEFAULT_BUFFER_SIZE],
            config,
        }
    }

    /// Create processor with default configuration
    pub fn with_defaults() -> Self {
        Self::new(HookConfig::default())
    }

    /// Get the maximum allowed input size in bytes
    pub fn max_size(&self) -> usize {
        (self.config.max_input_size_mb as usize) * crate::constants::BYTES_PER_MB
    }

    /// Get the stdin read timeout in milliseconds
    pub fn stdin_timeout_ms(&self) -> u64 {
        self.config.stdin_read_timeout_ms
    }

    /// Get the maximum processing timeout in milliseconds
    pub fn processing_timeout_ms(&self) -> u64 {
        self.config.max_processing_time_ms
    }

    /// Clear the internal buffer (for reuse)
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Get a pointer to the buffer (for testing buffer reuse)
    #[cfg(test)]
    pub fn buffer_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    /// Validate that input size is within limits
    /// 🔥 ENHANCED DoS PROTECTION: Delegates to unified resource validator
    pub fn validate_size(&self, size: usize) -> Result<()> {
        let validator = crate::security::resource_validator::ResourceValidator::from_hook_config(
            self.config.max_input_size_mb,
        );
        validator.validate_input_size(size)
    }

    /// Read and parse JSON from stdin with timeout and security validation
    pub async fn read_json<T>(&mut self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        // Capture config values before mutable borrow
        let stdin_timeout_ms = self.config.stdin_read_timeout_ms;
        let processing_timeout_ms = self.config.max_processing_time_ms;
        let max_depth = self.config.max_json_depth;

        // 🔥 CRITICAL FIX: Clean error handling (no more double unwrap anti-pattern)
        let start_time = std::time::Instant::now();
        let input = match tokio::time::timeout(
            Duration::from_millis(processing_timeout_ms),
            self.read_to_buffer_with_timeout(stdin_timeout_ms),
        )
        .await
        {
            Ok(Ok(buffer)) => buffer,
            Ok(Err(io_err)) => return Err(io_err), // Inner I/O error
            Err(_timeout) => {
                return Err(MaosError::Timeout {
                    operation: "total_processing".to_string(),
                    timeout_ms: processing_timeout_ms,
                });
            }
        };

        // Validate JSON depth before parsing
        Self::validate_json_depth_static(input, max_depth)?;

        // 🛡️ DoS Protection Layer 3: Track memory consumption during parsing
        let memory_before = Self::get_memory_usage();

        // Parse JSON with remaining time budget
        let elapsed = start_time.elapsed().as_millis() as u64;
        if elapsed >= processing_timeout_ms {
            return Err(MaosError::Timeout {
                operation: "json_parsing".to_string(),
                timeout_ms: processing_timeout_ms,
            });
        }

        // 🛡️ DoS Protection Layer 4: Parse with memory monitoring
        let result: Result<T> = serde_json::from_slice(input).map_err(MaosError::Json);

        // 🛡️ DoS Protection Layer 5: Post-parsing memory validation
        let memory_after = Self::get_memory_usage();

        // Clean handling of optional memory tracking
        match (memory_before, memory_after) {
            (Some(before), Some(after)) => {
                let memory_growth = after.saturating_sub(before);
                // Warn if parsing consumed excessive memory
                if memory_growth > crate::constants::MEMORY_WARNING_THRESHOLD {
                    tracing::warn!(
                        "High memory consumption during JSON parsing: {} bytes growth",
                        memory_growth
                    );
                }
            }
            _ => {
                // Memory tracking unavailable, skip memory growth validation
                tracing::debug!("Memory tracking unavailable for DoS protection");
            }
        }

        result
    }

    /// Read stdin into the internal buffer with timeout per operation
    async fn read_to_buffer_with_timeout(&mut self, timeout_ms: u64) -> Result<&[u8]> {
        self.buffer.clear();

        let mut stdin = stdin();
        // 🔥 EFFICIENCY FIX: Use pre-allocated reusable buffer instead of creating new vec

        loop {
            // Apply timeout to each read operation
            let n = tokio::time::timeout(
                Duration::from_millis(timeout_ms),
                stdin.read(&mut self.read_buffer),
            )
            .await
            .map_err(|_| MaosError::Timeout {
                operation: "stdin_read_operation".to_string(),
                timeout_ms,
            })?
            .map_err(MaosError::Io)?;

            if n == 0 {
                break; // EOF reached
            }

            // Check size before adding to buffer
            self.validate_size(self.buffer.len() + n)?;
            self.buffer.extend_from_slice(&self.read_buffer[..n]);
        }

        Ok(&self.buffer)
    }

    /// Read HookInput from stdin
    pub async fn read_hook_input(&mut self) -> Result<crate::io::HookInput> {
        self.read_json().await
    }

    /// Validate JSON depth to prevent JSON bomb attacks
    ///
    /// Delegates to the unified JSON security validator to avoid duplication
    pub fn validate_json_depth_static(input: &[u8], max_depth: u32) -> Result<()> {
        // Use a very large size limit since we're only checking depth here
        crate::security::json::validate_json_structure(
            input,
            max_depth,
            usize::MAX, // Only depth matters for this function
        )
    }

    /// Get current memory usage for DoS protection monitoring
    /// 🛡️ DoS Protection: Track memory consumption to detect attacks
    /// Made public for testing the memory tracking functionality
    ///
    /// Returns `Some(bytes)` if memory tracking is available on this platform,
    /// `None` if memory tracking is unavailable (allows tests to skip accordingly)
    pub fn get_memory_usage() -> Option<usize> {
        // ✅ PROPER IMPLEMENTATION: OS-specific memory tracking with Linux optimization
        #[cfg(target_os = "linux")]
        {
            // On Linux, read from /proc/self/status for more accurate tracking
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines().filter(|line| line.starts_with("VmRSS:")) {
                    if let Some(kb_str) = line.split_whitespace().nth(1)
                        && let Ok(kb) = kb_str.parse::<usize>()
                    {
                        return Some(kb * 1024); // Convert KB to bytes
                    }
                }
            }
        }

        // ✅ MACOS IMPLEMENTATION: Use ps command for memory tracking
        #[cfg(target_os = "macos")]
        {
            // macOS: Read RSS (Resident Set Size) via ps command
            if let Ok(output) = std::process::Command::new("ps")
                .args(["-o", "rss=", "-p"])
                .arg(std::process::id().to_string())
                .output()
                && let Ok(rss_str) = String::from_utf8(output.stdout)
                && let Ok(rss_kb) = rss_str.trim().parse::<usize>()
            {
                return Some(rss_kb * 1024); // Convert KB to bytes
            }
        }

        // ✅ WINDOWS IMPLEMENTATION: Use Windows API for memory tracking
        #[cfg(windows)]
        {
            use winapi::um::processthreadsapi::GetCurrentProcess;
            use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};

            unsafe {
                let mut counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
                let result = GetProcessMemoryInfo(
                    GetCurrentProcess(),
                    &mut counters,
                    std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                );
                if result != 0 {
                    return Some(counters.WorkingSetSize);
                }
            }
        }

        // ✅ FALLBACK: Honest unavailability for other platforms
        // Return None to clearly indicate memory tracking is not available
        // DoS protection can rely on other metrics (JSON size, execution time)
        None
    }
}

impl Default for StdinProcessor {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    // Test module placeholder - tests are in separate test files
}
