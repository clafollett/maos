/// MAOS Domain Layer
///
/// This crate contains the pure business logic for the Multi-Agent Orchestration System.
/// It follows Domain-Driven Design principles with no dependencies on external frameworks
/// or infrastructure concerns.
pub mod aggregates;
pub mod events;
pub mod services;
pub mod value_objects;

// Re-export commonly used types (enabled as modules are populated)
pub use aggregates::*;
// pub use events::*;
// pub use services::*;
// pub use value_objects::*;
