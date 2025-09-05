use crate::ddd::{Identifier, Version, DomainTimestamp};
use crate::Result;
use async_trait::async_trait;
use std::fmt::Debug;

/// Entity trait - base for all domain entities
#[async_trait]
pub trait Entity: Debug + Send + Sync {
    type Id: Identifier;

    /// Get the entity's unique identifier
    fn id(&self) -> &Self::Id;

    /// Check if this entity equals another entity
    fn equals(&self, other: &Self) -> bool {
        self.id() == other.id()
    }

    /// Validate the entity's state
    fn validate(&self) -> Result<()>;

    /// Get the entity's version (for concurrency control)
    fn version(&self) -> &Version;

    /// Get the entity's creation timestamp
    fn created_at(&self) -> &DomainTimestamp;

    /// Get the entity's last update timestamp
    fn updated_at(&self) -> &DomainTimestamp;

    /// Check if the entity is valid
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// BaseEntity - provides common entity functionality
#[derive(Debug)]
pub struct BaseEntity<T: Identifier> {
    id: T,
    version: Version,
    created_at: DomainTimestamp,
    updated_at: DomainTimestamp,
}

impl<T: Identifier> BaseEntity<T> {
    pub fn new(id: T) -> Self {
        let now = DomainTimestamp::new();
        Self {
            id,
            version: Version::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn id(&self) -> &T {
        &self.id
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn created_at(&self) -> &DomainTimestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DomainTimestamp {
        &self.updated_at
    }

    pub fn increment_version(&mut self) {
        self.version.increment();
        self.updated_at = DomainTimestamp::new();
    }

    pub fn touch(&mut self) {
        self.updated_at = DomainTimestamp::new();
    }
}

/// Entity validation helper
pub struct EntityValidator;

impl EntityValidator {
    pub fn validate_required<T: AsRef<str>>(field: &str, value: Option<T>) -> Result<()> {
        match value {
            Some(v) if !v.as_ref().trim().is_empty() => Ok(()),
            _ => Err(crate::DddError::validation(format!("{} is required", field))),
        }
    }

    pub fn validate_length(field: &str, value: &str, min: usize, max: usize) -> Result<()> {
        if value.len() < min {
            Err(crate::DddError::validation(format!(
                "{} must be at least {} characters",
                field, min
            )))
        } else if value.len() > max {
            Err(crate::DddError::validation(format!(
                "{} must be at most {} characters",
                field, max
            )))
        } else {
            Ok(())
        }
    }

    pub fn validate_email(email: &str) -> Result<()> {
        use regex::Regex;
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|e| crate::DddError::validation(format!("Invalid email regex: {}", e)))?;
        
        if email_regex.is_match(email) {
            Ok(())
        } else {
            Err(crate::DddError::validation("Invalid email format"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestEntity {
        base: BaseEntity<Uuid>,
        name: String,
    }

    impl Entity for TestEntity {
        type Id = Uuid;

        fn id(&self) -> &Self::Id {
            self.base.id()
        }

        fn validate(&self) -> Result<()> {
            EntityValidator::validate_required("name", Some(&self.name))?;
            EntityValidator::validate_length("name", &self.name, 2, 100)?;
            Ok(())
        }

        fn version(&self) -> &Version {
            self.base.version()
        }

        fn created_at(&self) -> &DomainTimestamp {
            self.base.created_at()
        }

        fn updated_at(&self) -> &DomainTimestamp {
            self.base.updated_at()
        }
    }

    #[test]
    fn test_entity_creation() {
        let id = Uuid::new_v4();
        let entity = TestEntity {
            base: BaseEntity::new(id),
            name: "Test Entity".to_string(),
        };

        assert_eq!(entity.id(), &id);
        assert!(entity.is_valid());
    }

    #[test]
    fn test_entity_validation() {
        let id = Uuid::new_v4();
        let entity = TestEntity {
            base: BaseEntity::new(id),
            name: "".to_string(),
        };

        assert!(!entity.is_valid());
    }

    #[test]
    fn test_entity_equals() {
        let id = Uuid::new_v4();
        let entity1 = TestEntity {
            base: BaseEntity::new(id.clone()),
            name: "Test Entity".to_string(),
        };

        let entity2 = TestEntity {
            base: BaseEntity::new(id),
            name: "Different Name".to_string(),
        };

        assert!(entity1.equals(&entity2));
    }
}