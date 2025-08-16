//! High-performance stdin processor for Claude Code hooks

use bytes::BytesMut;
use maos_core::{MaosError, Result, config::HookConfig};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tokio::io::{AsyncReadExt, stdin};

/// High-performance JSON input processor for stdin
///
/// Provides async stdin reading with timeout protection, buffer reuse,
/// and security hardening for Claude Code hook inputs.
///
/// # Example
///
/// ```ignore
/// use maos::io::StdinProcessor;
/// use maos::io::HookInput;
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
            buffer: BytesMut::with_capacity(8192), // 8KB initial capacity
            read_buffer: vec![0u8; 8192],          // 🔥 EFFICIENCY FIX: Pre-allocated read buffer
            config,
        }
    }

    /// Create processor with default configuration
    pub fn with_defaults() -> Self {
        Self::new(HookConfig::default())
    }

    /// Get the maximum allowed input size in bytes
    pub fn max_size(&self) -> usize {
        (self.config.max_input_size_mb * 1024 * 1024) as usize
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
    pub fn validate_size(&self, size: usize) -> Result<()> {
        let max_size = self.max_size();
        if size > max_size {
            return Err(MaosError::InvalidInput {
                message: format!("Input size {} exceeds maximum {}", size, max_size),
            });
        }
        Ok(())
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

        // Parse JSON with remaining time budget
        let elapsed = start_time.elapsed().as_millis() as u64;
        if elapsed >= processing_timeout_ms {
            return Err(MaosError::Timeout {
                operation: "json_parsing".to_string(),
                timeout_ms: processing_timeout_ms,
            });
        }

        serde_json::from_slice(input).map_err(MaosError::Json)
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
    pub fn validate_json_depth_static(input: &[u8], max_depth: u32) -> Result<()> {
        let mut depth = 0u32;
        let mut max_seen = 0u32;
        let mut in_string = false;
        let mut escape_next = false;

        for &byte in input {
            if escape_next {
                escape_next = false;
                continue;
            }

            match byte {
                b'"' if !escape_next => in_string = !in_string,
                b'\\' if in_string => escape_next = true,
                b'{' | b'[' if !in_string => {
                    depth += 1;
                    max_seen = max_seen.max(depth);
                    if depth > max_depth {
                        return Err(MaosError::InvalidInput {
                            message: format!(
                                "JSON nesting depth {} exceeds maximum {}",
                                depth, max_depth
                            ),
                        });
                    }
                }
                b'}' | b']' if !in_string => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        Ok(())
    }
}

impl Default for StdinProcessor {
    fn default() -> Self {
        Self::with_defaults()
    }
}
