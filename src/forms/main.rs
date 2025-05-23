use serde::Deserialize;

use crate::domain::{
    hub::NewHub as DomainNewHub, role::NewRole as DomainNewRole,
    user::UpdateUser as DomainUpdateUser,
};

#[derive(Deserialize)]
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
pub struct AddRoleForm {
    pub name: String,
}

impl From<AddRoleForm> for DomainNewRole {
    fn from(form: AddRoleForm) -> Self {
        Self { name: form.name }
    }
}

#[derive(Deserialize)]
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
pub struct AddHubForm {
    pub name: String,
}

impl From<AddHubForm> for DomainNewHub {
    fn from(form: AddHubForm) -> Self {
        Self { name: form.name }
    }
}
