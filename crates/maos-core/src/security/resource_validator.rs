//! Unified resource validation for DoS protection
//!
//! This module consolidates ALL resource validation logic including
//! memory, size, and execution time limits.

use crate::constants::BYTES_PER_MB;
use crate::{MaosError, Result};

/// Types of resources that can be validated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Memory,
    InputSize,
    ExecutionTime,
    JsonDepth,
    FileCount,
}

/// Resource usage statistics
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    pub memory_bytes: Option<usize>,
    pub input_size: Option<usize>,
    pub execution_time_ms: Option<u64>,
    pub json_depth: Option<u32>,
    pub file_count: Option<usize>,
}

/// Configuration for resource limits
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_bytes: usize,
    pub max_input_size: usize,
    pub max_execution_time_ms: u64,
    pub max_json_depth: u32,
    pub max_file_count: usize,
    pub warning_threshold_percent: usize, // Percentage of limit to trigger warning
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 100 * BYTES_PER_MB, // 100MB
            max_input_size: 10 * BYTES_PER_MB,    // 10MB
            max_execution_time_ms: 5000,          // 5 seconds
            max_json_depth: 64,
            max_file_count: 1000,
            warning_threshold_percent: 50,
        }
    }
}

/// Unified resource validator
pub struct ResourceValidator {
    limits: ResourceLimits,
}

impl ResourceValidator {
    /// Create a new resource validator with specified limits
    pub fn new(limits: ResourceLimits) -> Self {
        Self { limits }
    }

    /// Create a validator with default limits
    pub fn with_defaults() -> Self {
        Self::new(ResourceLimits::default())
    }

    /// Validate all resources in one call
    pub fn validate_all_resources(&self, usage: &ResourceUsage) -> Result<()> {
        if let Some(memory) = usage.memory_bytes {
            self.validate_memory(memory)?;
        }

        if let Some(size) = usage.input_size {
            self.validate_input_size(size)?;
        }

        if let Some(time) = usage.execution_time_ms {
            self.validate_execution_time(time)?;
        }

        if let Some(depth) = usage.json_depth {
            self.validate_json_depth(depth)?;
        }

        if let Some(count) = usage.file_count {
            self.validate_file_count(count)?;
        }

        Ok(())
    }

    /// Validate memory usage
    pub fn validate_memory(&self, memory_bytes: usize) -> Result<()> {
        if memory_bytes > self.limits.max_memory_bytes {
            return Err(MaosError::ResourceLimit {
                resource: "memory".to_string(),
                limit: self.limits.max_memory_bytes as u64,
                actual: memory_bytes as u64,
                message: format!(
                    "Memory usage {} bytes exceeds limit {} bytes",
                    memory_bytes, self.limits.max_memory_bytes
                ),
            });
        }

        // Warn if approaching limit
        self.check_warning_threshold(
            ResourceType::Memory,
            memory_bytes,
            self.limits.max_memory_bytes,
        );

        Ok(())
    }

    /// Validate input size (with enhanced DoS protection)
    pub fn validate_input_size(&self, size: usize) -> Result<()> {
        // Hard size limit
        if size > self.limits.max_input_size {
            return Err(MaosError::InvalidInput {
                message: "Input exceeds maximum allowed size for security".to_string(),
            });
        }

        // Warning for suspicious sizes
        self.check_warning_threshold(ResourceType::InputSize, size, self.limits.max_input_size);

        Ok(())
    }

    /// Validate execution time
    pub fn validate_execution_time(&self, time_ms: u64) -> Result<()> {
        if time_ms > self.limits.max_execution_time_ms {
            return Err(MaosError::ResourceLimit {
                resource: "execution_time".to_string(),
                limit: self.limits.max_execution_time_ms,
                actual: time_ms,
                message: format!(
                    "Execution time {}ms exceeds limit {}ms",
                    time_ms, self.limits.max_execution_time_ms
                ),
            });
        }

        Ok(())
    }

