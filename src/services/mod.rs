//! Core service layer containing business logic used by routes.
//!
//! Submodules:
//! - [`admin`]: administrative operations.
//! - [`api`]: API-facing utilities.
//! - [`auth`]: authentication workflows.
//! - [`main`]: main application view helpers.

use crate::domain::types::TypeConstraintError;
use pushkind_common::services::errors::ServiceError;

pub mod admin;
pub mod api;
pub mod auth;
pub mod main;

pub(crate) fn map_type_error(err: TypeConstraintError) -> ServiceError {
    ServiceError::Form(err.to_string())
}
