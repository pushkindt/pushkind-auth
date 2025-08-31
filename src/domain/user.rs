use chrono::NaiveDateTime;
use pushkind_common::models::auth::AuthenticatedUser;
use serde::{Deserialize, Serialize};

use crate::domain::role::Role;

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
    pub roles: Vec<i32>,
}

#[derive(Serialize)]
pub struct UserWithRoles {
    pub user: User,
    pub roles: Vec<Role>,
}

#[derive(Clone, Debug, Deserialize)]
/// Data required to create a new user.
pub struct NewUser<'a> {
    pub email: String,
    pub name: Option<&'a str>,
    pub hub_id: i32,
    pub password: &'a str,
}

impl<'a> NewUser<'a> {
    /// Creates a new [`NewUser`] ensuring the email is lowercased.
    pub fn new(email: &'a str, name: Option<&'a str>, hub_id: i32, password: &'a str) -> Self {
        Self {
            email: email.to_lowercase(),
            name,
            hub_id,
            password,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
/// Optional fields that can be updated for a user.
pub struct UpdateUser<'a> {
    pub name: &'a str,
    pub password: Option<&'a str>,
}

impl From<User> for AuthenticatedUser {
    fn from(user: User) -> Self {
        let mut result = Self {
            sub: user.id.to_string(),
            email: user.email,
            hub_id: user.hub_id,
            name: user.name.unwrap_or_default(),
            roles: vec![],
            exp: 0,
        };
        result.set_expiration(7);
        result
    }
}

impl From<UserWithRoles> for AuthenticatedUser {
    fn from(ur: UserWithRoles) -> Self {
        let mut result = Self {
            sub: ur.user.id.to_string(),
            email: ur.user.email,
            hub_id: ur.user.hub_id,
            name: ur.user.name.unwrap_or_default(),
            roles: ur.roles.into_iter().map(|r| r.name).collect(),
            exp: 0,
        };
        result.set_expiration(7);
        result
    }
}
