//! Domain user model and related wrappers for roles and creation/update input.

use chrono::NaiveDateTime;
use pushkind_common::domain::auth::AuthenticatedUser;
use serde::{Deserialize, Serialize};

use crate::domain::role::Role;
use crate::domain::types::{
    HubId, RoleId, TypeConstraintError, UserEmail, UserId, UserName, UserPassword,
};

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

impl User {
    /// Constructs a user from validated domain types.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: UserId,
        email: UserEmail,
        name: Option<UserName>,
        hub_id: HubId,
        password_hash: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
        roles: Vec<RoleId>,
    ) -> Self {
        Self {
            id,
            email,
            name,
            hub_id,
            password_hash,
            created_at,
            updated_at,
            roles,
        }
    }

    /// Validates raw values before constructing a user.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: i32,
        email: impl Into<String>,
        name: Option<String>,
        hub_id: i32,
        password_hash: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
        roles: Vec<i32>,
    ) -> Result<Self, TypeConstraintError> {
        let roles = roles
            .into_iter()
            .map(RoleId::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(
            UserId::try_from(id)?,
            UserEmail::try_from(email.into())?,
            name.map(UserName::try_from).transpose()?,
            HubId::try_from(hub_id)?,
            password_hash,
            created_at,
            updated_at,
            roles,
        ))
    }
}
#[derive(Clone, Serialize)]
/// Wrapper combining a [`User`] with the fully resolved [`Role`]s attached to
/// the account.
pub struct UserWithRoles {
    pub user: User,
    pub roles: Vec<Role>,
}

impl UserWithRoles {
    /// Constructs a user-with-roles bundle, syncing role IDs on the user.
    pub fn new(mut user: User, roles: Vec<Role>) -> Self {
        user.roles = roles.iter().map(|role| role.id).collect();
        Self { user, roles }
    }

    /// Builds a user-with-roles bundle without additional validation.
    pub fn try_new(user: User, roles: Vec<Role>) -> Result<Self, TypeConstraintError> {
        Ok(Self::new(user, roles))
    }
}

#[derive(Clone, Debug, Deserialize)]
/// Data required to create a new user.
pub struct NewUser {
    pub email: UserEmail,
    pub name: Option<UserName>,
    pub hub_id: HubId,
    pub password: UserPassword,
}

impl NewUser {
    /// Creates a new [`NewUser`] from already validated and normalized input.
    pub fn new(
        email: UserEmail,
        name: Option<UserName>,
        hub_id: HubId,
        password: UserPassword,
    ) -> Self {
        Self {
            email,
            name,
            hub_id,
            password,
        }
    }

    /// Validates raw values before constructing a new user payload.
    pub fn try_new(
        email: impl Into<String>,
        name: Option<String>,
        hub_id: i32,
        password: impl Into<String>,
    ) -> Result<Self, TypeConstraintError> {
        Ok(Self::new(
            UserEmail::try_from(email.into())?,
            name.map(UserName::try_from).transpose()?,
            HubId::try_from(hub_id)?,
            UserPassword::try_from(password.into())?,
        ))
    }
}

#[derive(Clone, Debug, Deserialize)]
/// Optional fields that can be updated for a user.
pub struct UpdateUser {
    pub name: UserName,
    pub password: Option<UserPassword>,
    pub roles: Option<Vec<RoleId>>,
}

impl UpdateUser {
    /// Constructs an update payload from validated domain types.
    pub fn new(name: UserName, password: Option<UserPassword>, roles: Option<Vec<RoleId>>) -> Self {
        Self {
            name,
            password,
            roles,
        }
    }

    /// Validates raw values before constructing an update payload.
    pub fn try_new(
        name: impl Into<String>,
        password: Option<String>,
        roles: Option<Vec<i32>>,
    ) -> Result<Self, TypeConstraintError> {
        let roles = roles
            .map(|roles| roles.into_iter().map(RoleId::try_from).collect())
            .transpose()?;
        Ok(Self::new(
            UserName::try_from(name.into())?,
            password.map(UserPassword::try_from).transpose()?,
            roles,
        ))
    }
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
