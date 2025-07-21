//! Database models used by Diesel.
//!
//! These types closely mirror the schema of the database and are used by the
//! repository layer. They also implement conversions to the domain layer types.

pub mod config;
pub mod hub;
pub mod menu;
pub mod role;
pub mod user;
