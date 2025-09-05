use crate::Result;
use std::fmt::Debug;

/// ValueObject trait - base for all domain value objects
pub trait ValueObject: Debug + Clone + PartialEq + Eq + Send + Sync {
    /// Validate the value object's state
    fn validate(&self) -> Result<()>;

    /// Check if this value object equals another
    fn equals(&self, other: &Self) -> bool {
        self == other
    }

    /// Check if the value object is valid
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// Email value object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email {
    value: String,
}

impl Email {
    pub fn new(value: String) -> Result<Self> {
        let email = Self { value };
        email.validate()?;
        Ok(email)
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl ValueObject for Email {
    fn validate(&self) -> Result<()> {
        crate::ddd::entity::EntityValidator::validate_email(&self.value)
    }
}

impl TryFrom<String> for Email {
    type Error = crate::DddError;

    fn try_from(value: String) -> Result<Self> {
        Email::new(value)
    }
}

impl TryFrom<&str> for Email {
    type Error = crate::DddError;

    fn try_from(value: &str) -> Result<Self> {
        Email::new(value.to_string())
    }
}

/// Money value object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    amount: i64,    // in smallest currency unit (cents)
    currency: String,
}

impl Money {
    pub fn new(amount: i64, currency: String) -> Result<Self> {
        let money = Self { amount, currency };
        money.validate()?;
        Ok(money)
    }

    pub fn amount(&self) -> i64 {
        self.amount
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn add(&self, other: &Money) -> Result<Self> {
        if self.currency != other.currency {
            return Err(crate::DddError::validation("Cannot add money with different currencies"));
        }
        Ok(Self {
            amount: self.amount + other.amount,
            currency: self.currency.clone(),
        })
    }

    pub fn subtract(&self, other: &Money) -> Result<Self> {
        if self.currency != other.currency {
            return Err(crate::DddError::validation("Cannot subtract money with different currencies"));
        }
        if self.amount < other.amount {
            return Err(crate::DddError::validation("Insufficient funds"));
        }
        Ok(Self {
            amount: self.amount - other.amount,
            currency: self.currency.clone(),
        })
    }
}

impl ValueObject for Money {
    fn validate(&self) -> Result<()> {
        if self.amount < 0 {
            return Err(crate::DddError::validation("Amount cannot be negative"));
        }
        if self.currency.is_empty() {
            return Err(crate::DddError::validation("Currency cannot be empty"));
        }
        if self.currency.len() != 3 {
            return Err(crate::DddError::validation("Currency must be 3 characters"));
        }
        Ok(())
    }
}

/// Address value object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    street: String,
    city: String,
    state: String,
    postal_code: String,
    country: String,
}

impl Address {
    pub fn new(
        street: String,
        city: String,
        state: String,
        postal_code: String,
        country: String,
    ) -> Result<Self> {
        let address = Self {
            street,
            city,
            state,
            postal_code,
            country,
        };
        address.validate()?;
        Ok(address)
    }

    pub fn street(&self) -> &str {
        &self.street
    }

    pub fn city(&self) -> &str {
        &self.city
    }

    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn postal_code(&self) -> &str {
        &self.postal_code
    }

    pub fn country(&self) -> &str {
        &self.country
    }
}

impl ValueObject for Address {
    fn validate(&self) -> Result<()> {
        crate::ddd::entity::EntityValidator::validate_required("street", Some(&self.street))?;
        crate::ddd::entity::EntityValidator::validate_required("city", Some(&self.city))?;
        crate::ddd::entity::EntityValidator::validate_required("state", Some(&self.state))?;
        crate::ddd::entity::EntityValidator::validate_required("postal_code", Some(&self.postal_code))?;
        crate::ddd::entity::EntityValidator::validate_required("country", Some(&self.country))?;
        Ok(())
    }
}

/// Percentage value object (0-100)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Percentage {
    value: u8,
}

impl Percentage {
    pub fn new(value: u8) -> Result<Self> {
        let percentage = Self { value };
        percentage.validate()?;
        Ok(percentage)
    }

    pub fn value(&self) -> u8 {
        self.value
    }

    pub fn as_decimal(&self) -> f64 {
        self.value as f64 / 100.0
    }
}

impl ValueObject for Percentage {
    fn validate(&self) -> Result<()> {
        if self.value > 100 {
            return Err(crate::DddError::validation("Percentage cannot exceed 100"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_creation() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        assert_eq!(email.value(), "test@example.com");
        assert!(email.is_valid());
    }

    #[test]
    fn test_email_validation() {
        let result = Email::new("invalid-email".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_money_creation() {
        let money = Money::new(100, "USD".to_string()).unwrap();
        assert_eq!(money.amount(), 100);
        assert_eq!(money.currency(), "USD");
        assert!(money.is_valid());
    }

    #[test]
    fn test_money_addition() {
        let money1 = Money::new(100, "USD".to_string()).unwrap();
        let money2 = Money::new(50, "USD".to_string()).unwrap();
        let result = money1.add(&money2).unwrap();
        assert_eq!(result.amount(), 150);
    }

    #[test]
    fn test_money_subtraction() {
        let money1 = Money::new(100, "USD".to_string()).unwrap();
        let money2 = Money::new(50, "USD".to_string()).unwrap();
        let result = money1.subtract(&money2).unwrap();
        assert_eq!(result.amount(), 50);
    }

    #[test]
    fn test_percentage_creation() {
        let percentage = Percentage::new(50).unwrap();
        assert_eq!(percentage.value(), 50);
        assert_eq!(percentage.as_decimal(), 0.5);
    }
}