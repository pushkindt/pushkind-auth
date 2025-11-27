use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::domain::types::{RoleId, RoleName, UserId};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Role assigned to a user describing a set of permissions.
pub struct Role {
    pub id: RoleId,
    pub name: RoleName,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize)]
/// Information required to create a new [`Role`].
pub struct NewRole {
    pub name: RoleName,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Mapping table between users and roles.
pub struct UserRole {
    pub user_id: UserId,
    pub role_id: RoleId,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// New entry in the user/role mapping table.
pub struct NewUserRole {
    pub user_id: UserId,
    pub role_id: RoleId,
}
