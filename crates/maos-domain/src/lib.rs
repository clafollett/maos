/// MAOS Domain Layer
///
/// This crate contains the pure business logic for the Multi-Agent Orchestration System.
/// It follows Domain-Driven Design principles with clean separation of concerns.
pub mod aggregates;
// pub mod events;  // Temporarily disabled
// pub mod repositories;  // Temporarily disabled
// pub mod services;  // Temporarily disabled
pub mod value_objects;

// Re-export commonly used types for convenience
pub use aggregates::*;
// pub use events::*;
// pub use repositories::*;
// pub use services::*;
pub use value_objects::*;
