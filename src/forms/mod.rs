//! Request form definitions used by Actix handlers.
//!
//! These structs validate incoming payloads and translate them into domain
//! types so the service layer can remain framework-agnostic.
pub mod auth;
pub mod main;
