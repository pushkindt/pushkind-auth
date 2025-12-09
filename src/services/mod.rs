//! Core service layer containing business logic used by routes.
//!
//! Submodules:
//! - [`admin`]: administrative operations.
//! - [`api`]: API-facing utilities.
//! - [`auth`]: authentication workflows.
//! - [`main`]: main application view helpers.

use crate::domain::types::TypeConstraintError;
use pushkind_common::services::errors::{ServiceError, ServiceResult};
use validator::Validate;

pub mod admin;
pub mod api;
pub mod auth;
pub mod main;

impl From<TypeConstraintError> for ServiceError {
    fn from(err: TypeConstraintError) -> Self {
        ServiceError::Form(err.to_string())
    }
}

pub(crate) fn validate_form<T: Validate>(form: &T) -> ServiceResult<()> {
    form.validate()
        .map_err(|e| ServiceError::Form(e.to_string()))
}
