//! DTOs used by frontend-facing routes that still return JSON payloads.

use serde::Serialize;

use crate::domain::role::Role;
use crate::domain::user::User;

/// Role option exposed in the React admin user modal.
#[derive(Clone, Debug, Serialize)]
pub struct RoleOptionDto {
    pub id: i32,
    pub name: String,
}

impl From<Role> for RoleOptionDto {
    fn from(role: Role) -> Self {
        Self {
            id: role.id.get(),
            name: role.name.into_inner(),
        }
    }
}

/// Editable user data exposed in the React admin user modal.
#[derive(Clone, Debug, Serialize)]
pub struct AdminEditableUserDto {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub roles: Vec<i32>,
}

impl From<User> for AdminEditableUserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id.get(),
            email: user.email.into_inner(),
            name: user.name.map(|name| name.into_inner()).unwrap_or_default(),
            roles: user
                .roles
                .into_iter()
                .map(|role_id| role_id.get())
                .collect(),
        }
    }
}

/// JSON payload used to populate the React admin user modal.
#[derive(Clone, Debug, Serialize)]
pub struct AdminUserModalBootstrap {
    pub user: Option<AdminEditableUserDto>,
    pub roles: Vec<RoleOptionDto>,
}
