//! Main library crate for the Pushkind authentication service.
//!
//! This crate exposes the domain models, database access layer and HTTP
//! handlers that make up the application. It is used by `main.rs` to build
//! the Actix-Web application and can also be reused for integration tests.

pub mod domain;
pub mod forms;
pub mod middleware;
pub mod models;
pub mod repository;
pub mod routes;
pub mod schema;
pub mod services;

pub const SERVICE_ACCESS_ROLE: &str = "admin";
