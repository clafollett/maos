//! Common validation traits for security components
//!
//! This module provides reusable traits that standardize validation
//! patterns across all security validators.

use std::fmt::Debug;

/// Type alias for validation results to simplify signatures
pub type ValidationResult<E> = std::result::Result<(), E>;

/// Core trait for all security validators
pub trait SecurityValidator<T> {
    /// The error type returned by validation
    type Error: Debug;

    /// Validate the input and return an error if validation fails
    fn validate(&self, input: &T) -> ValidationResult<Self::Error>;

    /// Check if the input is safe (convenience method)
    fn is_safe(&self, input: &T) -> bool {
        self.validate(input).is_ok()
    }

    /// Get a description of what this validator checks
    fn description(&self) -> &'static str;
}

/// Trait for pattern-based security matchers
pub trait PatternMatcher {
    /// Check if the input matches a security pattern
    fn matches_security_pattern(&self, input: &str) -> bool;

    /// Get the name of the pattern being matched
    fn pattern_name(&self) -> &'static str;

    /// Get a description of what this pattern detects
    fn pattern_description(&self) -> &'static str;
}

/// Trait for validators that can be chained together
pub trait ChainableValidator<T>: SecurityValidator<T> {
    /// Chain this validator with another
    #[allow(clippy::type_complexity)]
    fn and_then<V>(self, other: V) -> ChainedValidator<T, Self, V>
    where
        Self: Sized,
        V: SecurityValidator<T, Error = Self::Error>,
    {
        ChainedValidator {
            first: self,
            second: other,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// A validator that chains two validators together
pub struct ChainedValidator<T, V1, V2> {
    first: V1,
    second: V2,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, V1, V2> SecurityValidator<T> for ChainedValidator<T, V1, V2>
where
    V1: SecurityValidator<T>,
    V2: SecurityValidator<T, Error = V1::Error>,
{
    type Error = V1::Error;

    fn validate(&self, input: &T) -> ValidationResult<Self::Error> {
        self.first.validate(input)?;
        self.second.validate(input)
    }

    fn description(&self) -> &'static str {
        "Chained validator"
    }
}

/// Trait for validators that can provide detailed validation reports
pub trait ReportingValidator<T>: SecurityValidator<T> {
    /// Type representing a validation report
    type Report: Debug;

    /// Validate and return a detailed report
    fn validate_with_report(&self, input: &T) -> Self::Report;
}

/// Trait for validators that support configuration
pub trait ConfigurableValidator {
    /// Configuration type for this validator
    type Config: Clone + Debug;

    /// Create a new validator with the given configuration
    fn with_config(config: Self::Config) -> Self;

    /// Get the current configuration
    fn config(&self) -> &Self::Config;

    /// Update the configuration
    fn set_config(&mut self, config: Self::Config);
}

/// Trait for validators that can be composed with OR logic
pub trait OrValidator<T>: SecurityValidator<T> {
    /// Create a validator that passes if either this OR other passes
    #[allow(clippy::type_complexity)]
    fn or<V>(self, other: V) -> OrComposedValidator<T, Self, V>
    where
        Self: Sized,
        V: SecurityValidator<T, Error = Self::Error>,
    {
        OrComposedValidator {
            first: self,
            second: other,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// A validator that passes if either of two validators pass
pub struct OrComposedValidator<T, V1, V2> {
    first: V1,
    second: V2,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, V1, V2> SecurityValidator<T> for OrComposedValidator<T, V1, V2>
where
    V1: SecurityValidator<T>,
    V2: SecurityValidator<T, Error = V1::Error>,
{
    type Error = V1::Error;

    fn validate(&self, input: &T) -> ValidationResult<Self::Error> {
        match self.first.validate(input) {
            Ok(()) => Ok(()),
            Err(_) => self.second.validate(input),
        }
    }

    fn description(&self) -> &'static str {
        "OR composed validator"
    }
}

/// Standard implementation for all pattern matchers
pub struct PatternMatcherImpl {
    pattern: String,
    name: &'static str,
    description: &'static str,
}

impl PatternMatcherImpl {
    pub fn new(pattern: impl Into<String>, name: &'static str, description: &'static str) -> Self {
        Self {
            pattern: pattern.into(),
            name,
            description,
        }
    }
}

impl PatternMatcher for PatternMatcherImpl {
    fn matches_security_pattern(&self, input: &str) -> bool {
        input.contains(&self.pattern)
    }

    fn pattern_name(&self) -> &'static str {
        self.name
    }

    fn pattern_description(&self) -> &'static str {
        self.description
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example validator for testing
    struct LengthValidator {
        max_length: usize,
    }

    impl SecurityValidator<String> for LengthValidator {
        type Error = String;

        fn validate(&self, input: &String) -> ValidationResult<Self::Error> {
            if input.len() > self.max_length {
                Err(format!(
                    "String too long: {} > {}",
                    input.len(),
                    self.max_length
                ))
            } else {
                Ok(())
            }
        }

        fn description(&self) -> &'static str {
            "Length validator"
        }
    }

    impl ChainableValidator<String> for LengthValidator {}

    #[test]
    fn test_basic_validator() {
        let validator = LengthValidator { max_length: 10 };

        assert!(validator.is_safe(&"short".to_string()));
        assert!(!validator.is_safe(&"this is a very long string".to_string()));
    }

    #[test]
    fn test_chained_validators() {
        let validator1 = LengthValidator { max_length: 20 };
        let validator2 = LengthValidator { max_length: 10 };

        let chained = validator1.and_then(validator2);

        // Must pass both validators
        assert!(chained.is_safe(&"short".to_string()));
        assert!(!chained.is_safe(&"medium length str".to_string())); // Fails second validator
    }

    #[test]
    fn test_pattern_matcher() {
        let matcher =
            PatternMatcherImpl::new("../", "path_traversal", "Detects path traversal attempts");

        assert!(matcher.matches_security_pattern("../etc/passwd"));
        assert!(!matcher.matches_security_pattern("safe/path"));
        assert_eq!(matcher.pattern_name(), "path_traversal");
    }
}
