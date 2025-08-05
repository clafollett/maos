//! Core type definitions for MAOS
//!
//! This module contains all the fundamental types used throughout MAOS,
//! organized into logical submodules for clarity and maintainability.

pub mod agent;
pub mod session;
pub mod tool;

/// Macro to implement common ID type functionality
///
/// This macro generates the implementation for ID types that follow
/// the pattern: `{prefix}_{uuid}`
///
/// # Example
///
/// ```ignore
/// impl_id_type!(SessionId, "sess");
/// ```
#[macro_export]
macro_rules! impl_id_type {
    ($name:ident, $prefix:expr) => {
        impl $name {
            /// Generate a new unique ID
            pub fn generate() -> Self {
                let uuid = uuid::Uuid::new_v4();
                Self(format!("{}_{}", $prefix, uuid))
            }

            /// Check if the ID format is valid
            pub fn is_valid(&self) -> bool {
                let parts: Vec<&str> = self.0.splitn(2, '_').collect();

                if parts.len() != 2 || parts[0] != $prefix {
                    return false;
                }

                // Validate UUID format
                uuid::Uuid::parse_str(parts[1]).is_ok()
            }

            /// Get the ID as a string slice
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let id = $name(s.to_string());
                if id.is_valid() {
                    Ok(id)
                } else {
                    Err(format!("Invalid {} format: {}", stringify!($name), s))
                }
            }
        }
    };
}
