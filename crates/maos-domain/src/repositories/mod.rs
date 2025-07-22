//! Repository traits for domain aggregates
//! 
//! These traits define the persistence contracts for domain aggregates.
//! Implementations will be provided in the infrastructure layer.

pub mod session_repository;
pub mod agent_repository;
pub mod instance_repository;

pub use agent_repository::*;
pub use instance_repository::*;
pub use session_repository::*;