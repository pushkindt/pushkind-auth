use serde::Deserialize;

use crate::domain::{
    role::NewRole as DomainNewRole, role::NewUserRole as DomainNewUserRole,
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
pub struct AssignUserRoleForm {
    pub user_id: i32,
    pub role_id: i32,
}

impl From<AssignUserRoleForm> for DomainNewUserRole {
    fn from(form: AssignUserRoleForm) -> Self {
        Self {
            user_id: form.user_id,
            role_id: form.role_id,
        }
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
