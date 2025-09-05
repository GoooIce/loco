//! Core DDD concepts and implementations

pub mod aggregate;
pub mod entity;
pub mod value_object;
pub mod repository;
pub mod service;
pub mod event;
pub mod command;
pub mod query;

// Re-export core types
pub use aggregate::*;
pub use entity::*;
pub use value_object::*;
pub use repository::*;
pub use service::*;
pub use event::*;
pub use command::*;
pub use query::*;

use crate::Result;
use std::fmt::Debug;
use uuid::Uuid;

/// Trait for domain identifiers
pub trait Identifier: Debug + Clone + PartialEq + Eq + Send + Sync {
    fn as_string(&self) -> String;
    fn from_str(s: &str) -> Result<Self>
    where
        Self: Sized;
}

impl Identifier for Uuid {
    fn as_string(&self) -> String {
        self.to_string()
    }

    fn from_str(s: &str) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Uuid::parse_str(s)?)
    }
}

impl Identifier for String {
    fn as_string(&self) -> String {
        self.clone()
    }

    fn from_str(s: &str) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(s.to_string())
    }
}

/// Domain version tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u32);

impl Version {
    pub fn new() -> Self {
        Version(0)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new()
    }
}

/// Domain timestamp
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainTimestamp(chrono::DateTime<chrono::Utc>);

impl DomainTimestamp {
    pub fn new() -> Self {
        Self(chrono::Utc::now())
    }

    pub fn from_datetime(dt: chrono::DateTime<chrono::Utc>) -> Self {
        Self(dt)
    }

    pub fn as_datetime(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.0
    }

    pub fn as_str(&self) -> String {
        self.0.to_rfc3339()
    }
}

impl Default for DomainTimestamp {
    fn default() -> Self {
        Self::new()
    }
}