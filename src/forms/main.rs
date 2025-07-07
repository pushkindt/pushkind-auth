use serde::Deserialize;

use crate::domain::{
    hub::NewHub as DomainNewHub, role::NewRole as DomainNewRole,
    user::UpdateUser as DomainUpdateUser,
};

#[derive(Deserialize)]
/// Form used on the profile page to update the current user.
pub struct SaveUserForm {
    pub name: Option<String>,
    pub password: Option<String>,
}

impl From<SaveUserForm> for DomainUpdateUser {
    fn from(form: SaveUserForm) -> Self {
        Self {
            name: form.name,
            password: form.password,
        }
    }
}

#[derive(Deserialize)]
/// Request payload for creating a new role via the admin interface.
pub struct AddRoleForm {
    pub name: String,
}

impl From<AddRoleForm> for DomainNewRole {
    fn from(form: AddRoleForm) -> Self {
        Self { name: form.name }
    }
}

#[derive(Deserialize)]
/// Full user editing form used by administrators.
pub struct UpdateUserForm {
    pub id: i32,
    pub name: Option<String>,
    pub password: Option<String>,
    #[serde(default)]
    pub roles: Vec<i32>,
}

impl From<UpdateUserForm> for DomainUpdateUser {
    fn from(form: UpdateUserForm) -> Self {
        Self {
            name: form.name,
            password: form.password,
        }
    }
}

#[derive(Deserialize)]
/// Parameters for adding a new hub.
pub struct AddHubForm {
    pub name: String,
}

impl From<AddHubForm> for DomainNewHub {
    fn from(form: AddHubForm) -> Self {
        Self { name: form.name }
    }
}

#[derive(Deserialize)]
/// Payload for adding a menu entry to a hub.
pub struct AddMenuForm {
    pub name: String,
    pub url: String,
}
