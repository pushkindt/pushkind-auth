//! Domain models for roles and user-role associations.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::domain::types::{RoleId, RoleName, TypeConstraintError, UserId};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Role assigned to a user describing a set of permissions.
pub struct Role {
    pub id: RoleId,
    pub name: RoleName,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Role {
    /// Constructs a role from validated domain types.
    pub fn new(
        id: RoleId,
        name: RoleName,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    ) -> Self {
        Self {
            id,
            name,
            created_at,
            updated_at,
        }
    }

    /// Validates raw values before constructing a role.
    pub fn try_new(
        id: i32,
        name: impl Into<String>,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    ) -> Result<Self, TypeConstraintError> {
        Ok(Self::new(
            RoleId::try_from(id)?,
            RoleName::try_from(name.into())?,
            created_at,
            updated_at,
        ))
    }
}

#[derive(Clone, Debug, Deserialize)]
/// Information required to create a new [`Role`].
pub struct NewRole {
    pub name: RoleName,
}

impl NewRole {
    /// Constructs a new role payload from validated domain types.
    pub fn new(name: RoleName) -> Self {
        Self { name }
    }

    /// Validates raw values before constructing a new role payload.
    pub fn try_new(name: impl Into<String>) -> Result<Self, TypeConstraintError> {
        Ok(Self::new(RoleName::try_from(name.into())?))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Mapping table between users and roles.
pub struct UserRole {
    pub user_id: UserId,
    pub role_id: RoleId,
}

impl UserRole {
    /// Constructs a user-role mapping from validated identifiers.
    pub fn new(user_id: UserId, role_id: RoleId) -> Self {
        Self { user_id, role_id }
    }

    /// Validates raw values before constructing a user-role mapping.
    pub fn try_new(user_id: i32, role_id: i32) -> Result<Self, TypeConstraintError> {
        Ok(Self::new(
            UserId::try_from(user_id)?,
            RoleId::try_from(role_id)?,
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// New entry in the user/role mapping table.
pub struct NewUserRole {
    pub user_id: UserId,
    pub role_id: RoleId,
}

impl NewUserRole {
    /// Constructs a new user-role payload from validated identifiers.
    pub fn new(user_id: UserId, role_id: RoleId) -> Self {
        Self { user_id, role_id }
    }

    /// Validates raw values before constructing a new user-role payload.
    pub fn try_new(user_id: i32, role_id: i32) -> Result<Self, TypeConstraintError> {
        Ok(Self::new(
            UserId::try_from(user_id)?,
            RoleId::try_from(role_id)?,
        ))
    }
}