    /// Validate JSON depth
    pub fn validate_json_depth(&self, depth: u32) -> Result<()> {
        if depth > self.limits.max_json_depth {
            return Err(MaosError::InvalidInput {
                message: format!(
                    "JSON nesting depth {} exceeds maximum {}",
                    depth, self.limits.max_json_depth
                ),
            });
        }

        Ok(())
    }

    /// Validate file count
    pub fn validate_file_count(&self, count: usize) -> Result<()> {
        if count > self.limits.max_file_count {
            return Err(MaosError::ResourceLimit {
                resource: "file_count".to_string(),
                limit: self.limits.max_file_count as u64,
                actual: count as u64,
                message: format!(
                    "File count {} exceeds limit {}",
                    count, self.limits.max_file_count
                ),
            });
        }

        Ok(())
    }

    /// Check if a resource usage is approaching its limit and log warning
    fn check_warning_threshold(&self, resource: ResourceType, actual: usize, limit: usize) {
        let threshold = (limit * self.limits.warning_threshold_percent) / 100;

        if actual > threshold {
            let percent = (actual * 100) / limit;
            tracing::warn!(
                "Resource {:?} at {}% of limit: {} / {}",
                resource,
                percent,
                actual,
                limit
            );
        }
    }

    /// Get current limits
    pub fn limits(&self) -> &ResourceLimits {
        &self.limits
    }

    /// Create validator from HookConfig (for compatibility)
    pub fn from_hook_config(max_input_size_mb: u64) -> Self {
        let limits = ResourceLimits {
            max_input_size: (max_input_size_mb as usize) * BYTES_PER_MB,
            ..Default::default()
        };
        Self::new(limits)
    }
}

/// Legacy compatibility function
pub fn validate_resource_usage(
    memory_bytes: u64,
    execution_time_ms: u64,
    memory_limit: u64,
    time_limit: u64,
) -> Result<()> {
    let limits = ResourceLimits {
        max_memory_bytes: memory_limit as usize,
        max_execution_time_ms: time_limit,
        ..Default::default()
    };

    let validator = ResourceValidator::new(limits);
    let usage = ResourceUsage {
        memory_bytes: Some(memory_bytes as usize),
        execution_time_ms: Some(execution_time_ms),
        ..Default::default()
    };

    validator.validate_all_resources(&usage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_validation() {
        let validator = ResourceValidator::with_defaults();

        // Within limits
        assert!(validator.validate_memory(50 * BYTES_PER_MB).is_ok());

        // Exceeds limit
        assert!(validator.validate_memory(200 * BYTES_PER_MB).is_err());
    }

    #[test]
    fn test_input_size_validation() {
        let validator = ResourceValidator::with_defaults();

        // Within limits
        assert!(validator.validate_input_size(5 * BYTES_PER_MB).is_ok());

        // Exceeds limit
        assert!(validator.validate_input_size(20 * BYTES_PER_MB).is_err());
    }

    #[test]
    fn test_all_resources_validation() {
        let validator = ResourceValidator::with_defaults();

        let usage = ResourceUsage {
            memory_bytes: Some(50 * BYTES_PER_MB),
            input_size: Some(5 * BYTES_PER_MB),
            execution_time_ms: Some(2000),
            json_depth: Some(32),
            file_count: Some(100),
        };

        assert!(validator.validate_all_resources(&usage).is_ok());

        // Exceed one limit
        let bad_usage = ResourceUsage {
            memory_bytes: Some(200 * BYTES_PER_MB), // Exceeds
            ..usage
        };

        assert!(validator.validate_all_resources(&bad_usage).is_err());
    }

    #[test]
    fn test_legacy_compatibility() {
        // Test the legacy function still works
        assert!(
            validate_resource_usage(
                512 * BYTES_PER_MB as u64,
                1000,
                1024 * BYTES_PER_MB as u64,
                5000
            )
            .is_ok()
        );

        assert!(
            validate_resource_usage(
                2048 * BYTES_PER_MB as u64,
                1000,
                1024 * BYTES_PER_MB as u64,
                5000
            )
            .is_err()
        );
    }
}
