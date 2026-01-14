//! Error conversion glue for `data` feature consumers.
//!
//! The domain layer must not depend on service/repository error types, but
//! downstream crates using `pushkind-emailer` with only the `data` feature may
//! still want convenient conversions.
use pushkind_common::repository::errors::RepositoryError;
use pushkind_common::services::errors::ServiceError;

#[cfg(feature = "data")]
use crate::domain::types::TypeConstraintError;
#[cfg(feature = "server")]
use crate::forms::FormError;

#[cfg(feature = "data")]
impl From<TypeConstraintError> for ServiceError {
    fn from(val: TypeConstraintError) -> Self {
        ServiceError::TypeConstraint(val.to_string())
    }
}

#[cfg(feature = "server")]
impl From<FormError> for ServiceError {
    fn from(val: FormError) -> Self {
        ServiceError::Form(val.to_string())
    }
}

#[cfg(feature = "data")]
impl From<TypeConstraintError> for RepositoryError {
    fn from(val: TypeConstraintError) -> Self {
        RepositoryError::ValidationError(val.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "data")]
    #[test]
    fn type_constraint_converts_to_service_error() {
        let err: ServiceError = TypeConstraintError::InvalidEmail.into();
        match err {
            ServiceError::TypeConstraint(message) => {
                assert!(message.contains("invalid email address"));
            }
            _ => panic!("expected TypeConstraint service error"),
        }
    }

    #[cfg(feature = "data")]
    #[test]
    fn type_constraint_converts_to_repository_error() {
        let err: RepositoryError = TypeConstraintError::EmptyString.into();
        match err {
            RepositoryError::ValidationError(message) => {
                assert!(message.contains("value cannot be empty"));
            }
            _ => panic!("expected validation repository error"),
        }
    }

    #[cfg(feature = "server")]
    #[test]
    fn form_error_converts_to_service_error() {
        let err: ServiceError = FormError::InvalidEmail.into();
        match err {
            ServiceError::Form(message) => {
                assert!(message.contains("invalid email address"));
            }
            _ => panic!("expected form service error"),
        }
    }
}
