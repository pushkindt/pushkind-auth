use serde::Deserialize;

use crate::domain::user::UpdateUser as DomainUpdateUser;

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
