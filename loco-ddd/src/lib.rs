//! Loco DDD - Domain-Driven Design support for Loco framework
//!
//! This library provides comprehensive DDD (Domain-Driven Design) support
//! for applications built with the Loco framework.

pub mod ddd;
pub mod error;
pub mod prelude;

// Re-export commonly used types
pub use ddd::*;
pub use error::*;

/// Result type used throughout the DDD library
pub type Result<T> = std::result::Result<T, DddError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_initialization() {
        // Test that the library can be initialized
        assert!(true);
    }
}