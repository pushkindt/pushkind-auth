//! Request form definitions used by Actix handlers.
//!
//! These structs validate incoming payloads and translate them into domain
//! types so the service layer can remain framework-agnostic.

use thiserror::Error;
use validator::ValidationErrors;

pub mod auth;
pub mod main;

#[derive(Debug, Error)]
/// Errors that can occur when processing form data.
pub enum FormError {
    #[error("validation errors: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("invalid email address")]
    InvalidEmail,

    #[error("invalid password")]
    InvalidPassword,

    #[error("invalid hub_id")]
    InvalidHubId,

    #[error("invalid name")]
    InvalidName,

    #[error("invalid url")]
    InvalidUrl,

    #[error("invalid role")]
    InvalidRoleId,
}
