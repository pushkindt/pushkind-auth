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

impl<'a> From<&'a SaveUserForm> for DomainUpdateUser<'a> {
    fn from(form: &'a SaveUserForm) -> Self {
        Self {
            name: form.name.as_deref(),
            password: form.password.as_deref(),
        }
    }
}

#[derive(Deserialize)]
/// Request payload for creating a new role via the admin interface.
pub struct AddRoleForm {
    pub name: String,
}

impl<'a> From<&'a AddRoleForm> for DomainNewRole<'a> {
    fn from(form: &'a AddRoleForm) -> Self {
        Self {
            name: form.name.as_str(),
        }
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

impl<'a> From<&'a UpdateUserForm> for DomainUpdateUser<'a> {
    fn from(form: &'a UpdateUserForm) -> Self {
        Self {
            name: form.name.as_deref(),
            password: form.password.as_deref(),
        }
    }
}

#[derive(Deserialize)]
/// Parameters for adding a new hub.
pub struct AddHubForm {
    pub name: String,
}

impl<'a> From<&'a AddHubForm> for DomainNewHub<'a> {
    fn from(form: &'a AddHubForm) -> Self {
        Self {
            name: form.name.as_str(),
        }
    }
}

#[derive(Deserialize)]
/// Payload for adding a menu entry to a hub.
pub struct AddMenuForm {
    pub name: String,
    pub url: String,
}
