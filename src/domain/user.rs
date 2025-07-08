use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Representation of a user in the system.
///
/// This struct mirrors the data stored in the database but is free of any
/// persistence related logic.
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: Option<String>,
    pub hub_id: i32,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize)]
/// Data required to create a new user.
pub struct NewUser<'a> {
    pub email: &'a str,
    pub name: Option<&'a str>,
    pub hub_id: i32,
    pub password: &'a str,
}

#[derive(Clone, Debug, Deserialize)]
/// Optional fields that can be updated for a user.
pub struct UpdateUser<'a> {
    pub name: Option<&'a str>,
    pub password: Option<&'a str>,
}
