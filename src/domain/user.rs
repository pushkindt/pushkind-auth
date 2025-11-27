use chrono::NaiveDateTime;
use pushkind_common::domain::auth::AuthenticatedUser;
use serde::{Deserialize, Serialize};

use crate::domain::role::Role;
use crate::domain::types::{HubId, RoleId, UserEmail, UserId, UserName};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Representation of a user in the system.
///
/// This struct mirrors the data stored in the database but is free of any
/// persistence related logic.
pub struct User {
    pub id: UserId,
    pub email: UserEmail,
    pub name: Option<UserName>,
    pub hub_id: HubId,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub roles: Vec<RoleId>,
}

#[derive(Clone, Serialize)]
/// Wrapper combining a [`User`] with the fully resolved [`Role`]s attached to
/// the account.
pub struct UserWithRoles {
    pub user: User,
    pub roles: Vec<Role>,
}

#[derive(Clone, Debug, Deserialize)]
/// Data required to create a new user.
pub struct NewUser {
    pub email: UserEmail,
    pub name: Option<UserName>,
    pub hub_id: HubId,
    pub password: String,
}

impl NewUser {
    /// Creates a new [`NewUser`] from already validated and normalized input.
    #[must_use]
    pub fn new(email: UserEmail, name: Option<UserName>, hub_id: HubId, password: String) -> Self {
        Self {
            email,
            name,
            hub_id,
            password,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
/// Optional fields that can be updated for a user.
pub struct UpdateUser {
    pub name: UserName,
    pub password: Option<String>,
    pub roles: Option<Vec<RoleId>>,
}

impl From<User> for AuthenticatedUser {
    fn from(user: User) -> Self {
        let mut result = Self {
            sub: user.id.to_string(),
            email: user.email.as_str().to_string(),
            hub_id: user.hub_id.get(),
            name: user.name.map(|n| n.into_inner()).unwrap_or_default(),
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
            email: ur.user.email.as_str().to_string(),
            hub_id: ur.user.hub_id.get(),
            name: ur.user.name.map(|n| n.into_inner()).unwrap_or_default(),
            roles: ur.roles.into_iter().map(|r| r.name.into_inner()).collect(),
            exp: 0,
        };
        result.set_expiration(7);
        result
    }
}
