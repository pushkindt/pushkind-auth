//! Strongly-typed value objects used by domain entities.
//!
//! These wrappers enforce basic invariants (e.g., positive identifiers,
//! normalized/validated email) so that once a value reaches the domain layer it
//! can be treated as trusted.
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use thiserror::Error;
use validator::{ValidateEmail, ValidateUrl};

/// Errors produced when attempting to construct a constrained value object.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TypeConstraintError {
    /// Provided identifier is zero or negative.
    #[error("id must be greater than zero")]
    NonPositiveId,
    /// Provided email failed format validation.
    #[error("invalid email address")]
    InvalidEmail,
    /// Provided url failed format validation.
    #[error("invalid url address")]
    InvalidUrl,
    /// Provided string contained no non-whitespace characters.
    #[error("value cannot be empty")]
    EmptyString,
}

/// Macro to generate lightweight newtypes for positive identifiers.
macro_rules! id_newtype {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
        pub struct $name(i32);

        impl $name {
            /// Creates a new identifier ensuring it is greater than zero.
            pub fn new(value: i32) -> Result<Self, TypeConstraintError> {
                if value > 0 {
                    Ok(Self(value))
                } else {
                    Err(TypeConstraintError::NonPositiveId)
                }
            }

            /// Returns the raw `i32` backing this identifier.
            pub const fn get(self) -> i32 {
                self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl TryFrom<i32> for $name {
            type Error = TypeConstraintError;

            fn try_from(value: i32) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for i32 {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

id_newtype!(UserId);
id_newtype!(HubId);
id_newtype!(RoleId);
id_newtype!(MenuId);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Lower-cased and validated email address.
pub struct UserEmail(String);

impl UserEmail {
    /// Validates and normalizes an email string.
    pub fn new<S: Into<String>>(email: S) -> Result<Self, TypeConstraintError> {
        let normalized = email.into().trim().to_lowercase();
        if normalized.validate_email() {
            Ok(Self(normalized))
        } else {
            Err(TypeConstraintError::InvalidEmail)
        }
    }

    /// Borrow the email as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the owned inner `String`.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Display for UserEmail {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for UserEmail {
    type Error = TypeConstraintError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for UserEmail {
    type Error = TypeConstraintError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<UserEmail> for String {
    fn from(value: UserEmail) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Wrapper for non-empty, trimmed strings.
pub struct NonEmptyString(String);

impl NonEmptyString {
    /// Trims whitespace and rejects empty inputs.
    pub fn new<S: Into<String>>(value: S) -> Result<Self, TypeConstraintError> {
        let trimmed = value.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(TypeConstraintError::EmptyString);
        }
        Ok(Self(trimmed))
    }

    /// Borrow the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the wrapper returning the owned string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Display for NonEmptyString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for NonEmptyString {
    type Error = TypeConstraintError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for NonEmptyString {
    type Error = TypeConstraintError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<NonEmptyString> for String {
    fn from(value: NonEmptyString) -> Self {
        value.0
    }
}

macro_rules! non_empty_string_newtype {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Constructs a trimmed, non-empty value.
            pub fn new<S: Into<String>>(value: S) -> Result<Self, TypeConstraintError> {
                let inner = NonEmptyString::new(value)?;
                Ok(Self(inner.into_inner()))
            }

            /// Borrow the value as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consume the wrapper and return the owned string.
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl TryFrom<String> for $name {
            type Error = TypeConstraintError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = TypeConstraintError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

non_empty_string_newtype!(HubName, "Hub name wrapper enforcing non-empty values.");

non_empty_string_newtype!(
    RoleName,
    "User role name wrapper enforcing non-empty values."
);

non_empty_string_newtype!(MenuName, "Menu name wrapper enforcing non-empty values.");

non_empty_string_newtype!(UserName, "User name wrapper enforcing non-empty values.");

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// User password wrapper enforcing non-empty values.
///
/// Intentionally does not implement [`Display`] or [`Deref`] to reduce the risk
/// of accidental logging/formatting or implicit string coercions.
pub struct UserPassword(String);

impl UserPassword {
    /// Constructs a trimmed, non-empty password.
    pub fn new<S: Into<String>>(value: S) -> Result<Self, TypeConstraintError> {
        let inner = NonEmptyString::new(value)?;
        Ok(Self(inner.into_inner()))
    }

    /// Borrow the password as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the wrapper and return the owned string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl TryFrom<String> for UserPassword {
    type Error = TypeConstraintError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for UserPassword {
    type Error = TypeConstraintError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<UserPassword> for String {
    fn from(value: UserPassword) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Non-empty, trimmed menu URL.
pub struct MenuUrl(String);

impl MenuUrl {
    /// Ensures a trimmed menu URL is non-empty before wrapping.
    pub fn new<S: Into<String>>(value: S) -> Result<Self, TypeConstraintError> {
        let url = NonEmptyString::new(value)?;

        if !url.as_str().validate_url() {
            Err(TypeConstraintError::InvalidUrl)
        } else {
            Ok(Self(url.into_inner()))
        }
    }

    /// Borrow the menu URL.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extract the owned menu URL.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Display for MenuUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for MenuUrl {
    type Error = TypeConstraintError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for MenuUrl {
    type Error = TypeConstraintError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<MenuUrl> for String {
    fn from(value: MenuUrl) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_newtype_rejects_non_positive_values() {
        assert_eq!(
            UserId::new(0).unwrap_err(),
            TypeConstraintError::NonPositiveId
        );
        assert_eq!(
            UserId::try_from(-5).unwrap_err(),
            TypeConstraintError::NonPositiveId
        );
    }

    #[test]
    fn id_newtype_roundtrips_value() {
        let id = UserId::new(42).unwrap();
        assert_eq!(id.get(), 42);
        assert_eq!(i32::from(id), 42);
        assert_eq!(format!("{id}"), "42");
    }

    #[test]
    fn user_email_normalizes_and_validates() {
        let email = UserEmail::new("  Example@Email.com ").unwrap();
        assert_eq!(email.as_str(), "example@email.com");
        assert_eq!(String::from(email.clone()), "example@email.com");
        assert_eq!(
            UserEmail::try_from("not-an-email").unwrap_err(),
            TypeConstraintError::InvalidEmail
        );
    }

    #[test]
    fn non_empty_string_trims_and_rejects_empty() {
        let value = NonEmptyString::new("  hello ").unwrap();
        assert_eq!(value.as_str(), "hello");
        assert_eq!(
            NonEmptyString::try_from("   ").unwrap_err(),
            TypeConstraintError::EmptyString
        );
    }

    #[test]
    fn named_string_newtype_uses_trimmed_value() {
        let name = HubName::new("  Alpha ").unwrap();
        assert_eq!(name.as_str(), "Alpha");
        assert_eq!(&*name, "Alpha");
        assert_eq!(format!("{name}"), "Alpha");
        assert_eq!(String::from(name.clone()), "Alpha");
    }

    #[test]
    fn user_password_rejects_empty_values() {
        let password = UserPassword::new("  secret ").unwrap();
        assert_eq!(password.as_str(), "secret");
        assert_eq!(
            UserPassword::try_from(" ").unwrap_err(),
            TypeConstraintError::EmptyString
        );
    }

    #[test]
    fn menu_url_validates_format() {
        let url = MenuUrl::new("https://example.com/path").unwrap();
        assert_eq!(url.as_str(), "https://example.com/path");
        assert_eq!(
            MenuUrl::try_from("not-a-url").unwrap_err(),
            TypeConstraintError::InvalidUrl
        );
    }
}
